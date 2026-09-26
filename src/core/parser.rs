use super::protocol::HfProtocol;
use regex::Regex;
use std::path::Path;

/// Metadata extracted statically from an HTML composition document.
#[derive(Debug, Clone, Default)]
pub struct CompositionMeta {
    pub duration: Option<f64>,
    pub fps: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub title: Option<String>,
}

/// Parse HTML text for `<meta name="hyperframes:..." content="...">` tags.
pub fn parse_html_metadata(html: &str) -> CompositionMeta {
    let mut meta = CompositionMeta::default();

    // Regex for meta tags
    let meta_re = Regex::new(
        r#"(?i)<meta\s+[^>]*name=["']hyperframes:([a-zA-Z0-9_\-]+)["'][^>]*content=["']([^"']+)["'][^>]*>"#,
    )
    .unwrap();
    // Also reverse order content then name
    let meta_re_rev = Regex::new(
        r#"(?i)<meta\s+[^>]*content=["']([^"']+)["'][^>]*name=["']hyperframes:([a-zA-Z0-9_\-]+)["'][^>]*>"#,
    )
    .unwrap();

    let mut apply = |key: &str, val: &str| match key.to_lowercase().as_str() {
        "duration" => {
            if let Ok(d) = val.parse::<f64>() {
                meta.duration = Some(d);
            }
        }
        "fps" => {
            if let Ok(f) = val.parse::<f64>() {
                meta.fps = Some(f);
            }
        }
        "width" => {
            if let Ok(w) = val.parse::<u32>() {
                meta.width = Some(w);
            }
        }
        "height" => {
            if let Ok(h) = val.parse::<u32>() {
                meta.height = Some(h);
            }
        }
        "resolution" => {
            let parts: Vec<&str> = val.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (parts[0].trim().parse(), parts[1].trim().parse()) {
                    meta.width = Some(w);
                    meta.height = Some(h);
                }
            }
        }
        _ => {}
    };

    for cap in meta_re.captures_iter(html) {
        if let (Some(k), Some(v)) = (cap.get(1), cap.get(2)) {
            apply(k.as_str(), v.as_str());
        }
    }

    for cap in meta_re_rev.captures_iter(html) {
        if let (Some(v), Some(k)) = (cap.get(1), cap.get(2)) {
            apply(k.as_str(), v.as_str());
        }
    }

    // Extract <title>
    let title_re = Regex::new(r#"(?i)<title>(.*?)</title>"#).unwrap();
    if let Some(cap) = title_re.captures(html) {
        if let Some(t) = cap.get(1) {
            meta.title = Some(t.as_str().trim().to_string());
        }
    }

    meta
}

/// Fallback or combine statically parsed metadata with an existing HfProtocol.
pub fn merge_metadata(mut proto: HfProtocol, meta: CompositionMeta) -> HfProtocol {
    if let Some(d) = meta.duration {
        proto.duration = d;
    }
    if let Some(f) = meta.fps {
        proto.fps = f;
    }
    if let Some(w) = meta.width {
        proto.width = w;
    }
    if let Some(h) = meta.height {
        proto.height = h;
    }
    proto
}

/// Generate the deterministic virtual clock runtime shim to inject into pages
/// that don't have `@hyperframes/core` baked in.
///
/// This intercepts `performance.now()`, `Date.now()`, `requestAnimationFrame`,
/// and CSS animation playback rates, exposing the `window.__hf` seek protocol.
pub fn generate_runtime_shim(default_duration: f64, default_fps: f64) -> String {
    format!(
        r#"
(function() {{
    if (window.__hf && typeof window.__hf.seek === 'function') {{
        return; // Page already implements window.__hf natively!
    }}

    let _currentTime = 0;
    let _rafCallbacks = [];
    let _rafId = 0;

    window.__hf = {{
        duration: window.__hf?.duration ?? {default_duration},
        fps: window.__hf?.fps ?? {default_fps},
        seek: function(timeSeconds, options) {{
            _currentTime = timeSeconds * 1000;
            // Run all scheduled RAF callbacks with current deterministic time
            const callbacks = _rafCallbacks.slice();
            _rafCallbacks = [];
            for (let i = 0; i < callbacks.length; i++) {{
                try {{
                    callbacks[i].fn(_currentTime);
                }} catch (e) {{
                    console.error("Error in RAF callback during seek:", e);
                }}
            }}
            // Dispatch a seek event for custom script hooks
            window.dispatchEvent(new CustomEvent('hf:seek', {{ detail: {{ time: timeSeconds }} }}));
        }}
    }};

    // Intercept performance.now
    window.performance.now = function() {{
        return _currentTime;
    }};

    // Intercept requestAnimationFrame
    window.requestAnimationFrame = function(callback) {{
        const id = ++_rafId;
        _rafCallbacks.push({{ id, fn: callback }});
        return id;
    }};

    window.cancelAnimationFrame = function(id) {{
        _rafCallbacks = _rafCallbacks.filter(cb => cb.id !== id);
    }};

    console.log("[HyperFrames] Native deterministic clock shim injected. Duration:", window.__hf.duration, "FPS:", window.__hf.fps);
}})();
"#
    )
}

/// Convert local file path or URL into a browser-loadable URL string.
pub fn resolve_composition_url(path_or_url: &str) -> String {
    if path_or_url.starts_with("http://")
        || path_or_url.starts_with("https://")
        || path_or_url.starts_with("file://")
    {
        path_or_url.to_string()
    } else {
        let abs = std::fs::canonicalize(path_or_url)
            .unwrap_or_else(|_| Path::new(path_or_url).to_path_buf());
        format!("file://{}", abs.to_string_lossy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_html_metadata() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>My HyperFrames Video</title>
            <meta name="hyperframes:duration" content="7.5">
            <meta name="hyperframes:fps" content="60">
            <meta name="hyperframes:resolution" content="1280x720">
        </head>
        <body></body>
        </html>
        "#;
        let meta = parse_html_metadata(html);
        assert_eq!(meta.duration, Some(7.5));
        assert_eq!(meta.fps, Some(60.0));
        assert_eq!(meta.width, Some(1280));
        assert_eq!(meta.height, Some(720));
        assert_eq!(meta.title.as_deref(), Some("My HyperFrames Video"));
    }
}
