pub mod browser;
pub mod capture;
pub mod session;

pub use browser::{BrowserLaunchOptions, find_chrome_executable, launch_browser};
pub use capture::{CapturedFrame, FrameExtractionOptions, stream_frames};
pub use session::CaptureSession;
