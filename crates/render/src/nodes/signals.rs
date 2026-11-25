//! Bộ giải quyết nguồn tín hiệu Video (`VideoInput`), Âm thanh (`AudioReactor`) và `Mod` sockets.

use std::collections::{HashMap, HashSet};

use darkworks_node_graph_core::{AudioBand, DagDocPlan, DarkNode, EvaluationContext, NodeGraphDoc};
use filmcraft_frame::AudioBuffer;
use filmcraft_geom::Affine;
use filmcraft_project::{ClipId, Project, Sequence, Track, TrackItem};
use filmcraft_time::Tick;

use crate::image::Image;
use crate::node_graph::eval_item_graph_inner;
use crate::{LayerWant, RenderOptions, SourceProvider, is_adjustment, item_layer_ex};

const MAX_VIDEO_INPUT_DEPTH: u8 = 4;

fn parse_track_index(track_name: &str, prefix: char) -> Option<usize> {
    let trimmed = track_name.trim();
    let rest = trimmed
        .strip_prefix(prefix)
        .or_else(|| trimmed.strip_prefix(prefix.to_ascii_lowercase()))?;
    let one_based: usize = rest.parse().ok()?;
    one_based.checked_sub(1)
}

fn find_video_track<'a>(seq: &'a Sequence, track_name: &str) -> Option<(usize, &'a Track)> {
    if let Some(idx) = parse_track_index(track_name, 'V')
        && let Some(tr) = seq.video_tracks.get(idx)
    {
        return Some((idx, tr));
    }
    seq.video_tracks
        .iter()
        .enumerate()
        .find(|(_, tr)| tr.name.eq_ignore_ascii_case(track_name.trim()))
}

fn find_audio_track<'a>(seq: &'a Sequence, track_name: &str) -> Option<&'a Track> {
    if let Some(idx) = parse_track_index(track_name, 'A')
        && let Some(tr) = seq.audio_tracks.get(idx)
    {
        return Some(tr);
    }
    seq.audio_tracks
        .iter()
        .find(|tr| tr.name.eq_ignore_ascii_case(track_name.trim()))
}

