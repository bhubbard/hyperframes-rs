use serde::{Deserialize, Serialize};

/// The seek protocol contract implemented by compositions (`window.__hf`).
///
/// Any web page that wants to be rendered deterministically as video
/// implements this protocol so the headless engine can advance time
/// frame-by-frame without clock drift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfProtocol {
    /// Total duration of the composition in seconds.
    pub duration: f64,
    /// Target frame rate (defaults to 30.0 if not specified).
    #[serde(default = "default_fps")]
    pub fps: f64,
    /// Frame width in pixels.
    #[serde(default = "default_width")]
    pub width: u32,
    /// Frame height in pixels.
    #[serde(default = "default_height")]
    pub height: u32,
    /// Optional media elements that the engine must extract and mix.
    #[serde(default)]
    pub media: Vec<HfMediaElement>,
    /// Optional shader transition metadata.
    #[serde(default)]
    pub transitions: Vec<HfTransitionMeta>,
}

fn default_fps() -> f64 {
    30.0
}

fn default_width() -> u32 {
    1920
}

fn default_height() -> u32 {
    1080
}

impl Default for HfProtocol {
    fn default() -> Self {
        Self {
            duration: 5.0,
            fps: 30.0,
            width: 1920,
            height: 1080,
            media: Vec::new(),
            transitions: Vec::new(),
        }
    }
}

/// Declares a `<video>` or `<audio>` media element the engine should handle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HfMediaElement {
    /// DOM id of the media element.
    #[serde(rename = "elementId")]
    pub element_id: String,
    /// Source file path or URL.
    pub src: String,
    /// Start time in the composition in seconds.
    #[serde(rename = "startTime")]
    pub start_time: f64,
    /// End time in the composition in seconds.
    #[serde(rename = "endTime")]
    pub end_time: f64,
    /// Offset into the media source in seconds (default: 0.0).
    #[serde(rename = "mediaOffset", default)]
    pub media_offset: f64,
    /// Volume multiplier from 0.0 to 1.0 (default: 1.0).
    #[serde(default = "default_volume")]
    pub volume: f64,
    /// Whether this element contains an audio track that should be mixed.
    #[serde(rename = "hasAudio", default = "default_has_audio")]
    pub has_audio: bool,
}

fn default_volume() -> f64 {
    1.0
}

fn default_has_audio() -> bool {
    true
}

/// Metadata for a shader transition between scenes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HfTransitionMeta {
    /// Timestamp in seconds when transition begins.
    pub time: f64,
    /// Duration of transition in seconds.
    pub duration: f64,
    /// Shader identifier (or CSS crossfade if empty/none).
    pub shader: Option<String>,
    /// Easing curve (e.g. "power2.inOut", "linear").
    #[serde(default = "default_ease")]
    pub ease: String,
    /// Scene id the transition starts from.
    #[serde(rename = "fromScene")]
    pub from_scene: String,
    /// Scene id the transition transitions into.
    #[serde(rename = "toScene")]
    pub to_scene: String,
}

fn default_ease() -> String {
    "power2.inOut".to_string()
}

/// Seek options passed into `window.__hf.seek(time, options)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct HfSeekOptions {
    /// Suppress firing user event callbacks during discrete frame seek.
    #[serde(rename = "suppressEvents", default)]
    pub suppress_events: bool,
    /// Sub-frame sampling divisions for motion blur.
    #[serde(rename = "subFrameDivisions", default)]
    pub sub_frame_divisions: Option<u32>,
}

/// Video output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VideoFormat {
    #[default]
    #[serde(rename = "mp4")]
    Mp4,
    #[serde(rename = "mov")]
    Mov,
    #[serde(rename = "webm")]
    Webm,
    #[serde(rename = "gif")]
    Gif,
    #[serde(rename = "png-sequence")]
    PngSequence,
}

impl std::str::FromStr for VideoFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mp4" => Ok(VideoFormat::Mp4),
            "mov" => Ok(VideoFormat::Mov),
            "webm" => Ok(VideoFormat::Webm),
            "gif" => Ok(VideoFormat::Gif),
            "png" | "png-sequence" => Ok(VideoFormat::PngSequence),
            other => Err(format!("Unknown video format: {other}")),
        }
    }
}

/// Video codec selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VideoCodec {
    #[default]
    #[serde(rename = "h264")]
    H264,
    #[serde(rename = "h265")]
    H265,
    #[serde(rename = "prores")]
    ProRes,
    #[serde(rename = "vp9")]
    Vp9,
}

impl std::str::FromStr for VideoCodec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "h264" | "libx264" => Ok(VideoCodec::H264),
            "h265" | "hevc" | "libx265" => Ok(VideoCodec::H265),
            "prores" | "prores_ks" => Ok(VideoCodec::ProRes),
            "vp9" | "libvpx-vp9" => Ok(VideoCodec::Vp9),
            other => Err(format!("Unknown video codec: {other}")),
        }
    }
}
