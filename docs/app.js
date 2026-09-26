/**
 * HyperFrames-rs Documentation & Interactive Studio
 * Deterministic virtual clock simulation and code generator
 */

(function () {
  'use strict';

  // --- State ---
  const state = {
    currentTime: 0.0,
    duration: 5.0,
    fps: 30,
    isPlaying: false,
    colorPreset: 'neutral',
    faderValue: 0, // -100 to +100
    lastTimestamp: 0,
  };

  // Preset color grading filters
  const colorGradingFilters = {
    'neutral': 'none',
    'warm-daylight': 'sepia(0.25) contrast(1.1) brightness(1.05) saturate(1.15)',
    'mono-clean': 'grayscale(1) contrast(1.25) brightness(1.02)',
    'vintage-wash': 'sepia(0.4) contrast(0.9) brightness(1.1) hue-rotate(-15deg)',
    'bright-pop': 'saturate(1.6) contrast(1.15) brightness(1.02)',
  };

  // --- DOM Elements ---
  const canvas = document.getElementById('studioCanvas');
  const ctx = canvas ? canvas.getContext('2d') : null;

  const scrubber = document.getElementById('timelineScrubber');
  const timecodeDisplay = document.getElementById('hudTimecode');
  const frameDisplay = document.getElementById('hudFrame');
  const timeDisplay = document.getElementById('hudTime');
  const playBtn = document.getElementById('playPauseBtn');
  const stepBackBtn = document.getElementById('stepBackBtn');
  const stepFwdBtn = document.getElementById('stepFwdBtn');
  const jumpBackBtn = document.getElementById('jumpBackBtn');
  const jumpFwdBtn = document.getElementById('jumpFwdBtn');
  const fpsSelect = document.getElementById('fpsSelect');
  const colorPresetSelect = document.getElementById('colorPresetSelect');
  const faderSlider = document.getElementById('faderSlider');
  const faderDbLabel = document.getElementById('faderDbLabel');

  // Generator DOM Elements
  const inputSource = document.getElementById('inputSource');
  const outputDest = document.getElementById('outputDest');
  const presetRes = document.getElementById('presetRes');
  const customWidth = document.getElementById('customWidth');
  const customHeight = document.getElementById('customHeight');
  const genFps = document.getElementById('genFps');
  const genCodec = document.getElementById('genCodec');
  const genCrf = document.getElementById('genCrf');
  const crfVal = document.getElementById('crfVal');
  const genGpu = document.getElementById('genGpu');

  const cliBlock = document.getElementById('cliCodeBlock');
  const rustBlock = document.getElementById('rustCodeBlock');
  const tabCli = document.getElementById('tabCli');
  const tabRust = document.getElementById('tabRust');
  const copyCliBtn = document.getElementById('copyCliBtn');
  const copyRustBtn = document.getElementById('copyRustBtn');
  const toast = document.getElementById('toast');

  // --- Audio Gain Math (matching hyperframes::core::audio_gain) ---
  function audioFaderToDb(fader) {
    if (fader <= -100) return -Infinity;
    if (fader === 0) return 0.0;
    if (fader < 0) {
      // [-100, 0) maps to [-60dB, 0dB]
      return (fader / 100.0 + 1.0) * 60.0 - 60.0;
    } else {
      // (0, 100] maps to (0dB, +12dB]
      return (fader / 100.0) * 12.0;
    }
  }

  function audioDbToLinear(db) {
    if (db === -Infinity) return 0.0;
    return Math.pow(10.0, db / 20.0);
  }

  // --- Timecode Formatting ---
  function formatTimecode(timeSec, fps) {
    const totalFrames = Math.max(0, Math.floor(timeSec * fps));
    const frame = totalFrames % Math.round(fps);
    const totalSeconds = Math.floor(totalFrames / fps);
    const sec = totalSeconds % 60;
    const min = Math.floor(totalSeconds / 60) % 60;
    const hrs = Math.floor(totalSeconds / 3600);

    const pad = (n) => String(n).padStart(2, '0');
    return `${pad(hrs)}:${pad(min)}:${pad(sec)}:${pad(frame)}`;
  }

  // --- Canvas Rendering (Deterministic Seek Simulation) ---
  function resizeCanvas() {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    renderFrame(state.currentTime);
  }

  function renderFrame(t) {
    if (!ctx || !canvas) return;
    const w = canvas.width;
    const h = canvas.height;
    const dpr = window.devicePixelRatio || 1;

    // Apply color grading CSS filter
    canvas.style.filter = colorGradingFilters[state.colorPreset] || 'none';

    // Background gradient
    const bgGrad = ctx.createLinearGradient(0, 0, w, h);
    bgGrad.addColorStop(0, '#06070a');
    bgGrad.addColorStop(0.5, '#0b0f1a');
    bgGrad.addColorStop(1, '#05070c');
    ctx.fillStyle = bgGrad;
    ctx.fillRect(0, 0, w, h);

    // Subtle Grid
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.04)';
    ctx.lineWidth = 1 * dpr;
    const gridSize = 40 * dpr;
    for (let x = 0; x < w; x += gridSize) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, h);
      ctx.stroke();
    }
    for (let y = 0; y < h; y += gridSize) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(w, y);
      ctx.stroke();
    }

    const cx = w / 2;
    const cy = h / 2;
    const progress = t / state.duration;

    // Centerpiece: Kinetic 3D Wireframe Polyhedron
    ctx.save();
    ctx.translate(cx, cy);

    const rotX = t * 1.5;
    const rotY = t * 2.2;
    const scale = Math.min(w, h) * 0.22;

    // 8 Vertices of a 3D Cube
    const vertices = [
      [-1, -1, -1], [1, -1, -1], [1, 1, -1], [-1, 1, -1],
      [-1, -1, 1], [1, -1, 1], [1, 1, 1], [-1, 1, 1]
    ];

    // Edges
    const edges = [
      [0,1], [1,2], [2,3], [3,0],
      [4,5], [5,6], [6,7], [7,4],
      [0,4], [1,5], [2,6], [3,7]
    ];

    // Project 3D to 2D
    const projected = vertices.map(([vx, vy, vz]) => {
      // Rotation around Y
      let x1 = vx * Math.cos(rotY) + vz * Math.sin(rotY);
      let z1 = -vx * Math.sin(rotY) + vz * Math.cos(rotY);
      // Rotation around X
      let y2 = vy * Math.cos(rotX) - z1 * Math.sin(rotX);
      let z2 = vy * Math.sin(rotX) + z1 * Math.cos(rotX);

      const fov = 3.5;
      const factor = fov / (fov + z2);
      return [x1 * scale * factor, y2 * scale * factor, z2];
    });

    // Draw Edges with glowing gradient
    ctx.lineWidth = 2.5 * dpr;
    edges.forEach(([i1, i2]) => {
      const p1 = projected[i1];
      const p2 = projected[i2];
      const depth = (p1[2] + p2[2]) / 2;
      const alpha = 0.4 + (depth + 1.5) * 0.25;

      const grad = ctx.createLinearGradient(p1[0], p1[1], p2[0], p2[1]);
      grad.addColorStop(0, `rgba(0, 242, 254, ${alpha})`);
      grad.addColorStop(1, `rgba(139, 92, 246, ${alpha})`);

      ctx.strokeStyle = grad;
      ctx.beginPath();
      ctx.moveTo(p1[0], p1[1]);
      ctx.lineTo(p2[0], p2[1]);
      ctx.stroke();
    });

    // Draw Vertex nodes
    projected.forEach(([px, py, pz]) => {
      const radius = Math.max(2, (3.5 + pz * 1.5) * dpr);
      ctx.fillStyle = '#00f2fe';
      ctx.shadowColor = '#00f2fe';
      ctx.shadowBlur = 10 * dpr;
      ctx.beginPath();
      ctx.arc(px, py, radius, 0, Math.PI * 2);
      ctx.fill();
      ctx.shadowBlur = 0;
    });

    ctx.restore();

    // Motion Typography
    ctx.save();
    ctx.font = `bold ${28 * dpr}px -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`;
    ctx.textAlign = 'center';
    ctx.fillStyle = '#ffffff';
    ctx.shadowColor = 'rgba(0, 242, 254, 0.4)';
    ctx.shadowBlur = 15 * dpr;
    const titleOpacity = Math.min(1, Math.max(0.1, Math.sin(progress * Math.PI) * 1.5));
    ctx.globalAlpha = titleOpacity;
    ctx.fillText('HYPERFRAMES ENGINE', cx, cy - scale - 30 * dpr);
    ctx.shadowBlur = 0;

    // Subtitle
    ctx.font = `500 ${13 * dpr}px "JetBrains Mono", monospace`;
    ctx.fillStyle = '#00f2fe';
    ctx.fillText(`SEEK: ${(t).toFixed(3)}s  •  TIMELINE PROGRESS: ${(progress * 100).toFixed(1)}%`, cx, cy + scale + 40 * dpr);
    ctx.restore();

    // Audio Visualizer Bar at the bottom
    const db = audioFaderToDb(state.faderValue);
    const linearGain = audioDbToLinear(db);
    const barCount = 32;
    const barWidth = 6 * dpr;
    const barGap = 4 * dpr;
    const totalBarsWidth = barCount * (barWidth + barGap);
    const startX = cx - totalBarsWidth / 2;
    const barBaseY = h - 25 * dpr;

    for (let i = 0; i < barCount; i++) {
      const freq = (i + 1) * 2.5;
      const rawHeight = Math.sin(t * 8.0 + i * 0.4) * Math.cos(t * 5.0 + i * 0.2);
      const normalizedHeight = Math.max(0.08, (rawHeight + 1.0) / 2.0);
      const barH = normalizedHeight * (40 * dpr) * linearGain;

      const bx = startX + i * (barWidth + barGap);
      const by = barBaseY - barH;

      ctx.fillStyle = i > 26 ? '#f43f5e' : (i > 18 ? '#f59e0b' : '#10b981');
      ctx.fillRect(bx, by, barWidth, barH);
    }
  }

  // --- Playback Loop ---
  function updateTelemetry() {
    const frame = Math.floor(state.currentTime * state.fps);
    if (frameDisplay) frameDisplay.textContent = String(frame);
    if (timeDisplay) timeDisplay.textContent = state.currentTime.toFixed(3) + 's';
    if (timecodeDisplay) timecodeDisplay.textContent = formatTimecode(state.currentTime, state.fps);
    if (scrubber) scrubber.value = state.currentTime;
  }

  function loop(timestamp) {
    if (!state.lastTimestamp) state.lastTimestamp = timestamp;
    const delta = (timestamp - state.lastTimestamp) / 1000.0;
    state.lastTimestamp = timestamp;

    if (state.isPlaying) {
      state.currentTime += delta;
      if (state.currentTime >= state.duration) {
        state.currentTime = 0; // loop
      }
      updateTelemetry();
      renderFrame(state.currentTime);
    }

    requestAnimationFrame(loop);
  }

  // --- Interactive Handlers ---
  function togglePlay() {
    state.isPlaying = !state.isPlaying;
    state.lastTimestamp = 0;
    if (playBtn) {
      playBtn.innerHTML = state.isPlaying
        ? `<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg> Pause`
        : `<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg> Play`;
      playBtn.classList.toggle('active', state.isPlaying);
    }
  }

  function seekTo(timeSec) {
    state.currentTime = Math.max(0, Math.min(state.duration, timeSec));
    updateTelemetry();
    renderFrame(state.currentTime);
  }

  function stepFrames(n) {
    const frameDuration = 1.0 / state.fps;
    seekTo(state.currentTime + n * frameDuration);
  }

  // --- Code Generator Logic ---
  function updateGenerator() {
    const src = inputSource.value || 'index.html';
    const dest = outputDest.value || 'video.mp4';
    const w = customWidth.value;
    const h = customHeight.value;
    const fps = genFps.value;
    const codec = genCodec.value;
    const crf = genCrf.value;
    const gpu = genGpu.checked;

    if (crfVal) crfVal.textContent = crf;

    // Build CLI string
    let cli = `hyperframes render ${src} \\\n  -o ${dest} \\\n  --fps ${fps} \\\n  --width ${w} \\\n  --height ${h} \\\n  --codec ${codec} \\\n  --crf ${crf}`;
    if (gpu) {
      cli += ` \\\n  --gpu`;
    }

    if (cliBlock) cliBlock.textContent = cli;

    // Build Rust string
    const formatEnum = dest.endsWith('.mov') ? 'VideoFormat::Mov' : (dest.endsWith('.webm') ? 'VideoFormat::WebM' : 'VideoFormat::Mp4');
    const codecEnum = codec === 'h265' ? 'VideoCodec::H265' : (codec === 'prores' ? 'VideoCodec::ProRes' : (codec === 'vp9' ? 'VideoCodec::Vp9' : 'VideoCodec::H264'));

    const rust = `use hyperframes::{render_composition, RenderOptions, VideoCodec, VideoFormat};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = RenderOptions {
        input_path: "${src}".to_string(),
        output_path: "${dest}".to_string(),
        width: Some(${w}),
        height: Some(${h}),
        fps: Some(${fps}.0),
        duration: None, // Auto-detect from meta or window.__hf
        format: ${formatEnum},
        codec: ${codecEnum},
        crf: ${crf},
        enable_gpu: ${gpu},
        quiet: false,
    };

    let stats = render_composition(options).await?;
    println!("Encoded {} frames in {:.2}s!", stats.total_frames, stats.elapsed_time_seconds);

    Ok(())
}`;

    if (rustBlock) rustBlock.textContent = rust;
  }

  function handlePresetResolution() {
    const val = presetRes.value;
    if (val === '1080p') {
      customWidth.value = 1920;
      customHeight.value = 1080;
    } else if (val === '4k') {
      customWidth.value = 3840;
      customHeight.value = 2160;
    } else if (val === '720p') {
      customWidth.value = 1280;
      customHeight.value = 720;
    } else if (val === '9:16') {
      customWidth.value = 1080;
      customHeight.value = 1920;
    } else if (val === 'square') {
      customWidth.value = 1080;
      customHeight.value = 1080;
    }
    updateGenerator();
  }

  function showToast(msg) {
    if (!toast) return;
    toast.textContent = msg;
    toast.classList.add('show');
    setTimeout(() => {
      toast.classList.remove('show');
    }, 2500);
  }

  function copyText(text, label) {
    navigator.clipboard.writeText(text).then(() => {
      showToast(`Copied ${label} to clipboard!`);
    }).catch(() => {
      showToast(`Failed to copy`);
    });
  }

  // --- Attach Event Listeners ---
  function initListeners() {
    window.addEventListener('resize', resizeCanvas);

    if (playBtn) playBtn.addEventListener('click', togglePlay);
    if (stepBackBtn) stepBackBtn.addEventListener('click', () => stepFrames(-1));
    if (stepFwdBtn) stepFwdBtn.addEventListener('click', () => stepFrames(1));
    if (jumpBackBtn) jumpBackBtn.addEventListener('click', () => stepFrames(-Math.round(state.fps)));
    if (jumpFwdBtn) jumpFwdBtn.addEventListener('click', () => stepFrames(Math.round(state.fps)));

    if (scrubber) {
      scrubber.addEventListener('input', (e) => {
        state.isPlaying = false;
        if (playBtn) {
          playBtn.innerHTML = `<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg> Play`;
          playBtn.classList.remove('active');
        }
        seekTo(parseFloat(e.target.value));
      });
    }

    if (fpsSelect) {
      fpsSelect.addEventListener('change', (e) => {
        state.fps = parseFloat(e.target.value);
        updateTelemetry();
        renderFrame(state.currentTime);
      });
    }

    if (colorPresetSelect) {
      colorPresetSelect.addEventListener('change', (e) => {
        state.colorPreset = e.target.value;
        renderFrame(state.currentTime);
      });
    }

    if (faderSlider) {
      faderSlider.addEventListener('input', (e) => {
        state.faderValue = parseInt(e.target.value, 10);
        const db = audioFaderToDb(state.faderValue);
        const display = db === -Infinity ? '-inf dB' : `${db >= 0 ? '+' : ''}${db.toFixed(1)} dB`;
        if (faderDbLabel) faderDbLabel.textContent = display;
        renderFrame(state.currentTime);
      });
    }

    // Keyboard shortcuts (Space to play/pause, Left/Right arrow to step)
    window.addEventListener('keydown', (e) => {
      if (['INPUT', 'SELECT', 'TEXTAREA'].includes(e.target.tagName)) return;
      if (e.code === 'Space') {
        e.preventDefault();
        togglePlay();
      } else if (e.code === 'ArrowLeft') {
        e.preventDefault();
        stepFrames(-1);
      } else if (e.code === 'ArrowRight') {
        e.preventDefault();
        stepFrames(1);
      }
    });

    // Generator inputs
    const genInputs = [inputSource, outputDest, customWidth, customHeight, genFps, genCodec, genCrf, genGpu];
    genInputs.forEach((el) => {
      if (el) el.addEventListener('input', updateGenerator);
    });

    if (presetRes) presetRes.addEventListener('change', handlePresetResolution);

    // Tab buttons
    if (tabCli && tabRust) {
      tabCli.addEventListener('click', () => {
        tabCli.classList.add('active');
        tabRust.classList.remove('active');
        cliBlock.style.display = 'block';
        rustBlock.style.display = 'none';
        copyCliBtn.style.display = 'inline-flex';
        copyRustBtn.style.display = 'none';
      });
      tabRust.addEventListener('click', () => {
        tabRust.classList.add('active');
        tabCli.classList.remove('active');
        rustBlock.style.display = 'block';
        cliBlock.style.display = 'none';
        copyRustBtn.style.display = 'inline-flex';
        copyCliBtn.style.display = 'none';
      });
    }

    if (copyCliBtn) {
      copyCliBtn.addEventListener('click', () => copyText(cliBlock.textContent, 'CLI command'));
    }
    if (copyRustBtn) {
      copyRustBtn.addEventListener('click', () => copyText(rustBlock.textContent, 'Rust code'));
    }
  }

  // --- Bootstrap ---
  window.addEventListener('DOMContentLoaded', () => {
    initListeners();
    resizeCanvas();
    updateTelemetry();
    updateGenerator();
    requestAnimationFrame(loop);
  });
})();
