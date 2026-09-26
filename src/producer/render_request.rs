use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::fps::Fps;
use crate::core::protocol::{VideoCodec, VideoFormat};

/// Complete, serializable RenderRequest configuration.
///
/// Ported from `@hyperframes/producer/src/renderRequest.ts`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderRequest {
    #[serde(rename = "projectDir")]
    pub project_dir: String,
    #[serde(rename = "outputPath")]
    pub output_path: String,
    pub options: RenderRequestOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderRequestOptions {
    pub fps: Fps,
    #[serde(default = "default_quality")]
    pub quality: String,
    pub format: VideoFormat,
    pub codec: VideoCodec,
    #[serde(default = "default_crf")]
    pub crf: u32,
    #[serde(rename = "useGpu", default)]
    pub use_gpu: bool,
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
    #[serde(rename = "entryFile", default = "default_entry_file")]
    pub entry_file: String,
    #[serde(rename = "chunkSize", skip_serializing_if = "Option::is_none")]
    pub chunk_size: Option<u64>,
}

fn default_quality() -> String {
    "high".to_string()
}

fn default_crf() -> u32 {
    18
}

fn default_entry_file() -> String {
    "index.html".to_string()
}

impl Default for RenderRequestOptions {
    fn default() -> Self {
        Self {
            fps: Fps::default(),
            quality: default_quality(),
            format: VideoFormat::Mp4,
            codec: VideoCodec::H264,
            crf: 18,
            use_gpu: false,
            variables: HashMap::new(),
            entry_file: default_entry_file(),
            chunk_size: None,
        }
    }
}

impl RenderRequest {
    pub fn new(project_dir: impl Into<String>, output_path: impl Into<String>) -> Self {
        Self {
            project_dir: project_dir.into(),
            output_path: output_path.into(),
            options: RenderRequestOptions::default(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_request_roundtrip() {
        let mut req = RenderRequest::new("/project", "/output/video.mp4");
        req.options.fps = Fps::new(60, 1);
        req.options.format = VideoFormat::Mp4;
        req.options.codec = VideoCodec::H265;
        req.options.crf = 16;
        req.options.chunk_size = Some(120);

        let json = req.to_json().unwrap();
        let parsed = RenderRequest::from_json(&json).unwrap();
        assert_eq!(req, parsed);
    }
}
