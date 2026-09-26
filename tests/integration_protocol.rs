use hyperframes::core::color_grading::{find_preset, standard_presets};
use hyperframes::core::parser::{merge_metadata, parse_html_metadata};
use hyperframes::core::protocol::{
    HfMediaElement, HfProtocol, HfSeekOptions, HfTransitionMeta, VideoCodec, VideoFormat,
};
use hyperframes::core::safe_path::is_safe_path;
use hyperframes::core::vfx::{HfVfxChain, HfVfxNode};
use hyperframes::engine::even_dimensions::{
    even_up, requires_even_dimensions, with_even_dimension_pad,
};
use hyperframes::producer::render_request::RenderRequest;
use tempfile::tempdir;

#[test]
fn test_integration_protocol_and_metadata_merge() {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Demo Clip</title>
        <meta name="hyperframes:duration" content="12.5">
        <meta name="hyperframes:fps" content="24">
        <meta name="hyperframes:resolution" content="1920x1080">
    </head>
    <body>
        <video id="bg" src="footage.mp4" data-start="0" data-end="12.5"></video>
    </body>
    </html>
    "#;

    let meta = parse_html_metadata(html);
    let proto = merge_metadata(HfProtocol::default(), meta);

    assert_eq!(proto.duration, 12.5);
    assert_eq!(proto.fps, 24.0);
    assert_eq!(proto.width, 1920);
    assert_eq!(proto.height, 1080);
}

#[test]
fn test_integration_transitions_and_media_elements() {
    let proto = HfProtocol {
        duration: 20.0,
        fps: 30.0,
        width: 3840,
        height: 2160,
        media: vec![HfMediaElement {
            element_id: "hero-video".to_string(),
            src: "assets/hero.mp4".to_string(),
            start_time: 0.0,
            end_time: 10.0,
            media_offset: 0.0,
            volume: 0.8,
            has_audio: true,
        }],
        transitions: vec![HfTransitionMeta {
            time: 10.0,
            duration: 1.5,
            shader: Some("cross-zoom".to_string()),
            ease: "power2.inOut".to_string(),
            from_scene: "scene-1".to_string(),
            to_scene: "scene-2".to_string(),
        }],
    };

    assert_eq!(proto.media.len(), 1);
    assert_eq!(proto.transitions.len(), 1);
    assert_eq!(proto.transitions[0].shader.as_deref(), Some("cross-zoom"));

    let opts = HfSeekOptions {
        suppress_events: true,
        sub_frame_divisions: Some(4),
    };
    assert!(opts.suppress_events);
    assert_eq!(opts.sub_frame_divisions, Some(4));
}

#[test]
fn test_integration_color_grading_and_vfx() {
    let presets = standard_presets();
    assert!(presets.len() >= 5);

    let pop = find_preset("bright-pop").unwrap();
    assert!(pop.adjust.exposure > 0.0);
    assert!(pop.adjust.contrast > 0.0);

    let vfx = HfVfxChain::new(vec![
        HfVfxNode::new("n1", "wave-warp").with_param("waveHeight", 10.0),
    ]);
    let json = vfx.to_json().unwrap();
    let recovered = HfVfxChain::from_json(&json).unwrap();
    assert_eq!(vfx, recovered);
}

#[test]
fn test_integration_safe_path_and_even_dims() {
    let dir = tempdir().unwrap();
    assert!(is_safe_path(
        dir.path(),
        dir.path().join("sub").join("file.mp4")
    ));
    assert!(!is_safe_path(dir.path(), "/etc/passwd"));

    assert_eq!(even_up(1079), 1080);
    assert!(requires_even_dimensions("yuv420p"));
    let pad = with_even_dimension_pad("", "yuv420p", Some(1921), Some(1080));
    assert!(pad.contains("pad="));
}

#[test]
fn test_integration_render_request_serialization() {
    let mut req = RenderRequest::new("/my/project", "final.mp4");
    req.options.format = VideoFormat::Mov;
    req.options.codec = VideoCodec::ProRes;

    let json = req.to_json().unwrap();
    let deserialized = RenderRequest::from_json(&json).unwrap();
    assert_eq!(req, deserialized);
}
