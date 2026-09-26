use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// GSAP's own `lagSmoothing` default threshold in ms.
pub const STALL_THRESHOLD_MS: f64 = 500.0;
/// Lag adjustment subtracted when clock resumes after a stall.
pub const STALL_ADJUSTED_LAG_MS: f64 = 33.0;

/// Snapshot of the transport clock state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransportClockSnapshot {
    pub time: f64,
    pub playing: bool,
    pub rate: f64,
    pub duration: f64,
    pub source: ClockSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClockSource {
    Monotonic,
    Audio,
}

/// High-precision transport clock with stall smoothing and audio-master sync.
///
/// Ported from `@hyperframes/core/src/runtime/clock.ts`.
#[derive(Clone)]
pub struct TransportClock {
    base_time: f64,
    play_start_ms: Option<f64>,
    rate: f64,
    duration: f64,
    last_read_ms: Option<f64>,
    audio_master_time: Option<f64>,
    custom_now_fn: Option<Arc<dyn Fn() -> f64 + Send + Sync>>,
    epoch: Instant,
}

impl Default for TransportClock {
    fn default() -> Self {
        Self::new(0.0, 1.0, f64::INFINITY)
    }
}

impl std::fmt::Debug for TransportClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportClock")
            .field("base_time", &self.base_time)
            .field("play_start_ms", &self.play_start_ms)
            .field("rate", &self.rate)
            .field("duration", &self.duration)
            .field("last_read_ms", &self.last_read_ms)
            .field("audio_master_time", &self.audio_master_time)
            .finish()
    }
}

impl TransportClock {
    /// Create a new transport clock with standard high-resolution monotonic time.
    pub fn new(initial_time: f64, rate: f64, duration: f64) -> Self {
        Self {
            base_time: initial_time.max(0.0),
            play_start_ms: None,
            rate: if rate.is_finite() && rate > 0.0 {
                rate.clamp(0.1, 5.0)
            } else {
                1.0
            },
            duration: if duration.is_finite() {
                duration.max(0.0)
            } else {
                f64::INFINITY
            },
            last_read_ms: None,
            audio_master_time: None,
            custom_now_fn: None,
            epoch: Instant::now(),
        }
    }

    /// Create a transport clock driven by a custom time provider function (useful for tests).
    pub fn with_now_fn(
        initial_time: f64,
        rate: f64,
        duration: f64,
        now_fn: impl Fn() -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self {
            base_time: initial_time.max(0.0),
            play_start_ms: None,
            rate: if rate.is_finite() && rate > 0.0 {
                rate.clamp(0.1, 5.0)
            } else {
                1.0
            },
            duration: if duration.is_finite() {
                duration.max(0.0)
            } else {
                f64::INFINITY
            },
            last_read_ms: None,
            audio_master_time: None,
            custom_now_fn: Some(Arc::new(now_fn)),
            epoch: Instant::now(),
        }
    }

    fn now_ms(&self) -> f64 {
        if let Some(ref f) = self.custom_now_fn {
            f()
        } else {
            self.epoch.elapsed().as_secs_f64() * 1000.0
        }
    }

    /// Read current timeline position in seconds.
    pub fn now(&mut self) -> f64 {
        if self.play_start_ms.is_none() {
            return self.base_time;
        }

        // If audio source is actively driving the clock, audio IS time.
        if let Some(audio_t) = self.audio_master_time {
            self.last_read_ms = None;
            if self.duration.is_finite() && audio_t >= self.duration {
                return self.duration;
            }
            return audio_t.max(0.0);
        }

        // Monotonic fallback with stall smoothing
        self.apply_stall_correction();
        let start_ms = self.play_start_ms.unwrap();
        let elapsed = (self.now_ms() - start_ms) / 1000.0;
        let t = self.base_time + elapsed * self.rate;

        if self.duration.is_finite() && t >= self.duration {
            self.duration
        } else {
            t.max(0.0)
        }
    }

    fn apply_stall_correction(&mut self) {
        if self.play_start_ms.is_none() {
            return;
        }
        let now_ms = self.now_ms();
        if let Some(last_read) = self.last_read_ms {
            let gap_ms = now_ms - last_read;
            if gap_ms > STALL_THRESHOLD_MS {
                if let Some(ref mut start_ms) = self.play_start_ms {
                    *start_ms += gap_ms - STALL_ADJUSTED_LAG_MS;
                }
            }
        }
        self.last_read_ms = Some(now_ms);
    }

    /// Start or resume playback.
    pub fn play(&mut self) -> bool {
        if self.play_start_ms.is_some() {
            return false;
        }
        if self.duration.is_finite() && self.base_time >= self.duration {
            return false;
        }
        self.play_start_ms = Some(self.now_ms());
        self.last_read_ms = None;
        true
    }

