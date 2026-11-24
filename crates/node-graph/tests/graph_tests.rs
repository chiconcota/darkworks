//! Kiểm thử tích hợp cho darkworks-node-graph

use darkworks_node_graph::{
    ActionScope, AudioBand, BlendMode, BlurType, BuiltinPresetKind, DagEvaluator, DarkNode,
    DistortType, NodeDataType, NodeGraphPreset, ShapeType, TimeMode, TransitionType, VisualGroup,
};
use egui::{Color32, Pos2, Rect, Vec2};
use egui_snarl::{InPinId, OutPinId, Snarl};

#[test]
fn test_node_graph_creation_and_connections() {
    let mut snarl = Snarl::<DarkNode>::new();

    // 1. Tạo ShapeGenerator (Nền xanh)
    let shape_id = snarl.insert_node(
        Pos2::new(50.0, 50.0),
        DarkNode::ShapeGenerator {
            shape_type: ShapeType::RoundedRectangle,
            size: [1920.0, 80.0],
            corner_radius: 8.0,
            color: [0.0, 0.9, 0.4, 1.0], // Green
        },
    );

    // 2. Tạo TextGenerator (Chữ đen)
    let text_id = snarl.insert_node(
        Pos2::new(50.0, 200.0),
        DarkNode::TextGenerator {
            text: "BẢN TIN NÓNG HÔM NAY...".to_string(),
            font_size: 28.0,
            color: [0.0, 0.0, 0.0, 1.0], // Black
            tracking: 0.5,
            line_spacing: 1.1,
        },
    );

    // 3. Tạo Blend Node (Hòa trộn chữ lên nền)
    let blend_id = snarl.insert_node(
        Pos2::new(350.0, 100.0),
        DarkNode::Blend {
            mode: BlendMode::Normal,
            opacity: 1.0,
        },
    );

    // 4. Tạo Final Output
    let output_id = snarl.insert_node(Pos2::new(600.0, 100.0), DarkNode::Output);

    // Nối dây:
    // Shape (Nền xanh) -> Blend input 0 (Background)
    assert!(snarl.connect(
        OutPinId { node: shape_id, output: 0 },
        InPinId { node: blend_id, input: 0 },
    ));

    // Text (Chữ đen) -> Blend input 1 (Foreground)
    assert!(snarl.connect(
        OutPinId { node: text_id, output: 0 },
        InPinId { node: blend_id, input: 1 },
    ));

    // Blend -> Final Output
    assert!(snarl.connect(
        OutPinId { node: blend_id, output: 0 },
        InPinId { node: output_id, input: 0 },
    ));

    // Kiểm tra số lượng node
    assert_eq!(snarl.node_ids().count(), 4);
}

#[test]
fn test_node_data_type_compatibility() {
    assert!(NodeDataType::Texture.can_connect_to(NodeDataType::Texture));
    assert!(NodeDataType::Scalar.can_connect_to(NodeDataType::Scalar));
    assert!(!NodeDataType::AudioFloat.can_connect_to(NodeDataType::Texture));
    assert!(NodeDataType::Any.can_connect_to(NodeDataType::Texture));
    assert!(NodeDataType::Texture.can_connect_to(NodeDataType::Any));
    // AudioFloat có thể kết nối vào Scalar để lái thông số (Procedural Motion)
    assert!(NodeDataType::AudioFloat.can_connect_to(NodeDataType::Scalar));
}

