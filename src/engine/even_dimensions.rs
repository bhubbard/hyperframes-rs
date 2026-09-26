/// Check if a pixel format requires even dimensions (e.g. 4:2:0 chroma subsampling).
pub fn requires_even_dimensions(pix_fmt: &str) -> bool {
    matches!(
        pix_fmt.to_lowercase().as_str(),
        "yuv420p" | "yuv420p10le" | "yuvj420p" | "nv12" | "nv21"
    )
}

/// Round up an integer dimension to the nearest even number.
#[inline]
pub fn even_up(dim: u32) -> u32 {
    (dim + 1) & !1
}

/// Append or construct the FFmpeg even-dimension padding filter for subsampled formats.
pub fn with_even_dimension_pad(
    existing_vf: &str,
    pix_fmt: &str,
    width: Option<u32>,
    height: Option<u32>,
) -> String {
    if !requires_even_dimensions(pix_fmt) {
        return existing_vf.to_string();
    }

    if let (Some(w), Some(h)) = (width, height) {
        if w % 2 == 0 && h % 2 == 0 {
            return existing_vf.to_string();
        }
    }

    let pad_filter = "pad=ceil(iw/2)*2:ceil(ih/2)*2";
    if existing_vf.trim().is_empty() {
        pad_filter.to_string()
    } else {
        format!("{},{}", existing_vf.trim(), pad_filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_even_dimensions() {
        assert!(requires_even_dimensions("yuv420p"));
        assert!(requires_even_dimensions("yuv420p10le"));
        assert!(requires_even_dimensions("yuvj420p"));

        // 4:4:4 or alpha formats do not require even dims
        assert!(!requires_even_dimensions("yuva444p10le"));
        assert!(!requires_even_dimensions("yuva420p"));
        assert!(!requires_even_dimensions("rgb48le"));
    }

    #[test]
    fn test_even_up() {
        assert_eq!(even_up(1080), 1080);
        assert_eq!(even_up(723), 724);
        assert_eq!(even_up(1), 2);
        assert_eq!(even_up(0), 0);
    }

    #[test]
    fn test_with_even_dimension_pad() {
        assert_eq!(
            with_even_dimension_pad("scale=in_range=pc:out_range=tv", "yuv420p", None, None),
            "scale=in_range=pc:out_range=tv,pad=ceil(iw/2)*2:ceil(ih/2)*2"
        );
        assert_eq!(
            with_even_dimension_pad("", "yuv420p", None, None),
            "pad=ceil(iw/2)*2:ceil(ih/2)*2"
        );
        assert_eq!(
            with_even_dimension_pad("", "yuv420p", Some(1920), Some(1080)),
            ""
        );
        assert_eq!(
            with_even_dimension_pad(
                "scale=in_range=pc:out_range=tv",
                "yuv420p",
                Some(1920),
                Some(1080)
            ),
            "scale=in_range=pc:out_range=tv"
        );
        assert_eq!(
            with_even_dimension_pad("", "yuv420p", Some(1921), Some(1080)),
            "pad=ceil(iw/2)*2:ceil(ih/2)*2"
        );
        assert_eq!(
            with_even_dimension_pad("", "yuv420p", Some(1920), Some(1081)),
            "pad=ceil(iw/2)*2:ceil(ih/2)*2"
        );
        assert_eq!(
            with_even_dimension_pad("scale=in_range=pc:out_range=tv", "yuva444p10le", None, None),
            "scale=in_range=pc:out_range=tv"
        );
    }
}
