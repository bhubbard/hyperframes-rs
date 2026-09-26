//! # HyperFrames Rust (`hyperframes`)
//!
//! A high-performance programmatic video engine with seekable HTML/CSS/JS clock protocol.
//!
//! HyperFrames transforms web pages into frame-accurate, production-quality video (MP4, ProRes, WebM, GIF)
//! using headless Chrome, deterministic virtual time injection, and piped FFmpeg encoding.

pub mod cli;
pub mod core;
pub mod engine;
pub mod producer;

// Convenient top-level re-exports
pub use core::clock::{TransportClock, frame_to_time, time_to_frame, total_frames};
pub use core::protocol::{HfMediaElement, HfProtocol, HfSeekOptions, VideoCodec, VideoFormat};
pub use engine::browser::{BrowserLaunchOptions, find_chrome_executable, launch_browser};
pub use engine::session::CaptureSession;
pub use producer::pipeline::{RenderOptions, RenderStats, render_composition};
