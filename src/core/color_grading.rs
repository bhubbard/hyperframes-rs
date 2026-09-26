use serde::{Deserialize, Serialize};

/// Standard color space used for HyperFrames grading.
pub const HF_COLOR_GRADING_COLOR_SPACE: &str = "srgb";

/// Color adjustment parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorAdjustments {
    /// Exposure adjustment (-2.0 .. +2.0 stops).
    #[serde(default)]
    pub exposure: f64,
    /// Contrast adjustment (-1.0 .. +1.0).
    #[serde(default)]
    pub contrast: f64,
    /// Saturation multiplier (-1.0 to +1.0, -1.0 is grayscale).
    #[serde(default)]
    pub saturation: f64,
    /// Vibrance multiplier (-1.0 .. +1.0).
    #[serde(default)]
    pub vibrance: f64,
    /// Color temperature (-1.0 warm .. +1.0 cool).
    #[serde(default)]
    pub temperature: f64,
    /// Color tint (-1.0 green .. +1.0 magenta).
    #[serde(default)]
    pub tint: f64,
}

impl Default for ColorAdjustments {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            contrast: 0.0,
            saturation: 0.0,
            vibrance: 0.0,
            temperature: 0.0,
            tint: 0.0,
        }
    }
}

/// Color grading preset definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorGradingPreset {
    pub id: String,
    pub name: String,
    pub adjust: ColorAdjustments,
    pub vignette: f64,
}

impl ColorGradingPreset {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            adjust: ColorAdjustments::default(),
            vignette: 0.0,
        }
    }
}

/// Get standard color grading presets.
pub fn standard_presets() -> Vec<ColorGradingPreset> {
    vec![
        ColorGradingPreset {
            id: "neutral".to_string(),
            name: "Neutral".to_string(),
            adjust: ColorAdjustments::default(),
            vignette: 0.0,
        },
        ColorGradingPreset {
            id: "warm-daylight".to_string(),
            name: "Warm Daylight".to_string(),
            adjust: ColorAdjustments {
                temperature: 0.25,
                saturation: 0.1,
                contrast: 0.05,
                ..Default::default()
            },
            vignette: 0.0,
        },
        ColorGradingPreset {
            id: "mono-clean".to_string(),
            name: "Monochrome Clean".to_string(),
            adjust: ColorAdjustments {
                saturation: -1.0,
                contrast: 0.15,
                ..Default::default()
            },
            vignette: 0.1,
        },
        ColorGradingPreset {
            id: "vintage-wash".to_string(),
            name: "Vintage Wash".to_string(),
            adjust: ColorAdjustments {
                exposure: 0.1,
                contrast: -0.1,
                temperature: 0.2,
                ..Default::default()
            },
            vignette: 0.35,
        },
        ColorGradingPreset {
            id: "bright-pop".to_string(),
            name: "Bright Pop".to_string(),
            adjust: ColorAdjustments {
                exposure: 0.15,
                contrast: 0.1,
                saturation: 0.25,
                vibrance: 0.2,
                ..Default::default()
            },
            vignette: 0.05,
        },
    ]
}

/// Resolve preset by identifier.
pub fn find_preset(id: &str) -> Option<ColorGradingPreset> {
    standard_presets().into_iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_exist() {
        let presets = standard_presets();
        assert!(presets.iter().any(|p| p.id == "neutral"));
        assert!(presets.iter().any(|p| p.id == "warm-daylight"));
        assert!(presets.iter().any(|p| p.id == "mono-clean"));
    }

    #[test]
    fn test_mono_clean_is_grayscale() {
        let preset = find_preset("mono-clean").unwrap();
        assert_eq!(preset.adjust.saturation, -1.0);
    }

    #[test]
    fn test_warm_daylight_adjustments() {
        let preset = find_preset("warm-daylight").unwrap();
        assert!(preset.adjust.temperature > 0.0);
        assert!(preset.adjust.saturation > 0.0);
    }
}
