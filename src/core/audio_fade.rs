/// Fade configuration for audio tracks.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AudioFades {
    pub fade_in: f64,
    pub fade_out: f64,
}

impl AudioFades {
    pub fn new(fade_in: f64, fade_out: f64) -> Self {
        Self {
            fade_in: fade_in.max(0.0),
            fade_out: fade_out.max(0.0),
        }
    }
}

pub fn read_fade_seconds(raw: Option<&str>) -> f64 {
    let Some(s) = raw else {
        return 0.0;
    };
    match s.trim().parse::<f64>() {
        Ok(v) if v.is_finite() && v > 0.0 => v,
        _ => 0.0,
    }
}

pub fn clamp_fades_to_duration(fades: AudioFades, clip_duration: f64) -> AudioFades {
    if !clip_duration.is_finite() || clip_duration <= 0.0 {
        return fades;
    }
    let total = fades.fade_in + fades.fade_out;
    if total <= clip_duration {
        return fades;
    }
    // Scale both proportionally so they meet exactly inside the clip
    let scale = clip_duration / total;
    AudioFades {
        fade_in: fades.fade_in * scale,
        fade_out: fades.fade_out * scale,
    }
}

/// Instantaneous fade gain multiplier (0.0 to 1.0) at `elapsed_seconds` into a clip.
pub fn fade_gain(elapsed_seconds: f64, clip_duration: f64, fades: AudioFades) -> f64 {
    if !elapsed_seconds.is_finite() {
        return 1.0;
    }
    if fades.fade_in <= 0.0 && fades.fade_out <= 0.0 {
        return 1.0;
    }

    let clamped_fades = clamp_fades_to_duration(fades, clip_duration);
    let mut in_gain = 1.0;
    let mut out_gain = 1.0;

    // Fade-in curve
    if clamped_fades.fade_in > 0.0 {
        if elapsed_seconds <= 0.0 {
            in_gain = 0.0;
        } else if elapsed_seconds < clamped_fades.fade_in {
            in_gain = elapsed_seconds / clamped_fades.fade_in;
        }
    }

    // Fade-out curve
    if clamped_fades.fade_out > 0.0 && clip_duration.is_finite() {
        let remaining = clip_duration - elapsed_seconds;
        if remaining <= 0.0 {
            out_gain = 0.0;
        } else if remaining < clamped_fades.fade_out {
            out_gain = remaining / clamped_fades.fade_out;
        }
    }

    (in_gain * out_gain).clamp(0.0, 1.0)
}

pub fn format_fade_seconds(seconds: f64) -> String {
    if !seconds.is_finite() || seconds <= 0.0 {
        return "0".to_string();
    }
    let s = format!("{:.2}", seconds);
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_fade_seconds() {
        assert_eq!(read_fade_seconds(Some("0.5")), 0.5);
        assert_eq!(read_fade_seconds(Some("2")), 2.0);
        assert_eq!(read_fade_seconds(None), 0.0);
        assert_eq!(read_fade_seconds(Some("")), 0.0);
        assert_eq!(read_fade_seconds(Some("-1")), 0.0);
        assert_eq!(read_fade_seconds(Some("NaN")), 0.0);
        assert_eq!(read_fade_seconds(Some("abc")), 0.0);
    }

    #[test]
    fn test_clamp_fades_to_duration() {
        assert_eq!(
            clamp_fades_to_duration(AudioFades::new(1.0, 2.0), 10.0),
            AudioFades::new(1.0, 2.0)
        );
        assert_eq!(
            clamp_fades_to_duration(AudioFades::new(6.0, 6.0), 6.0),
            AudioFades::new(3.0, 3.0)
        );
        assert_eq!(
            clamp_fades_to_duration(AudioFades::new(3.0, 1.0), 2.0),
            AudioFades::new(1.5, 0.5)
        );
        assert_eq!(
            clamp_fades_to_duration(AudioFades::new(4.0, 4.0), f64::INFINITY),
            AudioFades::new(4.0, 4.0)
        );
    }

    #[test]
    fn test_fade_gain() {
        let fades = AudioFades::new(2.0, 1.0);
        assert_eq!(fade_gain(0.0, 10.0, AudioFades::default()), 1.0);
        assert_eq!(fade_gain(10.0, 10.0, AudioFades::default()), 1.0);

        // Fade in
        assert_eq!(fade_gain(0.0, 10.0, fades), 0.0);
        assert_eq!(fade_gain(1.0, 10.0, fades), 0.5);
        assert_eq!(fade_gain(2.0, 10.0, fades), 1.0);
        assert_eq!(fade_gain(5.0, 10.0, fades), 1.0);

        // Fade out
        assert_eq!(fade_gain(9.0, 10.0, fades), 1.0);
        assert_eq!(fade_gain(9.5, 10.0, fades), 0.5);
        assert_eq!(fade_gain(10.0, 10.0, fades), 0.0);
        assert_eq!(fade_gain(11.0, 10.0, fades), 0.0);
    }

    #[test]
    fn test_format_fade_seconds() {
        assert_eq!(format_fade_seconds(0.0), "0");
        assert_eq!(format_fade_seconds(1.0), "1");
        assert_eq!(format_fade_seconds(0.5), "0.5");
        assert_eq!(format_fade_seconds(0.333), "0.33");
        assert_eq!(format_fade_seconds(-2.0), "0");
    }
}
