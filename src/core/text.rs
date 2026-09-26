/// Options for fitting text within a max width.
#[derive(Debug, Clone, PartialEq)]
pub struct FitTextOptions {
    pub base_font_size: u32,
    pub min_font_size: u32,
    pub step: u32,
    pub font_family: String,
    pub font_weight: u32,
    pub max_width: f64,
}

impl Default for FitTextOptions {
    fn default() -> Self {
        Self {
            base_font_size: 78,
            min_font_size: 42,
            step: 2,
            font_family: "sans-serif".to_string(),
            font_weight: 400,
            max_width: 1200.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FitTextResult {
    pub font_size: u32,
    pub fits: bool,
}

/// Approximate text width estimation using average character metrics.
pub fn estimate_text_width(text: &str, font_size: u32) -> f64 {
    // Average proportional font aspect ratio is roughly 0.55 width/height
    text.chars().count() as f64 * (font_size as f64 * 0.55)
}

pub type TextMeasureFn<'a> = &'a dyn Fn(&str, u32) -> f64;

/// Calculate the optimal font size to fit text within `max_width`.
pub fn fit_text_font_size(
    text: &str,
    options: Option<FitTextOptions>,
    measure_fn: Option<TextMeasureFn>,
) -> FitTextResult {
    let opts = options.unwrap_or_default();
    let mut current_size = opts.base_font_size;

    while current_size >= opts.min_font_size {
        let width = if let Some(measure) = measure_fn {
            measure(text, current_size)
        } else {
            estimate_text_width(text, current_size)
        };

        if width <= opts.max_width {
            return FitTextResult {
                font_size: current_size,
                fits: true,
            };
        }

        if current_size <= opts.min_font_size + opts.step {
            break;
        }
        current_size -= opts.step;
    }

    FitTextResult {
        font_size: opts.min_font_size,
        fits: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_text_fits_at_base() {
        let res = fit_text_font_size(
            "Short",
            Some(FitTextOptions {
                base_font_size: 60,
                min_font_size: 30,
                step: 5,
                max_width: 500.0,
                ..Default::default()
            }),
            Some(&|_txt, size| (size as f64) * 3.0), // 60 * 3 = 180 <= 500
        );
        assert_eq!(
            res,
            FitTextResult {
                font_size: 60,
                fits: true
            }
        );
    }

    #[test]
    fn test_fit_text_shrinks_to_fit() {
        // Mock measure function: width = size * 10
        // At 80: 800 > 600
        // At 70: 700 > 600
        // At 60: 600 <= 600 (fits!)
        let res = fit_text_font_size(
            "Header Text",
            Some(FitTextOptions {
                base_font_size: 80,
                min_font_size: 40,
                step: 10,
                max_width: 600.0,
                ..Default::default()
            }),
            Some(&|_txt, size| size as f64 * 10.0),
        );
        assert_eq!(
            res,
            FitTextResult {
                font_size: 60,
                fits: true
            }
        );
    }

    #[test]
    fn test_fit_text_min_size_fallback() {
        let res = fit_text_font_size(
            "Massive Text",
            Some(FitTextOptions {
                base_font_size: 80,
                min_font_size: 40,
                step: 10,
                max_width: 100.0,
                ..Default::default()
            }),
            Some(&|_txt, size| size as f64 * 10.0), // 40 * 10 = 400 > 100
        );
        assert_eq!(
            res,
            FitTextResult {
                font_size: 40,
                fits: false
            }
        );
    }
}