#[test]
fn test_preset_serialization_and_deserialization() {
    let nodes = vec![
        DarkNode::ShapeGenerator {
            shape_type: ShapeType::Rectangle,
            size: [800.0, 60.0],
            corner_radius: 0.0,
            color: [0.0, 1.0, 0.0, 1.0],
        },
        DarkNode::TextGenerator {
            text: "Breaking News".to_string(),
            font_size: 32.0,
            color: [0.0, 0.0, 0.0, 1.0],
            tracking: 0.0,
            line_spacing: 1.0,
        },
        DarkNode::TimeDriver {
            speed: 5.0,
            mode: TimeMode::Linear,
        },
    ];

    let group = VisualGroup::new(
        1,
        "TICKER BANNER PIPELINE".to_string(),
        Pos2::new(10.0, 10.0),
        Vec2::new(400.0, 300.0),
        Color32::from_rgb(0, 150, 255),
    );

    let preset = NodeGraphPreset {
        name: "News Ticker Preset".to_string(),
        version: "1.0.0".to_string(),
        description: "Dải chữ chạy tin tức nền xanh chữ đen góc dưới".to_string(),
        nodes,
        groups: vec![group],
    };

    // Serialize
    let json_str = preset.to_json_string().unwrap_or_default();
    assert!(json_str.contains("Breaking News"));
    assert!(json_str.contains("TICKER BANNER PIPELINE"));

    // Deserialize
    let loaded_preset = NodeGraphPreset::from_json_str(&json_str).unwrap_or(NodeGraphPreset {
        name: String::new(),
        version: String::new(),
        description: String::new(),
        nodes: Vec::new(),
        groups: Vec::new(),
    });
    assert_eq!(loaded_preset.name, "News Ticker Preset");
    assert_eq!(loaded_preset.nodes.len(), 3);
    assert_eq!(loaded_preset.groups.len(), 1);
    assert_eq!(loaded_preset.groups[0].title, "TICKER BANNER PIPELINE");
}

#[test]
fn test_visual_group_translation() {
    let mut group = VisualGroup::new(
        1,
        "Color Grading Group".to_string(),
        Pos2::new(100.0, 100.0),
        Vec2::new(300.0, 200.0),
        Color32::from_rgb(255, 0, 128),
    );

    assert_eq!(group.rect().min, Pos2::new(100.0, 100.0));
    assert_eq!(group.rect().max, Pos2::new(400.0, 300.0));

    // Kiểm tra header rect và resize rect (32x32px)
    assert_eq!(group.header_rect().min, Pos2::new(100.0, 100.0));
    assert_eq!(group.header_rect().max, Pos2::new(400.0, 130.0));
    assert_eq!(group.resize_rect().min, Pos2::new(368.0, 268.0));
    assert_eq!(group.resize_rect().max, Pos2::new(400.0, 300.0));

    // Kiểm tra contains_node
    assert!(group.contains_node(Pos2::new(150.0, 150.0)));
    assert!(!group.contains_node(Pos2::new(50.0, 50.0)));

    // Bảng màu mặc định xoay vòng
    let c0 = VisualGroup::default_color(0);
    let c1 = VisualGroup::default_color(1);
    assert_ne!(c0, c1);

    // Kéo di chuyển group 50px sang phải và 30px xuống dưới
    group.translate(Vec2::new(50.0, 30.0));
    assert_eq!(group.rect().min, Pos2::new(150.0, 130.0));
    assert_eq!(group.rect().max, Pos2::new(450.0, 330.0));
    assert!(group.contains_node(Pos2::new(200.0, 180.0)));
}

#[test]
fn test_dag_evaluator_topological_sort_and_signals() {
    let mut snarl = Snarl::<DarkNode>::new();

    // 1. TimeDriver (Generator)
    let time_id = snarl.insert_node(
        Pos2::new(0.0, 0.0),
        DarkNode::TimeDriver {
            speed: 2.0,
            mode: TimeMode::Linear,
        },
    );

    // 2. AudioReactor (Generator)
    let audio_id = snarl.insert_node(
        Pos2::new(0.0, 100.0),
        DarkNode::AudioReactor {
            band: AudioBand::Bass,
            multiplier: 1.5,
            smooth: 0.1,
        },
    );

    // 3. Transform2D
    let transform_id = snarl.insert_node(
        Pos2::new(200.0, 50.0),
        DarkNode::Transform2D {
            position: [100.0, 200.0],
            scale: [1.0, 1.0],
            rotation_deg: 0.0,
            anchor: [0.5, 0.5],
            opacity: 1.0,
            flip_h: false,
            flip_v: false,
        },
    );

    // 4. Output
    let output_id = snarl.insert_node(Pos2::new(400.0, 50.0), DarkNode::Output);

    // Nối: Audio -> Transform (Scale)
    assert!(snarl.connect(
        OutPinId { node: audio_id, output: 0 },
        InPinId { node: transform_id, input: 2 },
    ));

    // Nối: Transform -> Output
    assert!(snarl.connect(
        OutPinId { node: transform_id, output: 0 },
        InPinId { node: output_id, input: 0 },
    ));

    // Xây dựng kế hoạch thực thi Topo
    let plan = DagEvaluator::build_execution_plan(&snarl).expect("DAG phải hợp lệ không chu trình");
    assert_eq!(plan.steps.len(), 4);

    // Đánh giá tín hiệu tại t = 2.0s, RMS peak = 0.8
    let ctx = DagEvaluator::evaluate_signals(&plan, 2.0, 0.8);
    let time_val = ctx.scalar_outputs.get(&time_id.0).copied().unwrap_or(0.0);
    assert!((time_val - 4.0).abs() < 1e-4);

    let audio_val = ctx.scalar_outputs.get(&audio_id.0).copied().unwrap_or(0.0);
    assert!((audio_val - 1.2).abs() < 1e-4);
}

