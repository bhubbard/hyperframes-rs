use serde::{Deserialize, Serialize};

/// Frame rate as an exact rational `{ num, den }`.
///
/// Carrying `{ num, den }` end-to-end (rather than collapsing to a float)
/// lets us pass NTSC / drop-frame rates straight through to FFmpeg via
/// `-r 30000/1001` without decimal precision loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fps {
    pub num: u32,
    pub den: u32,
}

impl Fps {
    pub const NTSC_24: Self = Self {
        num: 24000,
        den: 1001,
    };
    pub const NTSC_30: Self = Self {
        num: 30000,
        den: 1001,
    };
    pub const NTSC_60: Self = Self {
        num: 60000,
        den: 1001,
    };

    pub fn new(num: u32, den: u32) -> Self {
        Self {
            num,
            den: den.max(1),
        }
    }

    pub fn from_integer(n: u32) -> Self {
        Self { num: n, den: 1 }
    }

    pub fn as_f64(&self) -> f64 {
        if self.den == 0 {
            0.0
        } else {
            self.num as f64 / self.den as f64
        }
    }

    pub fn to_ffmpeg_arg(&self) -> String {
        if self.den == 1 {
            self.num.to_string()
        } else {
            format!("{}/{}", self.num, self.den)
        }
    }
}

impl Default for Fps {
    fn default() -> Self {
        Self { num: 30, den: 1 }
    }
}

impl From<u32> for Fps {
    fn from(n: u32) -> Self {
        Self::from_integer(n)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FpsParseError {
    Empty,
    NotANumber,
    NonPositive,
    OutOfRange,
    InvalidFraction,
    AmbiguousDecimal,
}

/// Parse user-supplied FPS input (e.g. "30", "30000/1001", "24000/1001").
pub fn parse_fps(input: &str) -> Result<Fps, FpsParseError> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err(FpsParseError::Empty);
    }

    if raw.contains('/') {
        let parts: Vec<&str> = raw.split('/').collect();
        if parts.len() != 2 {
            return Err(FpsParseError::InvalidFraction);
        }
        let num_str = parts[0].trim();
        let den_str = parts[1].trim();

        // Check if negative or non-integer
        if num_str.starts_with('-') || den_str.starts_with('-') {
            if den_str.starts_with('-') {
                return Err(FpsParseError::InvalidFraction);
            }
            return Err(FpsParseError::NonPositive);
        }

        let num: u32 = num_str
            .parse()
            .map_err(|_| FpsParseError::InvalidFraction)?;
        let den: u32 = den_str
            .parse()
            .map_err(|_| FpsParseError::InvalidFraction)?;

        if den == 0 {
            return Err(FpsParseError::InvalidFraction);
        }
        if num == 0 {
            return Err(FpsParseError::NonPositive);
        }

        let decimal = num as f64 / den as f64;
        if !(1.0..=240.0).contains(&decimal) {
            return Err(FpsParseError::OutOfRange);
        }

        return Ok(Fps { num, den });
    }

    // Check for decimal like "29.97"
    if raw.contains('.') {
        return Err(FpsParseError::AmbiguousDecimal);
    }

    let n: i64 = raw.parse().map_err(|_| FpsParseError::NotANumber)?;
    if n <= 0 {
        return Err(FpsParseError::NonPositive);
    }
    if n > 240 {
        return Err(FpsParseError::OutOfRange);
    }

    Ok(Fps {
        num: n as u32,
        den: 1,
    })
}

pub fn parse_fps_with_default(input: Option<&str>) -> Result<Fps, FpsParseError> {
    match input {
        None => Ok(Fps::default()),
        Some(s) if s.trim().is_empty() => Ok(Fps::default()),
        Some(s) => parse_fps(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fps_integers() {
        assert_eq!(parse_fps("30"), Ok(Fps { num: 30, den: 1 }));
        assert_eq!(parse_fps("  30  "), Ok(Fps { num: 30, den: 1 }));
        for n in [24, 60, 120, 240] {
            assert_eq!(parse_fps(&n.to_string()), Ok(Fps { num: n, den: 1 }));
        }
    }

    #[test]
    fn test_parse_fps_rationals() {
        assert_eq!(
            parse_fps("30000/1001"),
            Ok(Fps {
                num: 30000,
                den: 1001
            })
        );
        assert_eq!(
            parse_fps("24000/1001"),
            Ok(Fps {
                num: 24000,
                den: 1001
            })
        );
        assert_eq!(
            parse_fps("60000/1001"),
            Ok(Fps {
                num: 60000,
                den: 1001
            })
        );
        assert_eq!(parse_fps("60/2"), Ok(Fps { num: 60, den: 2 }));
    }

    #[test]
    fn test_parse_fps_rejected() {
        assert_eq!(parse_fps("abc"), Err(FpsParseError::NotANumber));
        assert_eq!(parse_fps("30/0"), Err(FpsParseError::InvalidFraction));
        assert_eq!(parse_fps("30/-1"), Err(FpsParseError::InvalidFraction));
        assert_eq!(parse_fps("0"), Err(FpsParseError::NonPositive));
        assert_eq!(parse_fps("241"), Err(FpsParseError::OutOfRange));
        assert_eq!(parse_fps("29.97"), Err(FpsParseError::AmbiguousDecimal));
        assert_eq!(parse_fps(""), Err(FpsParseError::Empty));
    }

    #[test]
    fn test_fps_to_ffmpeg_arg() {
        assert_eq!(Fps::from_integer(30).to_ffmpeg_arg(), "30");
        assert_eq!(Fps::NTSC_30.to_ffmpeg_arg(), "30000/1001");
    }
}
