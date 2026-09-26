# ⚡ HyperFrames Rust (`hyperframes`)

[![Crates.io](https://img.shields.io/crates/v/hyperframes.svg)](https://crates.io/crates/hyperframes)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue.svg)](https://code.brandonhubbard.com/hyperframes-rs/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](Cargo.toml)
[![CI](https://github.com/bhubbard/hyperframes-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bhubbard/hyperframes-rs/actions)

> High-performance native Rust programmatic video rendering engine with seekable HTML/CSS/JS clock protocol.
>
> 🌐 **Interactive Documentation & Live Seek Studio**: [code.brandonhubbard.com/hyperframes-rs](https://code.brandonhubbard.com/hyperframes-rs/)

A native Rust port and engine inspired by [HeyGen HyperFrames](https://github.com/heygen-com/hyperframes). HyperFrames turns any web page (HTML, CSS, SVG, Canvas, WebGL, GSAP, Three.js, Lottie) into frame-accurate, production-quality video without frame skips, clock drift, or wall-clock dependence.

---

## 🚀 Key Features

- **Frame-Accurate Deterministic Virtual Time**: Controls `performance.now()`, `Date.now()`, and `requestAnimationFrame` via the `window.__hf` seek protocol.
- **Native Chrome DevTools Protocol (CDP)**: Async headless browser automation via [`chromiumoxide`](https://crates.io/crates/chromiumoxide) with zero-overhead frame captures.
- **Piped FFmpeg Streaming Pipeline**: Frame image buffers stream directly into FFmpeg stdin over asynchronous pipes—never writing millions of temporary image files to disk.
- **Multi-Format & Multi-Codec**: First-class support for MP4 (`libx264`, `libx265`), MOV (`prores_ks`), WebM (`libvpx-vp9`), and animated GIFs.
- **DAW-Grade Audio Automation**: Exponential, linear, logarithmic, and conic via-point envelope interpolation for volume, speed ramps, and FX automation.
- **Zero-Config CLI & Pure Rust SDK**: Use it as a turnkey command-line tool or embed it directly into your Rust video pipelines.

---

## 📦 Installation

### From Cargo
```bash
cargo install hyperframes
```

### In Your `Cargo.toml`
```toml
[dependencies]
hyperframes = "0.2.0"
```

---

## 🛠️ Prerequisites

HyperFrames requires:
1. **Google Chrome** or **Chromium** (auto-detected on macOS, Linux, and Windows, or configured via `CHROME_BIN`).
2. **FFmpeg** installed and accessible on your `PATH`.

---

## 💻 CLI Usage

### 1. Scaffold a New Project
```bash
hyperframes init my-video
cd my-video
```

### 2. Inspect Composition Metadata
```bash
hyperframes info index.html
```

### 3. Render into High-Definition MP4
```bash
# Render using composition's native duration and resolution
hyperframes render index.html -o video.mp4

# Render with custom overrides
hyperframes render index.html \
  -o cinematic.mp4 \
  --fps 60 \
  --width 3840 \
  --height 2160 \
  --codec h265 \
  --crf 16
```

### 4. Render Apple ProRes 4444 (MOV)
```bash
hyperframes render index.html -o master.mov --format mov --codec prores
```

---

## 📜 The `window.__hf` Seek Protocol

HyperFrames does not force a specific animation library or frontend framework. Any page that implements `window.__hf` can be rendered deterministically:

```html
<!DOCTYPE html>
<html>
<head>
    <meta name="hyperframes:duration" content="5.0">
    <meta name="hyperframes:fps" content="30">
    <meta name="hyperframes:resolution" content="1920x1080">
</head>
<body>
    <h1 id="title">Hello World</h1>

    <script>
        window.__hf = {
            duration: 5.0,
            fps: 30,
            seek: function(timeSeconds) {
                const progress = timeSeconds / 5.0;
                document.getElementById('title').style.opacity = progress;
            }
        };
    </script>
</body>
</html>
```

*Note: For pages without an authored `window.__hf`, HyperFrames automatically injects a deterministic virtual clock shim that intercepts `performance.now()` and `requestAnimationFrame`!*

---

## 🦀 Programmatic Rust API

```rust
use hyperframes::{render_composition, RenderOptions, VideoCodec, VideoFormat};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = RenderOptions {
        input_path: "index.html".to_string(),
        output_path: "output.mp4".to_string(),
        width: Some(1920),
        height: Some(1080),
        fps: Some(30.0),
        duration: Some(5.0),
        format: VideoFormat::Mp4,
        codec: VideoCodec::H264,
        crf: 18,
        enable_gpu: false,
        quiet: false,
    };

    let stats = render_composition(options).await?;
    println!("Encoded {} frames in {:.2}s!", stats.total_frames, stats.elapsed_time_seconds);

    Ok(())
}
```

---

## 🧪 Testing & Verification

Run the test suite:
```bash
cargo test
```

Run linter checks:
```bash
cargo clippy -- -D warnings
```

---

## ⚖️ License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).