#[test]
fn test_dag_evaluator_cycle_detection() {
    use darkworks_node_graph::DagError;

    let mut snarl = Snarl::<DarkNode>::new();

    let node_a = snarl.insert_node(
        Pos2::new(0.0, 0.0),
        DarkNode::Transform2D {
            position: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation_deg: 0.0,
            anchor: [0.5, 0.5],
            opacity: 1.0,
            flip_h: false,
            flip_v: false,
        },
    );

    let node_b = snarl.insert_node(
        Pos2::new(200.0, 0.0),
        DarkNode::Transform2D {
            position: [10.0, 10.0],
            scale: [1.0, 1.0],
            rotation_deg: 0.0,
            anchor: [0.5, 0.5],
            opacity: 1.0,
            flip_h: false,
            flip_v: false,
        },
    );

    // Tạo chu trình kín A -> B -> A
    assert!(snarl.connect(
        OutPinId { node: node_a, output: 0 },
        InPinId { node: node_b, input: 0 },
    ));
    assert!(snarl.connect(
        OutPinId { node: node_b, output: 0 },
        InPinId { node: node_a, input: 0 },
    ));

    // DagEvaluator phải phát hiện và trả về CycleDetected
    let res = DagEvaluator::build_execution_plan(&snarl);
    assert!(matches!(res, Err(DagError::CycleDetected(_))));
}

#[test]
fn test_black_node_subgraph_and_presets() {
    use darkworks_node_graph::BlackNodeSubgraph;

    let mut black_node = BlackNodeSubgraph::new("LowerThird Banner");
    black_node.inner_snarl.insert_node(
        Pos2::new(0.0, 0.0),
        DarkNode::VideoInput {
            track_name: "V1".to_string(),
        },
    );
    black_node.inner_snarl.insert_node(Pos2::new(200.0, 0.0), DarkNode::Output);

    black_node.auto_detect_exposed_pins();
    assert_eq!(black_node.exposed_outputs.len(), 1);
    assert_eq!(black_node.exposed_inputs.len(), 1);

    // Xuất ra Preset JSON
    let json_str = black_node.to_preset_json().expect("Export preset JSON thành công");
    assert!(json_str.contains("LowerThird Banner"));

    // Nạp lại từ Preset JSON
    let reloaded = BlackNodeSubgraph::from_preset_json("LowerThird Banner", &json_str)
        .expect("Nạp preset JSON thành công");
    assert_eq!(reloaded.name, "LowerThird Banner");
    assert_eq!(reloaded.exposed_outputs.len(), 1);
}

#[test]
fn test_custom_wgsl_uniform_parser() {
    use darkworks_node_graph::{WgslError, WgslParser};

    let valid_wgsl = r#"
        @group(0) @binding(0) var<uniform> u_speed: f32;
        @group(0) @binding(1) var<uniform> u_center: vec2<f32>;
        @group(0) @binding(2) var<uniform> u_tint: vec4<f32>;

        @fragment
        fn fs_main() -> @location(0) vec4<f32> {
            return u_tint;
        }
    "#;

    let uniforms = WgslParser::parse_uniforms(valid_wgsl).expect("Phải parse thành công");
    assert_eq!(uniforms.len(), 3);
    assert_eq!(uniforms[0].name, "u_speed");
    assert_eq!(uniforms[0].data_type, NodeDataType::Scalar);
    assert_eq!(uniforms[1].name, "u_center");
    assert_eq!(uniforms[1].data_type, NodeDataType::Vec2);
    assert_eq!(uniforms[2].name, "u_tint");
    assert_eq!(uniforms[2].data_type, NodeDataType::Color);

    let invalid_wgsl = "var x = 1.0;";
    assert_eq!(
        WgslParser::parse_uniforms(invalid_wgsl),
        Err(WgslError::MissingFragmentEntry)
    );
}