    /// Pause playback.
    pub fn pause(&mut self) -> bool {
        if self.play_start_ms.is_none() {
            return false;
        }
        self.base_time = self.now();
        self.play_start_ms = None;
        true
    }

    /// Seek to a discrete timestamp in seconds.
    pub fn seek(&mut self, time_seconds: f64) {
        let clamped = if self.duration.is_finite() {
            time_seconds.clamp(0.0, self.duration)
        } else {
            time_seconds.max(0.0)
        };
        self.base_time = clamped;
        if self.play_start_ms.is_some() {
            self.play_start_ms = Some(self.now_ms());
            self.last_read_ms = None;
        }
    }

    /// Update playback rate multiplier (clamped to 0.1 .. 5.0).
    pub fn set_rate(&mut self, rate: f64) {
        let safe = if rate.is_finite() && rate > 0.0 {
            rate.clamp(0.1, 5.0)
        } else {
            1.0
        };
        if self.play_start_ms.is_some() {
            self.base_time = self.now();
            self.play_start_ms = Some(self.now_ms());
            self.last_read_ms = None;
        }
        self.rate = safe;
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }

    pub fn is_playing(&self) -> bool {
        self.play_start_ms.is_some()
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn set_audio_time(&mut self, audio_seconds: Option<f64>) {
        self.audio_master_time = audio_seconds;
    }

    pub fn snapshot(&mut self) -> TransportClockSnapshot {
        let t = self.now();
        TransportClockSnapshot {
            time: t,
            playing: self.is_playing(),
            rate: self.rate,
            duration: self.duration,
            source: if self.audio_master_time.is_some() {
                ClockSource::Audio
            } else {
                ClockSource::Monotonic
            },
        }
    }
}

/// Convert a discrete frame index into seconds for a given FPS.
#[inline]
pub fn frame_to_time(frame_index: u64, fps: f64) -> f64 {
    if fps <= 0.0 {
        return 0.0;
    }
    frame_index as f64 / fps
}

/// Convert seconds into a discrete frame index for a given FPS.
#[inline]
pub fn time_to_frame(time_seconds: f64, fps: f64) -> u64 {
    if fps <= 0.0 || time_seconds <= 0.0 {
        return 0;
    }
    (time_seconds * fps).round() as u64
}

/// Calculate the total frame count for a given duration and FPS.
#[inline]
pub fn total_frames(duration_seconds: f64, fps: f64) -> u64 {
    if duration_seconds <= 0.0 || fps <= 0.0 {
        return 0;
    }
    (duration_seconds * fps).ceil() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn test_clock_seek_and_now() {
        let mut clock = TransportClock::new(0.0, 1.0, 10.0);
        assert_eq!(clock.now(), 0.0);
        assert!(!clock.is_playing());

        clock.seek(4.5);
        assert_eq!(clock.now(), 4.5);

        // Clamp at duration
        clock.seek(15.0);
        assert_eq!(clock.now(), 10.0);

        // Clamp at 0
        clock.seek(-3.0);
        assert_eq!(clock.now(), 0.0);
    }

    #[test]
    fn test_clock_stall_smoothing() {
        let time_ms = Arc::new(AtomicU64::new(0));
        let time_clone = Arc::clone(&time_ms);

        let mut clock = TransportClock::with_now_fn(0.0, 1.0, 10.0, move || {
            time_clone.load(Ordering::SeqCst) as f64
        });

        clock.play();
        time_ms.fetch_add(16, Ordering::SeqCst);
        let before = clock.now();
        assert!((before - 0.016).abs() < 1e-4);

        // Simulate massive 4600ms thread stall
        time_ms.fetch_add(4600, Ordering::SeqCst);
        let after = clock.now();

        // Advance should be smoothed to ~33ms, not the entire 4.6 seconds
        let diff = after - before;
        assert!((diff - 0.033).abs() < 1e-3);
        assert!(diff < 4.0);

        // Next frame advances normally
        time_ms.fetch_add(16, Ordering::SeqCst);
        let next = clock.now();
        assert!((next - after - 0.016).abs() < 1e-3);
    }

    #[test]
    fn test_frame_conversions() {
        let fps = 30.0;
        assert_eq!(frame_to_time(0, fps), 0.0);
        assert_eq!(frame_to_time(30, fps), 1.0);
        assert_eq!(frame_to_time(45, fps), 1.5);

        assert_eq!(time_to_frame(1.0, fps), 30);
        assert_eq!(time_to_frame(1.5, fps), 45);

        assert_eq!(total_frames(5.0, 30.0), 150);
        assert_eq!(total_frames(5.0, 60.0), 300);
        assert_eq!(total_frames(2.5, 24.0), 60);
    }
}
