pub const MAX_AUDIO_GAIN_DB: f64 = 12.0;
pub const MIN_AUDIO_GAIN_DB: f64 = -60.0;
pub const AUDIO_GAIN_FADER_MIN: f64 = -100.0;
pub const AUDIO_GAIN_FADER_MAX: f64 = 100.0;

/// Linear gain from a decibel value.
pub fn audio_db_to_gain(db: f64) -> f64 {
    10.0_f64.powf(db / 20.0)
}

/// Decibels from a linear gain value.
pub fn audio_gain_to_db(gain: f64) -> f64 {
    if gain <= 0.0 {
        f64::NEG_INFINITY
    } else {
        20.0 * gain.log10()
    }
}

pub fn max_audio_gain() -> f64 {
    audio_db_to_gain(MAX_AUDIO_GAIN_DB)
}

pub fn clamp_audio_gain(value: f64) -> f64 {
    if !value.is_finite() {
        1.0
    } else {
        value.clamp(0.0, max_audio_gain())
    }
}

/// Format gain for `data-volume` (up to 6 decimal places).
pub fn format_audio_gain(gain: f64) -> String {
    let clamped = clamp_audio_gain(gain);
    let s = format!("{:.6}", clamped);
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn audio_fader_position_to_gain(position: f64) -> f64 {
    let safe = position.clamp(AUDIO_GAIN_FADER_MIN, AUDIO_GAIN_FADER_MAX);
    if safe == AUDIO_GAIN_FADER_MIN {
        return 0.0;
    }
    let db = if safe < 0.0 {
        (safe / AUDIO_GAIN_FADER_MIN.abs()) * MIN_AUDIO_GAIN_DB.abs()
    } else {
        (safe / AUDIO_GAIN_FADER_MAX) * MAX_AUDIO_GAIN_DB
    };
    audio_db_to_gain(db)
}

pub fn audio_gain_to_fader_position(gain: f64) -> f64 {
    if gain <= 0.0001 {
        return AUDIO_GAIN_FADER_MIN;
    }
    let db = audio_gain_to_db(gain);
    if db.abs() < 1e-6 {
        0.0
    } else if db < 0.0 {
        (db / MIN_AUDIO_GAIN_DB.abs()) * AUDIO_GAIN_FADER_MIN.abs()
    } else {
        (db / MAX_AUDIO_GAIN_DB) * AUDIO_GAIN_FADER_MAX
    }
}

pub fn audio_gain_to_text(gain: f64) -> String {
    if gain <= 0.0001 {
        "-∞ dB".to_string()
    } else {
        let db = audio_gain_to_db(gain);
        if db >= 0.0 {
            format!("+{:.1} dB", db)
        } else {
            format!("{:.1} dB", db)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_db_and_gain_conversions() {
        assert_eq!(audio_db_to_gain(0.0), 1.0);
        let g_minus_6 = audio_db_to_gain(-6.0);
        assert!((g_minus_6 - 10.0_f64.powf(-6.0 / 20.0)).abs() < 1e-10);
        assert!((max_audio_gain() - audio_db_to_gain(12.0)).abs() < 1e-10);
        assert!((audio_gain_to_db(1.0) - 0.0).abs() < 1e-10);
        assert!((audio_gain_to_db(g_minus_6) - (-6.0)).abs() < 1e-10);
    }

    #[test]
    fn test_unity_gain_fader_midpoint() {
        assert!((audio_gain_to_fader_position(1.0) - 0.0).abs() < 1e-6);
        assert!((audio_fader_position_to_gain(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fader_boost_and_silence() {
        assert!(
            (audio_fader_position_to_gain(AUDIO_GAIN_FADER_MAX) - max_audio_gain()).abs() < 1e-5
        );
        assert_eq!(audio_gain_to_text(max_audio_gain()), "+12.0 dB");
        assert_eq!(audio_fader_position_to_gain(AUDIO_GAIN_FADER_MIN), 0.0);
        assert_eq!(audio_gain_to_text(0.0), "-∞ dB");
    }

    #[test]
    fn test_format_audio_gain() {
        assert_eq!(format_audio_gain(1.0), "1");
        assert_eq!(format_audio_gain(0.5), "0.5");
        assert_eq!(format_audio_gain(0.0), "0");
    }
}