#[test]
fn test_18_atomic_nodes_creation_and_sockets() {
    // Kiểm tra toàn bộ 18 Atomic Nodes + Output
    let nodes = vec![
        DarkNode::VideoInput { track_name: "V1".to_string() },
        DarkNode::TextGenerator {
            text: "Hello".to_string(),
            font_size: 24.0,
            color: [1.0, 1.0, 1.0, 1.0],
            tracking: 0.0,
            line_spacing: 1.0,
        },
        DarkNode::ShapeGenerator {
            shape_type: ShapeType::Circle,
            size: [100.0, 100.0],
            corner_radius: 0.0,
            color: [1.0, 0.0, 0.0, 1.0],
        },
        DarkNode::TimeDriver { speed: 1.0, mode: TimeMode::Linear },
        DarkNode::ColorGrade {
            exposure: 0.5,
            contrast: 1.2,
            saturation: 1.0,
            temperature: 10.0,
            tint: 0.0,
            gamma: 1.0,
            invert: true,
            black_and_white: false,
        },
        DarkNode::BlurSharpen {
            blur_type: BlurType::BokehBloom,
            radius: 15.0,
            angle_deg: 0.0,
            highlight_boost: 1.2,
        },
        DarkNode::DistortLens {
            distort_type: DistortType::RippleWave,
            intensity: 0.8,
            radius: 1.0,
            center: [0.0, 0.0],
        },
        DarkNode::Keyer {
            key_color: [0.0, 1.0, 0.0],
            similarity: 0.4,
            smoothness: 0.1,
            spill_suppress: 0.5,
        },
        DarkNode::Transform2D {
            position: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation_deg: 0.0,
            anchor: [0.5, 0.5],
            opacity: 1.0,
            flip_h: true,
            flip_v: false,
        },
        DarkNode::CropMask {
            top: 0.1,
            bottom: 0.1,
            left: 0.1,
            right: 0.1,
            feather: 0.05,
        },
        DarkNode::Blend { mode: BlendMode::Difference, opacity: 1.0 },
        DarkNode::Transition {
            transition_type: TransitionType::CrossDissolve,
            progress: 0.5,
            feather: 0.1,
        },
        DarkNode::AudioReactor { band: AudioBand::MasterPeak, multiplier: 2.0, smooth: 0.1 },
        DarkNode::ActionNode { name: "Test".to_string(), scope: ActionScope::AllTracks, is_running: false },
        DarkNode::BlackNode { name: "Sub".to_string(), subgraph_id: "s1".to_string(), exposed_pins: vec![] },
        DarkNode::CustomWgsl { name: "Wgsl".to_string(), code: "".to_string() },
        DarkNode::StickyNote { title: "Note".to_string(), content: "".to_string(), color_idx: 0 },
        DarkNode::Reroute { data_type: NodeDataType::Texture },
        DarkNode::Output,
    ];

    assert_eq!(nodes.len(), 19);
    for node in nodes {
        assert!(!node.title().is_empty());
    }
}

#[test]
fn test_all_5_builtin_presets_validation() {
    let presets = [
        BuiltinPresetKind::NewsTickerBanner,
        BuiltinPresetKind::ContinuousAudioWaveform,
        BuiltinPresetKind::SpotlightInvert,
        BuiltinPresetKind::AudioBeatFlash,
        BuiltinPresetKind::CinematicSoftGlow,
    ];

    for kind in presets {
        let (snarl, groups) = NodeGraphPreset::load_builtin(kind);
        assert!(!groups.is_empty(), "Preset {:?} phải có ít nhất 1 VisualGroup", kind);

        // Đảm bảo toàn bộ 5 Presets đều được DagEvaluator xác thực hợp lệ không chu trình
        let plan = DagEvaluator::build_execution_plan(&snarl)
            .unwrap_or_else(|e| panic!("Preset {:?} thất bại khi xây dựng kế hoạch DAG: {:?}", kind, e));
        assert!(plan.steps.len() >= 4, "Preset {:?} phải có tối thiểu 4 nodes", kind);

        // Đánh giá tín hiệu mẫu tại t=1.0s, RMS peak=0.75
        let ctx = DagEvaluator::evaluate_signals(&plan, 1.0, 0.75);
        assert!((ctx.timeline_time_sec - 1.0).abs() < 1e-5);
    }
}