pub(crate) fn fit_image(img: &Image, w: usize, h: usize) -> Image {
    if img.w == w && img.h == h {
        img.clone()
    } else {
        img.transformed(
            w,
            h,
            &Affine::scale(
                w as f64 / img.w.max(1) as f64,
                h as f64 / img.h.max(1) as f64,
            ),
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_video_input(
    track_name: &str,
    project: &Project,
    seq: &Sequence,
    current_item: &TrackItem,
    t: Tick,
    opts: RenderOptions,
    sources: &dyn SourceProvider,
    clip_image: Option<&Image>,
    below_image: Option<&Image>,
    w: usize,
    h: usize,
    depth: u8,
    visited: &mut HashSet<ClipId>,
) -> Image {
    let key = track_name.trim();
    if key.is_empty() || key.eq_ignore_ascii_case("Auto") || key.eq_ignore_ascii_case("Clip") {
        return clip_image
            .or(below_image)
            .map_or_else(|| Image::new(w, h), |img| fit_image(img, w, h));
    }
    if key.eq_ignore_ascii_case("All Below") || key.eq_ignore_ascii_case("Below") {
        return below_image
            .or(clip_image)
            .map_or_else(|| Image::new(w, h), |img| fit_image(img, w, h));
    }

    let Some((_, track)) = find_video_track(seq, key) else {
        return Image::new(w, h);
    };

    // Nếu `current_item` nằm ngay trên track được chọn và đã giải mã sẵn `clip_image`
    if track.items.iter().any(|it| it.id == current_item.id)
        && let Some(img) = clip_image
    {
        return fit_image(img, w, h);
    }

    let Some(target) = track.item_at(t) else {
        return Image::new(w, h);
    };
    if !target.enabled || depth >= MAX_VIDEO_INPUT_DEPTH || visited.contains(&target.id) {
        return Image::new(w, h);
    }

    if is_adjustment(project, target) {
        if target.node_graph.is_some()
            && let Some(adj) = eval_item_graph_inner(
                project,
                seq,
                target,
                t,
                opts,
                sources,
                None,
                below_image,
                depth + 1,
                visited,
            )
        {
            return fit_image(&adj, w, h);
        }
        return Image::new(w, h);
    }

    let raw = item_layer_ex(
        project,
        seq,
        target,
        t,
        opts,
        sources,
        LayerWant {
            tc: "",
            allow_region: false,
        },
    )
    .map(|l| l.into_full(w, h).0);

    if target.node_graph.is_some()
        && let Some(graded) = eval_item_graph_inner(
            project,
            seq,
            target,
            t,
            opts,
            sources,
            raw.as_ref(),
            below_image,
            depth + 1,
            visited,
        )
    {
        return fit_image(&graded, w, h);
    }

    raw.map_or_else(|| Image::new(w, h), |img| fit_image(&img, w, h))
}

#[derive(Clone, Copy)]
struct AudioWin {
    start: i64,
    frames: usize,
    sr: u32,
}

fn resolve_audio_buffer(
    track_name: &str,
    project: &Project,
    seq: &Sequence,
    current_item: &TrackItem,
    t: Tick,
    win: AudioWin,
    sources: &dyn SourceProvider,
) -> AudioBuffer {
    let key = track_name.trim();
    if key.is_empty() || key.eq_ignore_ascii_case("Master") || key.eq_ignore_ascii_case("Master Mix") {
        return crate::audio::mix_sequence(project, seq, win.start, win.frames, sources);
    }
    if key.eq_ignore_ascii_case("Linked") {
        if let Some(link_id) = current_item.link {
            let linked_track = seq
                .audio_tracks
                .iter()
                .find(|tr| tr.items.iter().any(|it| it.link == Some(link_id) && it.range().contains(t)))
                .or_else(|| {
                    seq.audio_tracks
                        .iter()
                        .find(|tr| tr.items.iter().any(|it| it.link == Some(link_id)))
                });
            if let Some(tr) = linked_track {
                return crate::audio::track_input(project, tr, win.start, win.frames, win.sr, sources);
            }
        }
        return crate::audio::mix_sequence(project, seq, win.start, win.frames, sources);
    }
    if let Some(tr) = find_audio_track(seq, key) {
        return crate::audio::track_input(project, tr, win.start, win.frames, win.sr, sources);
    }
    AudioBuffer {
        sample_rate: win.sr,
        channels: Vec::new(),
    }
}

/// Tính RMS riêng cho từng `AudioReactor` node theo nguồn track (`Master`/`Linked`/`A1`/`A2`...), `band` và `smooth`.
pub(crate) fn compute_audio_rms_map(
    project: &Project,
    seq: &Sequence,
    current_item: &TrackItem,
    t: Tick,
    plan: &DagDocPlan,
    sources: &dyn SourceProvider,
) -> HashMap<usize, f32> {
    let mut reactors: Vec<(usize, AudioBand, f32, String)> = Vec::new();
    let mut max_smooth = 0.0f32;
    for s in &plan.steps {
        if let DarkNode::AudioReactor {
            band,
            smooth,
            track_name,
            ..
        } = &s.node
        {
            let sm = smooth.clamp(0.0, 1.0);
            max_smooth = max_smooth.max(sm);
            reactors.push((s.node_id, *band, sm, track_name.clone()));
        }
    }
    let mut result = HashMap::new();
    if reactors.is_empty() {
        return result;
    }

    let sr = seq.settings.sample_rate.max(8000);
    let win_frames = (1024.0 + max_smooth * 3072.0).round() as usize;
    let start = ((t.seconds() * sr as f64).round() as i64).saturating_sub((win_frames / 2) as i64);
    let win = AudioWin { start, frames: win_frames, sr };

    let mut buffers: HashMap<String, AudioBuffer> = HashMap::new();
    for (id, band, smooth, track_name) in reactors {
        let cache_key = track_name.trim().to_ascii_lowercase();
        let buf = buffers.entry(cache_key).or_insert_with(|| {
            resolve_audio_buffer(&track_name, project, seq, current_item, t, win, sources)
        });
        if buf.channels.is_empty() || buf.channels[0].is_empty() {
            result.insert(id, 0.0);
            continue;
        }

        let ch0 = &buf.channels[0];
        let n = ch0.len();
        let local_win = ((1024.0 + smooth * 3072.0).round() as usize).min(n).max(1);
        let lo = (n - local_win) / 2;
        let hi = (lo + local_win).min(n);

        let alpha = match band {
            AudioBand::SubBass => 0.04,
            AudioBand::Bass => 0.09,
            AudioBand::LowMid => 0.22,
            AudioBand::HighMid => 0.45,
            AudioBand::Treble | AudioBand::MasterPeak => 1.0,
        };

        let mut sum_sq = 0.0f64;
        let mut lp = 0.0f32;
        let mut prev_s = 0.0f32;
        for i in lo..hi {
            let mut s = ch0[i];
            if buf.channels.len() > 1 {
                s = 0.5 * (s + buf.channels[1][i]);
            }
            let val = match band {
                AudioBand::MasterPeak => s,
                AudioBand::Treble => {
                    let diff = if i > lo { s - prev_s } else { s };
                    prev_s = s;
                    diff
                }
                _ => {
                    lp += alpha * (s - lp);
                    lp
                }
            };
            sum_sq += (val as f64) * (val as f64);
        }
        let rms = ((sum_sq / (hi - lo).max(1) as f64).sqrt() as f32 * 2.2).clamp(0.0, 1.0);
        result.insert(id, rms);
    }
    result
}

pub(crate) fn input_texture(
    doc: &NodeGraphDoc,
    textures: &HashMap<usize, Image>,
    node_id: usize,
    pin: usize,
    w: usize,
    h: usize,
) -> Image {
    doc.source_for_pin(node_id, pin)
        .and_then(|(src_id, _)| textures.get(&src_id))
        .cloned()
        .unwrap_or_else(|| Image::new(w, h))
}

pub(crate) fn modulated_scalar(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return base * v;
    }
    base
}

pub(crate) fn modulated_scalar_mul(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return base * v.max(0.01);
    }
    base
}

pub(crate) fn modulated_scalar_add(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return base + v * 36.0;
    }
    base
}

pub(crate) fn modulated_scalar_add_raw(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return base + v;
    }
    base
}

pub(crate) fn modulated_scalar_override(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: f32) -> f32 {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.scalar_outputs.get(&src_id)
    {
        return v;
    }
    base
}

pub(crate) fn modulated_vec2(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: [f32; 2]) -> [f32; 2] {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin)
        && let Some(&v) = ctx.vec2_outputs.get(&src_id)
    {
        return [base[0] + v[0], base[1] + v[1]];
    }
    base
}

pub(crate) fn modulated_vec2_add(doc: &NodeGraphDoc, ctx: &EvaluationContext, node_id: usize, pin: usize, base: [f32; 2]) -> [f32; 2] {
    if let Some((src_id, _)) = doc.source_for_pin(node_id, pin) {
        if let Some(&v) = ctx.vec2_outputs.get(&src_id)
            && (v[0].abs() > 1e-5 || v[1].abs() > 1e-5)
        {
            return [base[0] + v[0], base[1] + v[1]];
        }
        if let Some(&s) = ctx.scalar_outputs.get(&src_id) {
            return [base[0] + s * 60.0, base[1]];
        }
    }
    base
}
