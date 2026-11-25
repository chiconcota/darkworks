//! Runtime JIT biên dịch và thực thi `CustomWgsl` Code Node trên GPU qua `wgpu`.
//!
//! Dùng chung thiết bị off-screen của [`crate::export_renderer::shared_headless_device`],
//! kiểm tra lỗi biên dịch an toàn qua `push_error_scope` (không panic khi sai cú pháp WGSL)
//! và cache `RenderPipeline` theo hash mã nguồn shader.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use filmcraft_render::Image;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const MAX_CACHED_PIPELINES: usize = 32;

const PREAMBLE: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i32(vid & 1u) * 4 - 1);
    let y = f32(i32(vid >> 1u) * 4 - 1);
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
    return out;
}
"#;

const DEFAULT_BINDINGS: &str = r#"
@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_samp: sampler;
@group(0) @binding(2) var<uniform> u_ctx: vec4<f32>;
"#;

struct CustomWgslState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    bgl: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    sampler: wgpu::Sampler,
    pipelines: HashMap<u64, Result<wgpu::RenderPipeline, String>>,
}

static WGSL_ENGINE: OnceLock<Option<Mutex<CustomWgslState>>> = OnceLock::new();

fn engine() -> Option<&'static Mutex<CustomWgslState>> {
    WGSL_ENGINE
        .get_or_init(|| {
            let (_, device, queue) = crate::export_renderer::shared_headless_device()?;
            let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("custom_wgsl_bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("custom_wgsl_layout"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0,
            });
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("custom_wgsl_sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
            Some(Mutex::new(CustomWgslState {
                device,
                queue,
                bgl,
                pipeline_layout,
                sampler,
                pipelines: HashMap::new(),
            }))
        })
        .as_ref()
}

fn assemble_shader(user_code: &str) -> String {
    let mut full = String::new();
    if !user_code.contains("fn vs_main") {
        full.push_str(PREAMBLE);
    }
    if !user_code.contains("var src_tex") {
        full.push_str(DEFAULT_BINDINGS);
    }
    full.push_str(user_code);
    full
}

fn compile_pipeline(state: &CustomWgslState, user_code: &str) -> Result<wgpu::RenderPipeline, String> {
    let source = assemble_shader(user_code);
    let scope = state.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let module = state.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custom_wgsl_module"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    if let Some(Some(err)) = crate::poll_now(scope.pop()) {
        return Err(format!("WGSL compile error: {err}"));
    }

    let scope = state.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("custom_wgsl_pipeline"),
        layout: Some(&state.pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    match crate::poll_now(scope.pop()) {
        Some(None) => Ok(pipeline),
        Some(Some(err)) => Err(format!("WGSL pipeline error: {err}")),
        None => Err("WGSL validation timed out".to_string()),
    }
}

/// Biên dịch (có cache) và chạy fragment shader WGSL trên `input`, trả về [`Image`] mới.
pub fn run_custom_wgsl(code: &str, input: &Image, time_sec: f32, param_scalar: f32) -> Result<Image, String> {
    if input.w == 0 || input.h == 0 {
        return Ok(input.clone());
    }
    let Some(lock) = engine() else {
        return Err("GPU adapter unavailable for CustomWgsl".to_string());
    };
    let mut state = lock.lock().map_err(|_| "CustomWgsl lock poisoned".to_string())?;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    code.hash(&mut hasher);
    let key = hasher.finish();

    if !state.pipelines.contains_key(&key) {
        if state.pipelines.len() >= MAX_CACHED_PIPELINES {
            state.pipelines.clear();
        }
        let compiled = compile_pipeline(&state, code);
        state.pipelines.insert(key, compiled);
    }
    let pipeline = match state.pipelines.get(&key) {
        Some(Ok(p)) => p.clone(),
        Some(Err(e)) => return Err(e.clone()),
        None => return Err("Missing compiled pipeline".to_string()),
    };

    let (w, h) = (input.w as u32, input.h as u32);
    let extent = wgpu::Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };

    let src_tex = state.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custom_wgsl_src"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut f16_bytes = Vec::with_capacity(input.px.len() * 2);
    for &v in &input.px {
        f16_bytes.extend_from_slice(&crate::f32_to_f16(v).to_ne_bytes());
    }
    state.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &src_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &f16_bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 8),
            rows_per_image: Some(h),
        },
        extent,
    );

    let dst_tex = state.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custom_wgsl_dst"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let mut uniform_bytes = [0u8; 16];
    let floats = [time_sec, param_scalar, w as f32, h as f32];
    for (idx, val) in floats.iter().enumerate() {
        uniform_bytes[idx * 4..idx * 4 + 4].copy_from_slice(&val.to_ne_bytes());
    }
    let u_buf = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("custom_wgsl_uniform"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    state.queue.write_buffer(&u_buf, 0, &uniform_bytes);

    let src_view = src_tex.create_view(&Default::default());
    let dst_view = dst_tex.create_view(&Default::default());
    let bg = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("custom_wgsl_bg"),
        layout: &state.bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&state.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: u_buf.as_entire_binding(),
            },
        ],
    });

    let row_bytes = (w * 8).div_ceil(256) * 256;
    let read_buf = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("custom_wgsl_readback"),
        size: (row_bytes * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut enc = state.device.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("custom_wgsl_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dst_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &dst_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &read_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row_bytes),
                rows_per_image: Some(h),
            },
        },
        extent,
    );
    state.queue.submit([enc.finish()]);

    let slice = read_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    crate::wait_mapped(&state.device, &rx).ok_or_else(|| "CustomWgsl readback timed out".to_string())?;
    let data = slice.get_mapped_range().map_err(|e| format!("map error: {e}"))?;
    let mut out_px = Vec::with_capacity(input.px.len());
    for y in 0..h {
        let row = &data[(y * row_bytes) as usize..(y * row_bytes + w * 8) as usize];
        for pair in row.as_chunks::<2>().0 {
            let bits = u16::from_ne_bytes([pair[0], pair[1]]);
            out_px.push(crate::f16_to_f32(bits));
        }
    }
    drop(data);
    read_buf.unmap();
    Ok(Image {
        w: input.w,
        h: input.h,
        px: out_px,
    })
}
