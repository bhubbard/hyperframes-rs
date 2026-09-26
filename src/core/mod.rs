pub mod audio_automation;
pub mod clock;
pub mod parser;
pub mod protocol;
pub mod speed_ramp;

pub use audio_automation::{
    HfAutomationLane, HfAutomationPoint, InterpolationScale, apply_curve, sample_automation_curve,
    sample_automation_lane, shape_progress, shape_via,
};
pub use clock::{
    ClockSource, TransportClock, TransportClockSnapshot, frame_to_time, time_to_frame, total_frames,
};
pub use parser::{
    CompositionMeta, generate_runtime_shim, merge_metadata, parse_html_metadata,
    resolve_composition_url,
};
pub use protocol::{
    HfMediaElement, HfProtocol, HfSeekOptions, HfTransitionMeta, VideoCodec, VideoFormat,
};
