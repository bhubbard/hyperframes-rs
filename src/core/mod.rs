pub mod audio_automation;
pub mod audio_fade;
pub mod audio_gain;
pub mod clock;
pub mod color_grading;
pub mod fps;
pub mod parser;
pub mod protocol;
pub mod safe_path;
pub mod speed_ramp;
pub mod text;
pub mod vfx;

pub use audio_automation::{
    HfAutomationLane, HfAutomationPoint, InterpolationScale, apply_curve, sample_automation_curve,
    sample_automation_lane, shape_progress, shape_via,
};
pub use audio_fade::{
    AudioFades, clamp_fades_to_duration, fade_gain, format_fade_seconds, read_fade_seconds,
};
pub use audio_gain::{
    audio_db_to_gain, audio_fader_position_to_gain, audio_gain_to_db, audio_gain_to_fader_position,
    audio_gain_to_text, clamp_audio_gain, format_audio_gain, max_audio_gain,
};
pub use clock::{
    ClockSource, TransportClock, TransportClockSnapshot, frame_to_time, time_to_frame, total_frames,
};
pub use color_grading::{ColorAdjustments, ColorGradingPreset, find_preset, standard_presets};
pub use fps::{Fps, FpsParseError, parse_fps, parse_fps_with_default};
pub use parser::{
    CompositionMeta, generate_runtime_shim, merge_metadata, parse_html_metadata,
    resolve_composition_url,
};
pub use protocol::{
    HfMediaElement, HfProtocol, HfSeekOptions, HfTransitionMeta, VideoCodec, VideoFormat,
};
pub use safe_path::{is_safe_path, resolve_within_project};
pub use text::{FitTextOptions, FitTextResult, estimate_text_width, fit_text_font_size};
pub use vfx::{HfVfxCapture, HfVfxChain, HfVfxNode, STANDARD_VFX_DEFS};