#[test]
fn test_color_grade_invert_title_and_transform_flips() {
    let invert_node = DarkNode::ColorGrade {
        exposure: 0.0,
        contrast: 1.0,
        saturation: 1.0,
        temperature: 0.0,
        tint: 0.0,
        gamma: 1.0,
        invert: true,
        black_and_white: false,
    };
    assert_eq!(invert_node.title(), "🎨 Color Grade (Invert)");

    let bw_node = DarkNode::ColorGrade {
        exposure: 0.0,
        contrast: 1.0,
        saturation: 1.0,
        temperature: 0.0,
        tint: 0.0,
        gamma: 1.0,
        invert: false,
        black_and_white: true,
    };
    assert_eq!(bw_node.title(), "🎨 Color Grade (B&W)");

    let transform_flipped = DarkNode::Transform2D {
        position: [0.0, 0.0],
        scale: [1.0, 1.0],
        rotation_deg: 0.0,
        anchor: [0.5, 0.5],
        opacity: 1.0,
        flip_h: true,
        flip_v: true,
    };
    if let DarkNode::Transform2D { flip_h, flip_v, .. } = transform_flipped {
        assert!(flip_h);
        assert!(flip_v);
    }
}

#[test]
fn test_visual_group_lock_and_palette() {
    let mut group = VisualGroup::new(
        1,
        "Audio FX Group".to_string(),
        Pos2::new(50.0, 50.0),
        Vec2::new(200.0, 150.0),
        VisualGroup::default_color(0),
    );

    // Mặc định chưa khóa
    assert!(!group.is_locked);

    // Kéo di chuyển khi chưa khóa
    group.translate(Vec2::new(20.0, 10.0));
    assert_eq!(group.min, [70.0, 60.0]);

    // Bật khóa (Lock)
    assert!(group.toggle_lock());
    assert!(group.is_locked);

    // Khi khóa: kéo di chuyển bị vô hiệu hóa
    group.translate(Vec2::new(50.0, 50.0));
    assert_eq!(group.min, [70.0, 60.0]); // Giữ nguyên tọa độ

    // Mở khóa lại
    assert!(!group.toggle_lock());
    assert!(!group.is_locked);

    // Kiểm tra Bảng màu ComfyUI
    assert_eq!(VisualGroup::PALETTE.len(), 6);
    assert_eq!(VisualGroup::COLOR_NAMES.len(), 6);

    group.set_color_index(2); // Dark Turquoise
    assert_eq!(group.color_rgba[0], 0);
    assert_eq!(group.color_rgba[1], 206);
    assert_eq!(group.color_rgba[2], 209);
    assert_eq!(group.color_rgba[3], 35);

    group.set_color(Color32::from_rgb(100, 150, 200));
    assert_eq!(group.color_rgba, [100, 150, 200, 255]);
}

#[test]
fn test_visual_group_auto_fit() {
    let mut group = VisualGroup::new(
        1,
        "Auto Fit Test Group".to_string(),
        Pos2::new(100.0, 100.0),
        Vec2::new(500.0, 400.0),
        VisualGroup::default_color(1),
    );

    // Giả lập 2 node nằm trong group
    let node1 = Rect::from_min_size(Pos2::new(150.0, 160.0), Vec2::new(200.0, 120.0));
    let node2 = Rect::from_min_size(Pos2::new(300.0, 250.0), Vec2::new(200.0, 120.0));
    // 1 node nằm tít ngoài group
    let outside_node = Rect::from_min_size(Pos2::new(900.0, 900.0), Vec2::new(200.0, 120.0));

    let updated = group.auto_fit(vec![node1, node2, outside_node]);
    assert!(updated);

    // Sau khi auto-fit, group phải bao trọn node 1 và node 2
    assert!(group.rect().contains(node1.min));
    assert!(group.rect().contains(node1.max));
    assert!(group.rect().contains(node2.min));
    assert!(group.rect().contains(node2.max));

    // Không bao trọn node bên ngoài
    assert!(!group.rect().contains(outside_node.min));

    // Nhóm rỗng không có node nào nằm trong: auto_fit trả về false
    let mut empty_group = VisualGroup::new(
        2,
        "Empty Group".to_string(),
        Pos2::new(0.0, 0.0),
        Vec2::new(50.0, 50.0),
        VisualGroup::default_color(2),
    );
    assert!(!empty_group.auto_fit(vec![node1, node2]));
}

