use anyhow::Result;
use tokio::sync::mpsc;
use tracing::debug;

use super::session::CaptureSession;
use crate::core::clock::{frame_to_time, total_frames};

/// Captured video frame buffer.
#[derive(Debug)]
pub struct CapturedFrame {
    pub frame_index: u64,
    pub time_seconds: f64,
    pub data: Vec<u8>,
}

/// Options controlling frame extraction.
#[derive(Debug, Clone, Default)]
pub struct FrameExtractionOptions {
    pub start_frame: u64,
    pub end_frame: Option<u64>,
    pub settle_ms: u64,
}

/// Extract frames sequentially from the capture session and stream them over an async channel.
pub async fn stream_frames(
    session: &CaptureSession,
    options: FrameExtractionOptions,
    tx: mpsc::Sender<CapturedFrame>,
) -> Result<u64> {
    let proto = session.protocol();
    let max_frames = total_frames(proto.duration, proto.fps);
    let end = options.end_frame.unwrap_or(max_frames).min(max_frames);
    let mut frames_captured = 0;

    for frame_idx in options.start_frame..end {
        let t = frame_to_time(frame_idx, proto.fps);
        debug!("Capturing frame {}/{} at t={:.3}s", frame_idx + 1, end, t);

        session.seek(t, None).await?;

        if options.settle_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(options.settle_ms)).await;
        }

        let bytes = session.capture_frame_png().await?;

        let frame = CapturedFrame {
            frame_index: frame_idx,
            time_seconds: t,
            data: bytes,
        };

        if tx.send(frame).await.is_err() {
            // Receiver closed pipe early
            break;
        }

        frames_captured += 1;
    }

    Ok(frames_captured)
}
