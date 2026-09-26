use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::process::Command;

/// Metadata extracted from a video or audio file using ffprobe.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaMetadata {
    pub duration_seconds: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub pixel_format: Option<String>,
    pub has_alpha: bool,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub total_frames: Option<u64>,
}

/// Check if a pixel format contains an alpha channel.
/// Matches upstream regex: `^(?:yuva|rgba|argb|bgra|abgr|gbrap|ya|ayuv)`
pub fn pixel_format_has_alpha(pix_fmt: &str) -> bool {
    let lower = pix_fmt.to_lowercase();
    lower.starts_with("yuva")
        || lower.starts_with("rgba")
        || lower.starts_with("argb")
        || lower.starts_with("bgra")
        || lower.starts_with("abgr")
        || lower.starts_with("gbrap")
        || lower.starts_with("ya")
        || lower.starts_with("ayuv")
}

/// Parse frame rate string like "30/1" or "30000/1001" or "24".
pub fn parse_frame_rate(rate_str: &str) -> Option<f64> {
    let trimmed = rate_str.trim();
    if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() == 2 {
            let num: f64 = parts[0].trim().parse().ok()?;
            let den: f64 = parts[1].trim().parse().ok()?;
            if den > 0.0 && num > 0.0 {
                return Some(num / den);
            }
        }
    } else if let Ok(val) = trimmed.parse::<f64>() {
        if val > 0.0 {
            return Some(val);
        }
    }
    None
}

/// Run ffprobe on a media file to inspect streams and container metadata.
pub async fn extract_media_metadata(file_path: impl AsRef<Path>) -> Result<MediaMetadata> {
    let path_ref = file_path.as_ref();
    if !path_ref.exists() {
        return Err(anyhow!("Media file not found: {}", path_ref.display()));
    }

    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(path_ref.to_string_lossy().as_ref())
        .output()
        .await
        .map_err(|e| anyhow!("Failed to execute ffprobe: {e}. Is ffmpeg/ffprobe installed?"))?;

    if !output.status.success() {
        return Err(anyhow!("ffprobe exited with status {:?}", output.status));
    }

    let json_text = String::from_utf8_lossy(&output.stdout);
    parse_ffprobe_json(&json_text)
}

/// Parse JSON output from ffprobe.
pub fn parse_ffprobe_json(json_str: &str) -> Result<MediaMetadata> {
    let root: serde_json::Value = serde_json::from_str(json_str)?;
    let mut meta = MediaMetadata::default();

    if let Some(format_obj) = root.get("format") {
        if let Some(d) = format_obj
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
        {
            meta.duration_seconds = Some(d);
        }
    }

    if let Some(streams) = root.get("streams").and_then(|v| v.as_array()) {
        for s in streams {
            let codec_type = s.get("codec_type").and_then(|v| v.as_str()).unwrap_or("");
            if codec_type == "video" && meta.video_codec.is_none() {
                meta.video_codec = s
                    .get("codec_name")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                meta.width = s.get("width").and_then(|v| v.as_u64()).map(|v| v as u32);
                meta.height = s.get("height").and_then(|v| v.as_u64()).map(|v| v as u32);
                meta.pixel_format = s.get("pix_fmt").and_then(|v| v.as_str()).map(String::from);

                if let Some(ref pf) = meta.pixel_format {
                    meta.has_alpha = pixel_format_has_alpha(pf);
                }

                if let Some(r_fps) = s.get("r_frame_rate").and_then(|v| v.as_str()) {
                    meta.fps = parse_frame_rate(r_fps);
                } else if let Some(avg_fps) = s.get("avg_frame_rate").and_then(|v| v.as_str()) {
                    meta.fps = parse_frame_rate(avg_fps);
                }

                if let Some(frames) = s
                    .get("nb_frames")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<u64>().ok())
                {
                    meta.total_frames = Some(frames);
                }
            } else if codec_type == "audio" && meta.audio_codec.is_none() {
                meta.audio_codec = s
                    .get("codec_name")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                meta.sample_rate = s
                    .get("sample_rate")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<u32>().ok());
                meta.channels = s.get("channels").and_then(|v| v.as_u64()).map(|v| v as u32);
            }
        }
    }

    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_format_has_alpha() {
        assert!(pixel_format_has_alpha("yuva420p"));
        assert!(pixel_format_has_alpha("yuva444p10le"));
        assert!(pixel_format_has_alpha("rgba"));
        assert!(pixel_format_has_alpha("bgra"));

        assert!(!pixel_format_has_alpha("yuv420p"));
        assert!(!pixel_format_has_alpha("rgb24"));
        assert!(!pixel_format_has_alpha("yuv422p"));
    }

    #[test]
    fn test_parse_frame_rate() {
        assert_eq!(parse_frame_rate("30/1"), Some(30.0));
        assert_eq!(parse_frame_rate("24"), Some(24.0));
        assert!((parse_frame_rate("30000/1001").unwrap() - 29.97).abs() < 0.01);
        assert_eq!(parse_frame_rate("0/0"), None);
        assert_eq!(parse_frame_rate("invalid"), None);
    }

    #[test]
    fn test_parse_ffprobe_json() {
        let sample_json = r#"{
            "streams": [
                {
                    "codec_type": "video",
                    "codec_name": "h264",
                    "width": 1920,
                    "height": 1080,
                    "r_frame_rate": "30/1",
                    "pix_fmt": "yuv420p",
                    "nb_frames": "150"
                },
                {
                    "codec_type": "audio",
                    "codec_name": "aac",
                    "sample_rate": "48000",
                    "channels": 2
                }
            ],
            "format": {
                "duration": "5.000000"
            }
        }"#;

        let meta = parse_ffprobe_json(sample_json).unwrap();
        assert_eq!(meta.duration_seconds, Some(5.0));
        assert_eq!(meta.width, Some(1920));
        assert_eq!(meta.height, Some(1080));
        assert_eq!(meta.fps, Some(30.0));
        assert_eq!(meta.video_codec.as_deref(), Some("h264"));
        assert_eq!(meta.audio_codec.as_deref(), Some("aac"));
        assert_eq!(meta.pixel_format.as_deref(), Some("yuv420p"));
        assert!(!meta.has_alpha);
        assert_eq!(meta.sample_rate, Some(48000));
        assert_eq!(meta.channels, Some(2));
        assert_eq!(meta.total_frames, Some(150));
    }
}
