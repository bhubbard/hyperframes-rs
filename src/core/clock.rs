use serde::{Deserialize, Serialize};
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
#[derive(Debug)]
pub struct TransportClock {
    base_time: f64,
    play_start_instant: Option<Instant>,
    rate: f64,
    duration: f64,
    last_read_instant: Option<Instant>,
    audio_master_time: Option<f64>,
}

impl Default for TransportClock {
    fn default() -> Self {
        Self::new(0.0, 1.0, f64::INFINITY)
    }
}

impl TransportClock {
    /// Create a new transport clock.
    pub fn new(initial_time: f64, rate: f64, duration: f64) -> Self {
        Self {
            base_time: initial_time.max(0.0),
            play_start_instant: None,
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
            last_read_instant: None,
            audio_master_time: None,
        }
    }

    /// Read the current timeline position in seconds.
    pub fn now(&mut self) -> f64 {
        // If an audio-master source is actively driving the clock, audio IS time.
        if let Some(audio_t) = self.audio_master_time {
            self.last_read_instant = None;
            if self.duration.is_finite() && audio_t >= self.duration {
                return self.duration;
            }
            return audio_t.max(0.0);
        }

        let Some(start_inst) = self.play_start_instant else {
            return self.base_time;
        };

        let now_inst = Instant::now();

        // Apply stall correction for lag > STALL_THRESHOLD_MS
        if let Some(last_read) = self.last_read_instant {
            let gap_ms = now_inst.duration_since(last_read).as_secs_f64() * 1000.0;
            if gap_ms > STALL_THRESHOLD_MS {
                // Fold the gap into play_start_instant
                let lag_correction =
                    std::time::Duration::from_secs_f64((gap_ms - STALL_ADJUSTED_LAG_MS) / 1000.0);
                self.play_start_instant = Some(start_inst + lag_correction);
            }
        }
        self.last_read_instant = Some(now_inst);

        let elapsed_secs = now_inst
            .duration_since(self.play_start_instant.unwrap())
            .as_secs_f64();
        let t = self.base_time + elapsed_secs * self.rate;

        if self.duration.is_finite() && t >= self.duration {
            self.duration
        } else {
            t.max(0.0)
        }
    }

    /// Start or resume playback.
    pub fn play(&mut self) -> bool {
        if self.play_start_instant.is_some() {
            return false;
        }
        if self.duration.is_finite() && self.base_time >= self.duration {
            return false;
        }
        self.play_start_instant = Some(Instant::now());
        self.last_read_instant = None;
        true
    }

    /// Pause playback.
    pub fn pause(&mut self) -> bool {
        if self.play_start_instant.is_none() {
            return false;
        }
        self.base_time = self.now();
        self.play_start_instant = None;
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
        if self.play_start_instant.is_some() {
            self.play_start_instant = Some(Instant::now());
            self.last_read_instant = None;
        }
    }

    /// Update playback rate multiplier (clamped to 0.1 .. 5.0).
    pub fn set_rate(&mut self, rate: f64) {
        let safe = if rate.is_finite() && rate > 0.0 {
            rate.clamp(0.1, 5.0)
        } else {
            1.0
        };
        if self.play_start_instant.is_some() {
            self.base_time = self.now();
            self.play_start_instant = Some(Instant::now());
            self.last_read_instant = None;
        }
        self.rate = safe;
    }

    /// Current playback rate.
    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Whether the clock is actively running.
    pub fn is_playing(&self) -> bool {
        self.play_start_instant.is_some()
    }

    /// Total duration in seconds.
    pub fn duration(&self) -> f64 {
        self.duration
    }

    /// Set an audio-master override timestamp.
    pub fn set_audio_time(&mut self, audio_seconds: Option<f64>) {
        self.audio_master_time = audio_seconds;
    }

    /// Snapshot the current clock state.
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
