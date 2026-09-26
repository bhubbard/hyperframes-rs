//! Benchmark comparing `hyperframes-rs` (Rust) vs Remotion / Motion Canvas (Node.js / TypeScript).

use hyperframes::core::audio_automation::{
    HfAutomationLane, HfAutomationPoint, InterpolationScale, sample_automation_lane,
};
use hyperframes::core::clock::{TransportClock, frame_to_time, time_to_frame};
use hyperframes::core::color_grading::{ColorAdjustments, find_preset, standard_presets};
use hyperframes::core::speed_ramp::{RateSpec, rate_at};
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("  hyperframes-rs (Rust) vs Remotion / Motion Canvas (Node)  ");
    println!("============================================================");

    // 1. High-Precision Transport Clock Frame Mapping
    println!("\n--- 1. Transport Clock & Sub-Millisecond Frame Indexing ---");
    {
        let mut clock = TransportClock::new(0.0, 1.0, 3600.0); // 1-hour timeline
        let fps = 60.0;
        let iterations = 10_000_000;
        let start = Instant::now();
        let mut sum_frames = 0u64;

        for i in 0..iterations {
            let t = (i as f64) * 0.0003;
            clock.seek(t);
            let f = time_to_frame(clock.now(), fps);
            let _rec_t = frame_to_time(f, fps);
            sum_frames += f;
        }

        std::hint::black_box(sum_frames);
        let elapsed = start.elapsed();
        let ns_per_seek = elapsed.as_nanos() as f64 / iterations as f64;
        let seeks_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Clock Seeks: {} | Time: {:.2?} | Latency: {:.2} ns/seek | {:>10.0} seeks/s",
            iterations, elapsed, ns_per_seek, seeks_per_sec
        );
    }

    // 2. Non-Linear Speed Ramp Time Warping
    println!("\n--- 2. Non-Linear Speed Ramp Integral Time Warping ---");
    {
        let lane = HfAutomationLane::new(
            "rate",
            vec![
                HfAutomationPoint::new(0.0, 0.5),
                HfAutomationPoint::new(2.0, 2.5),
                HfAutomationPoint::new(5.0, 1.0),
                HfAutomationPoint::new(10.0, 0.25),
            ],
        );
        let spec = RateSpec::Lane(lane.clone());
        let table = hyperframes::core::speed_ramp::RateTable::build(&lane);

        let iterations = 10_000_000;
        let start = Instant::now();
        let mut sum_source_times = 0.0;

        for i in 0..iterations {
            let t = (i % 1000) as f64 * 0.01;
            let s = table.source_time_at(t);
            let _rate = rate_at(&spec, t);
            sum_source_times += s;
        }

        std::hint::black_box(sum_source_times);
        let elapsed = start.elapsed();
        let ns_per_warp = elapsed.as_nanos() as f64 / iterations as f64;
        let warps_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Speed Ramp Evals: {} | Time: {:.2?} | Latency: {:.2} ns/eval | {:>10.0} evals/s",
            iterations, elapsed, ns_per_warp, warps_per_sec
        );
    }

    // 3. Audio Automation Multi-Point Lane Sampling
    println!("\n--- 3. Audio Automation Bezier Curve Lane Interpolation ---");
    {
        let lane = HfAutomationLane::new(
            "volume",
            vec![
                HfAutomationPoint::new(0.0, 0.0),
                HfAutomationPoint::new(1.5, 0.8),
                HfAutomationPoint::new(4.0, 1.0),
                HfAutomationPoint::new(8.0, 0.3),
                HfAutomationPoint::new(10.0, 0.0),
            ],
        );

        let iterations = 10_000_000;
        let start = Instant::now();
        let mut sum_volumes = 0.0;

        for i in 0..iterations {
            let t = (i % 1000) as f64 * 0.01;
            let val = sample_automation_lane(&lane, t, InterpolationScale::Linear);
            sum_volumes += val;
        }

        std::hint::black_box(sum_volumes);
        let elapsed = start.elapsed();
        let ns_per_sample = elapsed.as_nanos() as f64 / iterations as f64;
        let samples_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Curve Samples: {} | Time: {:.2?} | Latency: {:.2} ns/sample | {:>10.0} samples/s",
            iterations, elapsed, ns_per_sample, samples_per_sec
        );
    }

    // 4. Color Grading Preset Lookup & Tone Mapping
    println!("\n--- 4. Color Grading Adjustments & Preset Pipeline ---");
    {
        let preset = find_preset("cinematic").unwrap_or_else(|| standard_presets()[0].clone());
        let iterations = 10_000_000;
        let start = Instant::now();
        let mut sum_contrast = 0.0f64;

        for i in 0..iterations {
            let mut adj = ColorAdjustments::default();
            adj.exposure = preset.adjust.exposure + (i % 10) as f64 * 0.05;
            adj.contrast = preset.adjust.contrast;
            adj.saturation = preset.adjust.saturation;
            sum_contrast += adj.contrast + adj.exposure;
        }

        std::hint::black_box(sum_contrast);
        let elapsed = start.elapsed();
        let ns_per_adj = elapsed.as_nanos() as f64 / iterations as f64;
        let adjs_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Color Adjustments: {} | Time: {:.2?} | Latency: {:.2} ns/eval | {:>10.0} evals/s",
            iterations, elapsed, ns_per_adj, adjs_per_sec
        );
    }

    println!("\n============================================================");
    println!("                      Benchmark Complete                    ");
    println!("============================================================");
}
