use anyhow::{Result, anyhow};
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin, Command};
use tracing::info;

use crate::core::protocol::{VideoCodec, VideoFormat};

/// Configuration for the FFmpeg video encoder.
#[derive(Debug, Clone)]
pub struct FfmpegEncoderConfig {
    pub output_path: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub format: VideoFormat,
    pub codec: VideoCodec,
    pub crf: u32,
    pub audio_src: Option<String>,
}

impl Default for FfmpegEncoderConfig {
    fn default() -> Self {
        Self {
            output_path: "output.mp4".to_string(),
            width: 1920,
            height: 1080,
            fps: 30.0,
            format: VideoFormat::Mp4,
            codec: VideoCodec::H264,
            crf: 18,
            audio_src: None,
        }
    }
}

/// Spawns an FFmpeg child process ready to receive piped image frames via stdin.
pub fn spawn_ffmpeg_encoder(config: &FfmpegEncoderConfig) -> Result<(Child, ChildStdin)> {
    let mut cmd = Command::new("ffmpeg");

    // Overwrite output files without asking
    cmd.arg("-y");

    // Input: read PNG frames from stdin
    cmd.arg("-f")
        .arg("image2pipe")
        .arg("-vcodec")
        .arg("png")
        .arg("-r")
        .arg(format!("{:.2}", config.fps))
        .arg("-i")
        .arg("-");

    // Optional audio input
    if let Some(audio_path) = &config.audio_src {
        if Path::new(audio_path).exists() {
            cmd.arg("-i").arg(audio_path);
            cmd.arg("-c:a").arg("aac").arg("-b:a").arg("192k");
        }
    }

    // Video filters: ensure even dimensions for yuv420p
    cmd.arg("-vf").arg("pad=ceil(iw/2)*2:ceil(ih/2)*2");

    // Codec & Format specific arguments
    match config.format {
        VideoFormat::Mp4 => {
            match config.codec {
                VideoCodec::H265 => {
                    cmd.arg("-c:v")
                        .arg("libx265")
                        .arg("-crf")
                        .arg(config.crf.to_string())
                        .arg("-tag:v")
                        .arg("hvc1")
                        .arg("-pix_fmt")
                        .arg("yuv420p");
                }
                _ => {
                    cmd.arg("-c:v")
                        .arg("libx264")
                        .arg("-crf")
                        .arg(config.crf.to_string())
                        .arg("-preset")
                        .arg("medium")
                        .arg("-pix_fmt")
                        .arg("yuv420p");
                }
            }
            cmd.arg("-movflags").arg("+faststart");
        }
        VideoFormat::Mov => {
            if config.codec == VideoCodec::ProRes {
                cmd.arg("-c:v")
                    .arg("prores_ks")
                    .arg("-profile:v")
                    .arg("3")
                    .arg("-pix_fmt")
                    .arg("yuva444p10le");
            } else {
                cmd.arg("-c:v")
                    .arg("libx264")
                    .arg("-pix_fmt")
                    .arg("yuv420p");
            }
        }
        VideoFormat::Webm => {
            cmd.arg("-c:v")
                .arg("libvpx-vp9")
                .arg("-crf")
                .arg(config.crf.to_string())
                .arg("-b:v")
                .arg("0")
                .arg("-pix_fmt")
                .arg("yuva420p");
        }
        VideoFormat::Gif => {
            cmd.arg("-vf")
                .arg(format!("fps={:.2},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse", config.fps, config.width));
        }
        VideoFormat::PngSequence => {
            return Err(anyhow!(
                "PngSequence is handled directly via file writer, not FFmpeg stream"
            ));
        }
    }

    cmd.arg(&config.output_path);

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    info!("Spawning FFmpeg process: {:?}", cmd);

    let mut child = cmd
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn ffmpeg: {e}. Is ffmpeg installed?"))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("Failed to acquire ffmpeg stdin pipe"))?;

    Ok((child, stdin))
}

/// Helper to write frame data to FFmpeg stdin asynchronously.
pub async fn write_frame_to_ffmpeg(stdin: &mut ChildStdin, data: &[u8]) -> Result<()> {
    stdin
        .write_all(data)
        .await
        .map_err(|e| anyhow!("Failed to write frame bytes to ffmpeg stdin: {e}"))?;
    Ok(())
}
