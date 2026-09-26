use hyperframes::core::audio_automation::{
    HfAutomationLane, HfAutomationPoint, InterpolationScale, sample_automation_lane,
};
use hyperframes::core::audio_fade::{AudioFades, clamp_fades_to_duration, fade_gain};
use hyperframes::core::audio_gain::{
    audio_db_to_gain, audio_fader_position_to_gain, audio_gain_to_db, audio_gain_to_fader_position,
};
use hyperframes::core::clock::{TransportClock, frame_to_time, time_to_frame, total_frames};
use hyperframes::core::fps::{Fps, parse_fps};
use hyperframes::core::speed_ramp::{RateSpec, rate_at, source_time_at, time_at_source_time};

#[test]
fn test_integration_full_clock_pipeline() {
    let mut clock = TransportClock::new(0.0, 1.0, 60.0);
    assert_eq!(clock.duration(), 60.0);
    assert_eq!(clock.now(), 0.0);

    // Discrete seeks
    clock.seek(12.34);
    assert_eq!(clock.now(), 12.34);

    // Speed multiplier changes
    clock.set_rate(2.0);
    assert_eq!(clock.rate(), 2.0);

    // Frame mapping
    let fps = 30.0;
    assert_eq!(time_to_frame(12.34, fps), 370);
    assert!((frame_to_time(370, fps) - 12.333).abs() < 0.01);
    assert_eq!(total_frames(60.0, fps), 1800);
}

#[test]
fn test_integration_speed_ramp_mapping() {
    let lane = HfAutomationLane::new(
        "rate",
        vec![
            HfAutomationPoint::new(0.0, 0.5),
            HfAutomationPoint::new(4.0, 2.0),
        ],
    );
    let spec = RateSpec::Lane(lane.clone());

    // Rate at boundaries
    assert_eq!(rate_at(&spec, 0.0), 0.5);
    assert_eq!(rate_at(&spec, 4.0), 2.0);

    // Integral mapping is monotonic
    let s0 = source_time_at(&spec, 0.0);
    let s2 = source_time_at(&spec, 2.0);
    let s4 = source_time_at(&spec, 4.0);
    assert!(s0 < s2);
    assert!(s2 < s4);

    // Invertibility: time_at_source_time(source_time_at(t)) ≈ t
    for test_t in [0.5, 1.0, 2.5, 3.8] {
        let s = source_time_at(&spec, test_t);
        let recovered_t = time_at_source_time(&spec, s);
        assert!((recovered_t - test_t).abs() < 0.05);
    }
}

#[test]
fn test_integration_audio_envelope_and_gain() {
    let lane = HfAutomationLane::new(
        "volume",
        vec![
            HfAutomationPoint::new(0.0, 0.0),
            HfAutomationPoint::new(1.0, 0.8),
            HfAutomationPoint::new(3.0, 0.8),
            HfAutomationPoint::new(4.0, 0.0),
        ],
    );

    assert_eq!(
        sample_automation_lane(&lane, 0.0, InterpolationScale::Linear),
        0.0
    );
    assert_eq!(
        sample_automation_lane(&lane, 2.0, InterpolationScale::Linear),
        0.8
    );
    assert_eq!(
        sample_automation_lane(&lane, 4.0, InterpolationScale::Linear),
        0.0
    );

    // Fades
    let fades = AudioFades::new(1.0, 1.0);
    let clamped = clamp_fades_to_duration(fades, 5.0);
    assert_eq!(clamped.fade_in, 1.0);
    assert_eq!(clamped.fade_out, 1.0);

    assert_eq!(fade_gain(0.0, 5.0, fades), 0.0);
    assert_eq!(fade_gain(0.5, 5.0, fades), 0.5);
    assert_eq!(fade_gain(2.5, 5.0, fades), 1.0);
    assert_eq!(fade_gain(4.5, 5.0, fades), 0.5);
    assert_eq!(fade_gain(5.0, 5.0, fades), 0.0);

    // Gain fader round trip
    for db in [-30.0, -12.0, -6.0, 0.0, 3.0, 6.0, 12.0] {
        let gain = audio_db_to_gain(db);
        let pos = audio_gain_to_fader_position(gain);
        let recovered_gain = audio_fader_position_to_gain(pos);
        let recovered_db = audio_gain_to_db(recovered_gain);
        assert!((recovered_db - db).abs() < 0.01);
    }
}

#[test]
fn test_integration_fps_ntsc_math() {
    let ntsc_24 = parse_fps("24000/1001").unwrap();
    let ntsc_30 = parse_fps("30000/1001").unwrap();
    let ntsc_60 = parse_fps("60000/1001").unwrap();

    assert_eq!(ntsc_24, Fps::NTSC_24);
    assert_eq!(ntsc_30, Fps::NTSC_30);
    assert_eq!(ntsc_60, Fps::NTSC_60);

    assert!((ntsc_24.as_f64() - 23.976023).abs() < 1e-4);
    assert!((ntsc_30.as_f64() - 29.970029).abs() < 1e-4);
    assert!((ntsc_60.as_f64() - 59.940059).abs() < 1e-4);
}
