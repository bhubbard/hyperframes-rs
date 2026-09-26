use hyperframes::core::protocol::{VideoCodec, VideoFormat};
use hyperframes::engine::ffprobe::extract_media_metadata;
use hyperframes::producer::pipeline::{RenderOptions, render_composition};
use tempfile::tempdir;

#[tokio::test]
async fn test_integration_end_to_end_render() {
    let tmp = tempdir().unwrap();
    let html_path = tmp.path().join("index.html");
    let out_mp4 = tmp.path().join("output.mp4");

    let sample_html = r#"<!DOCTYPE html>
<html>
<head>
    <meta name="hyperframes:duration" content="1.0">
    <meta name="hyperframes:fps" content="24">
    <meta name="hyperframes:resolution" content="640x360">
    <style>
        body { margin: 0; background: #111; display: flex; align-items: center; justify-content: center; height: 100vh; }
        .box { width: 100px; height: 100px; background: #00f0ff; }
    </style>
</head>
<body>
    <div id="box" class="box"></div>
    <script>
        window.__hf = {
            duration: 1.0,
            fps: 24,
            seek: function(t) {
                document.getElementById('box').style.transform = 'rotate(' + (t * 360) + 'deg)';
            }
        };
    </script>
</body>
</html>"#;

    std::fs::write(&html_path, sample_html).unwrap();

    let render_opts = RenderOptions {
        input_path: html_path.to_string_lossy().to_string(),
        output_path: out_mp4.to_string_lossy().to_string(),
        width: Some(640),
        height: Some(360),
        fps: Some(24.0),
        duration: Some(1.0),
        format: VideoFormat::Mp4,
        codec: VideoCodec::H264,
        crf: 22,
        enable_gpu: false,
        quiet: true,
    };

    let stats = render_composition(render_opts).await.unwrap();

    assert_eq!(stats.total_frames, 24);
    assert_eq!(stats.fps, 24.0);
    assert_eq!(stats.width, 640);
    assert_eq!(stats.height, 360);
    assert!(out_mp4.exists());
    assert!(stats.output_bytes > 0);

    // Verify video metadata with ffprobe
    if let Ok(meta) = extract_media_metadata(&out_mp4).await {
        assert_eq!(meta.width, Some(640));
        assert_eq!(meta.height, Some(360));
        assert_eq!(meta.video_codec.as_deref(), Some("h264"));
        if let Some(dur) = meta.duration_seconds {
            assert!((dur - 1.0).abs() < 0.2);
        }
    }
}
