use anyhow::{Result, anyhow};
use chromiumoxide::{Browser, BrowserConfig};
use std::path::{Path, PathBuf};
use tracing::info;

/// Options for launching the headless browser.
#[derive(Debug, Clone)]
pub struct BrowserLaunchOptions {
    pub chrome_path: Option<PathBuf>,
    pub width: u32,
    pub height: u32,
    pub enable_gpu: bool,
    pub headless: bool,
}

impl Default for BrowserLaunchOptions {
    fn default() -> Self {
        Self {
            chrome_path: None,
            width: 1920,
            height: 1080,
            enable_gpu: false,
            headless: true,
        }
    }
}

/// Detect the Google Chrome / Chromium executable path on the host system.
pub fn find_chrome_executable() -> Option<PathBuf> {
    // 1. Environment variables
    if let Ok(env_path) = std::env::var("CHROME_BIN").or_else(|_| std::env::var("CHROME_PATH")) {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Platform-specific known locations
    #[cfg(target_os = "macos")]
    {
        let candidates = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/opt/homebrew/bin/chromium",
            "/usr/local/bin/chromium",
        ];
        for candidate in candidates {
            let p = Path::new(candidate);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let candidates = [
            "/usr/bin/google-chrome-stable",
            "/usr/bin/google-chrome",
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
            "/snap/bin/chromium",
        ];
        for candidate in candidates {
            let p = Path::new(candidate);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files\Chromium\Application\chrome.exe",
        ];
        for candidate in candidates {
            let p = Path::new(candidate);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
    }

    None
}

/// Launch a new headless browser instance configured for frame-accurate video rendering.
pub async fn launch_browser(
    options: &BrowserLaunchOptions,
) -> Result<(Browser, tokio::task::JoinHandle<()>)> {
    let chrome_bin = match &options.chrome_path {
        Some(p) => p.clone(),
        None => find_chrome_executable()
            .ok_or_else(|| anyhow!("Could not find Google Chrome or Chromium executable. Set CHROME_BIN environment variable."))?,
    };

    info!("Using Chrome binary at: {}", chrome_bin.display());

    let mut builder = BrowserConfig::builder()
        .chrome_executable(chrome_bin)
        .window_size(options.width, options.height)
        .viewport(chromiumoxide::handler::viewport::Viewport {
            width: options.width,
            height: options.height,
            device_scale_factor: Some(1.0),
            emulating_mobile: false,
            is_landscape: true,
            has_touch: false,
        })
        .arg("--disable-background-timer-throttling")
        .arg("--disable-backgrounding-occluded-windows")
        .arg("--disable-renderer-backgrounding")
        .arg("--hide-scrollbars")
        .arg("--mute-audio")
        .arg("--force-color-profile=srgb");

    if options.headless {
        builder = builder.arg("--headless=new");
    }

    if !options.enable_gpu {
        builder = builder.arg("--disable-gpu");
    } else {
        builder = builder.arg("--enable-gpu-rasterization");
    }

    let config = builder
        .build()
        .map_err(|e| anyhow!("Failed to build browser config: {e}"))?;

    let (browser, mut handler) = Browser::launch(config).await?;

    let handle = tokio::spawn(async move {
        while let Some(event) = futures::StreamExt::next(&mut handler).await {
            if let Err(e) = event {
                tracing::warn!("Browser event error: {:?}", e);
                break;
            }
        }
    });

    Ok((browser, handle))
}
