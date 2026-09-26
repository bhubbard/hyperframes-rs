pub mod browser;
pub mod capture;
pub mod even_dimensions;
pub mod ffprobe;
pub mod session;

pub use browser::{BrowserLaunchOptions, find_chrome_executable, launch_browser};
pub use capture::{CapturedFrame, FrameExtractionOptions, stream_frames};
pub use even_dimensions::{even_up, requires_even_dimensions, with_even_dimension_pad};
pub use ffprobe::{
    MediaMetadata, extract_media_metadata, parse_ffprobe_json, parse_frame_rate,
    pixel_format_has_alpha,
};
pub use session::CaptureSession;
