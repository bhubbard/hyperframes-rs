use anyhow::{Result, anyhow};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::Page;
use serde_json::Value;
use std::time::Duration;
use tracing::info;

use crate::core::parser::generate_runtime_shim;
use crate::core::protocol::{HfProtocol, HfSeekOptions};

/// An active rendering session attached to a browser page.
pub struct CaptureSession {
    page: Page,
    protocol: HfProtocol,
}

impl std::fmt::Debug for CaptureSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaptureSession")
            .field("protocol", &self.protocol)
            .finish()
    }
}

impl CaptureSession {
    /// Initialize a new capture session by navigating to the composition URL.
    pub async fn new(page: Page, url: &str, default_proto: HfProtocol) -> Result<Self> {
        info!("Navigating page to: {}", url);
        page.goto(url).await?;

        // Wait for DOM content loaded
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Inject runtime clock shim if the page doesn't have native window.__hf
        let shim = generate_runtime_shim(default_proto.duration, default_proto.fps);
        page.evaluate(shim).await?;

        // Read window.__hf configuration from the page
        let eval_res: Value = page
            .evaluate("JSON.stringify(window.__hf || {})")
            .await?
            .into_value()?;

        let mut final_proto = default_proto;

        if let Value::String(s) = eval_res {
            if let Ok(v) = serde_json::from_str::<Value>(&s) {
                if let Some(d) = v.get("duration").and_then(|v| v.as_f64()) {
                    if d > 0.0 {
                        final_proto.duration = d;
                    }
                }
                if let Some(f) = v.get("fps").and_then(|v| v.as_f64()) {
                    if f > 0.0 {
                        final_proto.fps = f;
                    }
                }
                if let Some(w) = v.get("width").and_then(|v| v.as_u64()) {
                    if w > 0 {
                        final_proto.width = w as u32;
                    }
                }
                if let Some(h) = v.get("height").and_then(|v| v.as_u64()) {
                    if h > 0 {
                        final_proto.height = h as u32;
                    }
                }
            }
        }

        info!(
            "Session initialized: duration={:.2}s, fps={:.1}, resolution={}x{}",
            final_proto.duration, final_proto.fps, final_proto.width, final_proto.height
        );

        Ok(Self {
            page,
            protocol: final_proto,
        })
    }

    /// Access the resolved protocol metadata.
    pub fn protocol(&self) -> &HfProtocol {
        &self.protocol
    }

    /// Explicitly override metadata settings (from CLI flags or programmatic options).
    pub fn override_settings(
        &mut self,
        width: Option<u32>,
        height: Option<u32>,
        fps: Option<f64>,
        duration: Option<f64>,
    ) {
        if let Some(w) = width {
            self.protocol.width = w;
        }
        if let Some(h) = height {
            self.protocol.height = h;
        }
        if let Some(f) = fps {
            self.protocol.fps = f;
        }
        if let Some(d) = duration {
            self.protocol.duration = d;
        }
    }

    /// Seek the composition to a specific timestamp in seconds.
    pub async fn seek(&self, time_seconds: f64, options: Option<HfSeekOptions>) -> Result<()> {
        let opts_json = match options {
            Some(opts) => serde_json::to_string(&opts)?,
            None => "{}".to_string(),
        };

        let script = format!(
            r#"
            if (window.__hf && typeof window.__hf.seek === 'function') {{
                window.__hf.seek({}, {});
            }}
            "#,
            time_seconds, opts_json
        );

        self.page.evaluate(script).await?;
        Ok(())
    }

    /// Capture the current frame as raw image bytes (PNG format).
    pub async fn capture_frame_png(&self) -> Result<Vec<u8>> {
        let screenshot_params = chromiumoxide::page::ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(false)
            .build();

        let bytes = self
            .page
            .screenshot(screenshot_params)
            .await
            .map_err(|e| anyhow!("Failed to capture frame screenshot: {e}"))?;

        Ok(bytes)
    }

    /// Capture the current frame as JPEG bytes with a specific quality (1-100).
    pub async fn capture_frame_jpeg(&self, quality: i64) -> Result<Vec<u8>> {
        let screenshot_params = chromiumoxide::page::ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Jpeg)
            .quality(quality)
            .full_page(false)
            .build();

        let bytes = self
            .page
            .screenshot(screenshot_params)
            .await
            .map_err(|e| anyhow!("Failed to capture frame screenshot: {e}"))?;

        Ok(bytes)
    }
}
