use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::core::clock::{frame_to_time, total_frames};
use crate::core::parser::{merge_metadata, parse_html_metadata, resolve_composition_url};
use crate::core::protocol::{HfProtocol, VideoCodec, VideoFormat};
use crate::engine::browser::{BrowserLaunchOptions, launch_browser};
use crate::engine::session::CaptureSession;
use crate::producer::ffmpeg::{FfmpegEncoderConfig, spawn_ffmpeg_encoder};

/// Options for the render pipeline.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub input_path: String,
    pub output_path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub duration: Option<f64>,
    pub format: VideoFormat,
    pub codec: VideoCodec,
    pub crf: u32,
    pub enable_gpu: bool,
    pub quiet: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: "output.mp4".to_string(),
            width: None,
            height: None,
            fps: None,
            duration: None,
            format: VideoFormat::Mp4,
            codec: VideoCodec::H264,
            crf: 18,
            enable_gpu: false,
            quiet: false,
        }
    }
}

/// Statistics returned after a successful video render.
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub total_frames: u64,
    pub duration_seconds: f64,
    pub fps: f64,
    pub width: u32,
    pub height: u32,
    pub elapsed_time_seconds: f64,
    pub output_path: String,
    pub output_bytes: u64,
}

/// Execute the full end-to-end rendering pipeline.
pub async fn render_composition(options: RenderOptions) -> Result<RenderStats> {
    let start_time = Instant::now();

    // 1. Resolve URL and parse any static metadata if local file
    let input_url = resolve_composition_url(&options.input_path);
    let mut initial_proto = HfProtocol::default();

    if Path::new(&options.input_path).exists() {
        if let Ok(html_content) = std::fs::read_to_string(&options.input_path) {
            let static_meta = parse_html_metadata(&html_content);
            initial_proto = merge_metadata(initial_proto, static_meta);
        }
    }

    // Apply explicit CLI overrides
    if let Some(w) = options.width {
        initial_proto.width = w;
    }
    if let Some(h) = options.height {
        initial_proto.height = h;
    }
    if let Some(f) = options.fps {
        initial_proto.fps = f;
    }
    if let Some(d) = options.duration {
        initial_proto.duration = d;
    }

    info!(
        "Rendering composition '{}' to '{}' (target: {}x{} @ {:.1} fps, {:.2}s)",
        options.input_path,
        options.output_path,
        initial_proto.width,
        initial_proto.height,
        initial_proto.fps,
        initial_proto.duration
    );

    // 2. Launch headless browser
    let browser_opts = BrowserLaunchOptions {
        chrome_path: None,
        width: initial_proto.width,
        height: initial_proto.height,
        enable_gpu: options.enable_gpu,
        headless: true,
    };

    let (mut browser, _handle) = launch_browser(&browser_opts).await?;
    let page = browser.new_page("about:blank").await?;

    // 3. Initialize capture session
    let mut session = CaptureSession::new(page, &input_url, initial_proto).await?;
    session.override_settings(options.width, options.height, options.fps, options.duration);
    let resolved_proto = session.protocol();

    let total = total_frames(resolved_proto.duration, resolved_proto.fps);
    if total == 0 {
        return Err(anyhow!("Composition has 0 total frames (duration <= 0)"));
    }

    // 4. Setup FFmpeg encoder
    let ffmpeg_config = FfmpegEncoderConfig {
        output_path: options.output_path.clone(),
        width: resolved_proto.width,
        height: resolved_proto.height,
        fps: resolved_proto.fps,
        format: options.format,
        codec: options.codec,
        crf: options.crf,
        audio_src: None,
    };

    let (child, mut stdin) = spawn_ffmpeg_encoder(&ffmpeg_config)?;

    // 5. Setup progress bar
    let pb = if !options.quiet {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} frames ({percent}%) ETA: {eta}")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(bar)
    } else {
        None
    };

    // 6. Capture loop & stream into FFmpeg
    for frame_idx in 0..total {
        let t = frame_to_time(frame_idx, resolved_proto.fps);

        session.seek(t, None).await?;

        // Screenshot PNG bytes
        let frame_bytes = session.capture_frame_png().await?;

        stdin.write_all(&frame_bytes).await.map_err(|e| {
            anyhow!(
                "Failed to write frame {} bytes to FFmpeg stdin: {e}",
                frame_idx + 1
            )
        })?;

        if let Some(ref bar) = pb {
            bar.inc(1);
        }
    }

    if let Some(ref bar) = pb {
        bar.finish_with_message("All frames captured. Finalizing video encoding...");
    }

    // Flush and close FFmpeg stdin
    stdin.flush().await?;
    drop(stdin);

    // Wait for FFmpeg process to complete
    let output = child.wait_with_output().await?;
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "FFmpeg encoding failed with status {:?}:\n{}",
            output.status,
            err_msg
        ));
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let file_size = std::fs::metadata(&options.output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    info!(
        "Render complete: {} frames encoded in {:.2}s ({:.1} fps render speed). Output size: {:.2} MB",
        total,
        elapsed,
        total as f64 / elapsed.max(0.001),
        file_size as f64 / (1024.0 * 1024.0)
    );

    let _ = browser.close().await;
    _handle.abort();

    Ok(RenderStats {
        total_frames: total,
        duration_seconds: resolved_proto.duration,
        fps: resolved_proto.fps,
        width: resolved_proto.width,
        height: resolved_proto.height,
        elapsed_time_seconds: elapsed,
        output_path: options.output_path,
        output_bytes: file_size,
    })
}