#[test]
fn test_nested_visual_groups_detection_and_translation() {
    // Group cha lớn (Parent Group)
    let mut parent = VisualGroup::new(
        1,
        "Parent Pipeline".to_string(),
        Pos2::new(100.0, 100.0),
        Vec2::new(600.0, 400.0),
        VisualGroup::default_color(0),
    );

    // Group con nằm hoàn toàn bên trong Group cha (Child Group)
    let mut child = VisualGroup::new(
        2,
        "Child Sub-group".to_string(),
        Pos2::new(200.0, 200.0),
        Vec2::new(250.0, 180.0),
        VisualGroup::default_color(1),
    );

    // Group độc lập nằm ngoài (Sibling Group)
    let mut sibling = VisualGroup::new(
        3,
        "Outside Group".to_string(),
        Pos2::new(800.0, 100.0),
        Vec2::new(200.0, 200.0),
        VisualGroup::default_color(2),
    );

    // Kiểm tra quan hệ lồng nhau qua strict containment (contains_group)
    assert!(parent.contains_group(&child));
    assert!(!parent.contains_group(&sibling));

    // Hai nhóm chỉ giao cắt/chồng đè một phần (như nhóm 1 và nhóm 2)
    let overlap1 = VisualGroup::new(4, "Nhóm 1".to_string(), Pos2::new(120.0, 120.0), Vec2::new(450.0, 300.0), VisualGroup::default_color(0));
    let mut overlap2 = VisualGroup::new(5, "Nhóm 2".to_string(), Pos2::new(170.0, 170.0), Vec2::new(450.0, 300.0), VisualGroup::default_color(1));
    assert!(!overlap1.contains_group(&overlap2), "Giao cắt một phần không được xem là lồng nhau");
    assert!(!overlap2.contains_group(&overlap1), "Giao cắt một phần không được xem là lồng nhau");

    // Giả lập logic tịnh tiến đồng bộ khi kéo group cha (Strict Containment)
    let delta = Vec2::new(50.0, -30.0);
    let old_parent_rect = parent.rect();
    parent.translate(delta);

    // Kéo các group con nằm trọn 100% trong old_parent_rect
    let groups = [&mut child, &mut sibling];
    for g in groups {
        if old_parent_rect.contains(g.rect().min) && old_parent_rect.contains(g.rect().max) {
            g.translate(delta);
        }
    }

    assert_eq!(parent.rect().min, Pos2::new(150.0, 70.0));
    assert_eq!(child.rect().min, Pos2::new(250.0, 170.0)); // Tịnh tiến cùng cha vì lồng trọn 100%
    assert_eq!(sibling.rect().min, Pos2::new(800.0, 100.0)); // Giữ nguyên vị trí

    // Giả lập kéo overlap1: overlap2 chỉ giao cắt một phần với overlap1 nên KHÔNG bị kéo theo
    let old_overlap1_rect = overlap1.rect();
    if old_overlap1_rect.contains(overlap2.rect().min) && old_overlap1_rect.contains(overlap2.rect().max) {
        overlap2.translate(delta);
    }
    assert_eq!(overlap2.rect().min, Pos2::new(170.0, 170.0)); // Giữ nguyên vị trí, không bị dính chùm!
}

#[test]
fn test_visual_group_close_and_drag_rects() {
    let group = VisualGroup::new(
        1,
        "Test Group".into(),
        Pos2::new(100.0, 100.0),
        Vec2::new(300.0, 200.0),
        egui::Color32::from_rgb(50, 50, 50),
    );
    let close = group.close_button_rect();
    let drag = group.header_drag_rect();
    assert_eq!(close.max.x, 396.0);
    assert_eq!(drag.max.x, 370.0);
    assert!(!drag.intersects(close), "Header drag area must not overlap close button");
}


