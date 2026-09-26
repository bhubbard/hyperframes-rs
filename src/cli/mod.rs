use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::Path;

use crate::core::parser::{merge_metadata, parse_html_metadata, resolve_composition_url};
use crate::core::protocol::{HfProtocol, VideoCodec, VideoFormat};
use crate::producer::pipeline::{RenderOptions, render_composition};

#[derive(Parser, Debug)]
#[command(
    name = "hyperframes",
    version,
    about = "High-performance native Rust programmatic video engine with seekable HTML/CSS/JS clock protocol",
    author = "Brandon Hubbard <bhubbard@gmail.com>"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Render an HTML composition or URL into video (MP4, MOV, WebM, GIF).
    Render {
        /// Path to the HTML composition or URL.
        input: String,

        /// Output video file path.
        #[arg(short, long, default_value = "output.mp4")]
        output: String,

        /// Target frame rate in frames per second.
        #[arg(long)]
        fps: Option<f64>,

        /// Target width in pixels.
        #[arg(long)]
        width: Option<u32>,

        /// Target height in pixels.
        #[arg(long)]
        height: Option<u32>,

        /// Override total duration in seconds.
        #[arg(long)]
        duration: Option<f64>,

        /// Output video format (mp4, mov, webm, gif).
        #[arg(long, default_value = "mp4")]
        format: String,

        /// Video encoder codec (h264, h265, prores, vp9).
        #[arg(long, default_value = "h264")]
        codec: String,

        /// Constant Rate Factor (CRF) quality (lower is higher quality, default: 18).
        #[arg(long, default_value_t = 18)]
        crf: u32,

        /// Enable GPU hardware rasterization in headless Chrome.
        #[arg(long)]
        gpu: bool,

        /// Suppress terminal progress output.
        #[arg(short, long)]
        quiet: bool,
    },

    /// Inspect an HTML composition and display its duration, FPS, and metadata.
    Info {
        /// Path to the HTML composition or URL.
        input: String,
    },

    /// Scaffold a new seekable HTML/CSS/JS animation project.
    Init {
        /// Target folder name or project title.
        #[arg(default_value = "my-animation")]
        name: String,
    },
}

pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render {
            input,
            output,
            fps,
            width,
            height,
            duration,
            format,
            codec,
            crf,
            gpu,
            quiet,
        } => {
            let video_format = format.parse::<VideoFormat>().map_err(|e| anyhow!("{e}"))?;
            let video_codec = codec.parse::<VideoCodec>().map_err(|e| anyhow!("{e}"))?;

            let render_opts = RenderOptions {
                input_path: input,
                output_path: output,
                width,
                height,
                fps,
                duration,
                format: video_format,
                codec: video_codec,
                crf,
                enable_gpu: gpu,
                quiet,
            };

            let stats = render_composition(render_opts).await?;

            if !quiet {
                println!("\n✨ Render complete!");
                println!("  🎬 Output:   {}", stats.output_path);
                println!(
                    "  🎞️ Frames:   {} @ {:.1} fps",
                    stats.total_frames, stats.fps
                );
                println!("  📐 Size:     {}x{}", stats.width, stats.height);
                println!(
                    "  ⏱️ Duration: {:.2}s (encoded in {:.2}s)",
                    stats.duration_seconds, stats.elapsed_time_seconds
                );
                println!(
                    "  📦 File:     {:.2} MB",
                    stats.output_bytes as f64 / (1024.0 * 1024.0)
                );
            }
        }

        Commands::Info { input } => {
            println!("🔍 Inspecting composition: {input}");
            let resolved_url = resolve_composition_url(&input);
            println!("   Resolved URL: {resolved_url}");

            if Path::new(&input).exists() {
                if let Ok(content) = std::fs::read_to_string(&input) {
                    let meta = parse_html_metadata(&content);
                    let proto = merge_metadata(HfProtocol::default(), meta.clone());

                    println!("\n📊 Static Metadata Detected:");
                    if let Some(t) = meta.title {
                        println!("   Title:      {t}");
                    }
                    println!("   Duration:   {:.2}s", proto.duration);
                    println!("   FPS:        {:.1}", proto.fps);
                    println!("   Resolution: {}x{}", proto.width, proto.height);
                    println!(
                        "   Total Frames: {}",
                        (proto.duration * proto.fps).ceil() as u64
                    );
                }
            } else {
                println!(
                    "   (Remote URL; render with 'hyperframes render' to evaluate dynamic DOM runtime)"
                );
            }
        }

        Commands::Init { name } => {
            let target_dir = Path::new(&name);
            if target_dir.exists() {
                return Err(anyhow!("Directory '{}' already exists", name));
            }
            std::fs::create_dir_all(target_dir)?;

            let sample_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="hyperframes:duration" content="5.0">
    <meta name="hyperframes:fps" content="30">
    <meta name="hyperframes:resolution" content="1920x1080">
    <title>HyperFrames Animation</title>
    <style>
        body, html {
            margin: 0;
            padding: 0;
            width: 100vw;
            height: 100vh;
            background: #0d1117;
            overflow: hidden;
            display: flex;
            align-items: center;
            justify-content: center;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }
        .container {
            text-align: center;
        }
        .title {
            color: #58a6ff;
            font-size: 72px;
            font-weight: 800;
            letter-spacing: -1px;
            margin-bottom: 16px;
            transform: scale(0.8);
            opacity: 0;
        }
        .badge {
            display: inline-block;
            background: #238636;
            color: #ffffff;
            font-size: 24px;
            font-weight: 600;
            padding: 8px 24px;
            border-radius: 9999px;
            transform: translateY(20px);
            opacity: 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div id="title" class="title">⚡ HyperFrames Rust</div>
        <div id="badge" class="badge">Frame-Accurate Video</div>
    </div>

    <script>
        // Native window.__hf Seek Protocol implementation
        window.__hf = {
            duration: 5.0,
            fps: 30,
            width: 1920,
            height: 1080,
            seek: function(timeSeconds) {
                const title = document.getElementById('title');
                const badge = document.getElementById('badge');

                // Animate title from 0.0s to 1.5s
                const pTitle = Math.min(1.0, Math.max(0.0, timeSeconds / 1.5));
                title.style.opacity = pTitle;
                title.style.transform = `scale(${0.8 + 0.2 * pTitle})`;

                // Animate badge from 1.0s to 2.5s
                const pBadge = Math.min(1.0, Math.max(0.0, (timeSeconds - 1.0) / 1.5));
                badge.style.opacity = pBadge;
                badge.style.transform = `translateY(${20 * (1.0 - pBadge)}px)`;
            }
        };
    </script>
</body>
</html>"#;

            let file_path = target_dir.join("index.html");
            std::fs::write(&file_path, sample_html)?;

            println!("✨ Created new HyperFrames project in '{}'", name);
            println!("   File: {}", file_path.display());
            println!("\nTo render into video:");
            println!(
                "   hyperframes render {}/index.html -o {}/video.mp4",
                name, name
            );
        }
    }

    Ok(())
}
