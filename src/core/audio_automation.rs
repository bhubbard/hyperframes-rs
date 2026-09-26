use serde::{Deserialize, Serialize};

pub const HF_AUDIO_AUTOMATION_VERSION: u32 = 1;
pub const MAX_AUTOMATION_POINTS: usize = 512;

/// A breakpoint in an automation lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HfAutomationPoint {
    /// Seconds from the start of the clip.
    pub t: f64,
    /// Value in parameter unit (e.g. dB, Hz, ratio).
    pub v: f64,
    /// Curvature of segment leaving this point: -1.0 to 1.0 (0.0 is linear).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve: Option<f64>,
    /// Normalized segment space interior point (0.0 .. 1.0).
    #[serde(rename = "viaX", skip_serializing_if = "Option::is_none")]
    pub via_x: Option<f64>,
    #[serde(rename = "viaY", skip_serializing_if = "Option::is_none")]
    pub via_y: Option<f64>,
}

impl HfAutomationPoint {
    pub fn new(t: f64, v: f64) -> Self {
        Self {
            t: t.max(0.0),
            v,
            curve: None,
            via_x: None,
            via_y: None,
        }
    }

    pub fn with_curve(mut self, curve: f64) -> Self {
        self.curve = Some(curve.clamp(-1.0, 1.0));
        self
    }

    pub fn with_via(mut self, via_x: f64, via_y: f64) -> Self {
        self.via_x = Some(via_x.clamp(0.001, 0.999));
        self.via_y = Some(via_y.clamp(0.001, 0.999));
        self
    }
}

/// An automation lane over a single parameter (e.g., "volume", "rate").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HfAutomationLane {
    pub target: String,
    pub points: Vec<HfAutomationPoint>,
}

impl HfAutomationLane {
    pub fn new(target: impl Into<String>, points: Vec<HfAutomationPoint>) -> Self {
        let mut sorted = points;
        sorted.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
        Self {
            target: target.into(),
            points: sorted,
        }
    }
}

/// Scale for interpolating between values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterpolationScale {
    #[default]
    Linear,
    Log,
}

/// Shape the 0.0 .. 1.0 progress across a segment using the curve parameter.
pub fn apply_curve(x: f64, curve: Option<f64>) -> f64 {
    match curve {
        Some(c) if c != 0.0 => x.powf(2.0_f64.powf(2.0 * c)),
        _ => x,
    }
}

fn via_conic(via_x: f64, via_y: f64) -> (f64, f64, f64) {
    let dx = via_x - 0.5;
    let dy = via_y - 0.5;
    let edge = 0.999;
    let max_weight = 1e6;

    let need_x = if dx > 0.0 {
        dx / (edge - via_x)
    } else if dx < 0.0 {
        -dx / (via_x - (1.0 - edge))
    } else {
        0.0
    };

    let need_y = if dy > 0.0 {
        dy / (edge - via_y)
    } else if dy < 0.0 {
        -dy / (via_y - (1.0 - edge))
    } else {
        0.0
    };

    let w = (1.0_f64).max(need_x).max(need_y).min(max_weight);
    (via_x + dx / w, via_y + dy / w, w)
}

fn conic_param(a: f64, b: f64, c: f64) -> f64 {
    if a.abs() < 1e-12 {
        return if b.abs() < 1e-12 { c } else { -c / b };
    }
    let discriminant = (b * b - 4.0 * a * c).max(0.0);
    let root = discriminant.sqrt();
    let first = (-b + root) / (2.0 * a);
    let second = (-b - root) / (2.0 * a);
    let on_arc = |t: f64| (-1e-9..=1.0 + 1e-9).contains(&t);

    if on_arc(first) {
        first.clamp(0.0, 1.0)
    } else if on_arc(second) {
        second.clamp(0.0, 1.0)
    } else {
        c
    }
}

pub fn shape_via(x: f64, via_x: f64, via_y: f64) -> f64 {
    let (cx, cy, w) = via_conic(via_x, via_y);
    let spread = 2.0 - 2.0 * w;
    let a = x * spread - 1.0 + 2.0 * w * cx;
    let b = -x * spread - 2.0 * w * cx;
    let t = conic_param(a, b, x);
    let rest = 1.0 - t;
    let denominator = rest * rest + 2.0 * w * t * rest + t * t;
    if denominator <= 0.0 || !denominator.is_finite() {
        return x;
    }
    (2.0 * w * cy * t * rest + t * t) / denominator
}

pub fn shape_progress(x: f64, point: &HfAutomationPoint) -> f64 {
    if let (Some(vx), Some(vy)) = (point.via_x, point.via_y) {
        if (vx - vy).abs() >= 1e-6 {
            return shape_via(x.clamp(0.0, 1.0), vx, vy);
        }
    }
    apply_curve(x, point.curve)
}

pub fn lerp_value(a: f64, b: f64, x: f64, scale: InterpolationScale) -> f64 {
    if scale == InterpolationScale::Log && a > 0.0 && b > 0.0 {
        (a.ln() + (b.ln() - a.ln()) * x).exp()
    } else {
        a + (b - a) * x
    }
}

/// Sample an automation lane at clip-local time `t`.
pub fn sample_automation_lane(lane: &HfAutomationLane, t: f64, scale: InterpolationScale) -> f64 {
    let pts = &lane.points;
    if pts.is_empty() {
        return 0.0;
    }
    let first = &pts[0];
    if t <= first.t {
        return first.v;
    }
    let last = &pts[pts.len() - 1];
    if t >= last.t {
        return last.v;
    }

    let mut lo = 0;
    let mut hi = pts.len() - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) >> 1;
        if pts[mid].t <= t {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let a = &pts[lo];
    let b = &pts[hi];
    let span = b.t - a.t;
    if span <= 0.0 {
        return b.v;
    }
    let progress = shape_progress((t - a.t) / span, a);
    lerp_value(a.v, b.v, progress, scale)
}

/// Sample an automation lane evenly onto `count` points between `from` and `to`.
pub fn sample_automation_curve(
    lane: &HfAutomationLane,
    from: f64,
    to: f64,
    count: usize,
    scale: InterpolationScale,
) -> Vec<f32> {
    let n = count.max(2);
    let span = to - from;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = from + (span * i as f64) / ((n - 1) as f64);
        out.push(sample_automation_lane(lane, t, scale) as f32);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_constant_lane() {
        let lane = HfAutomationLane::new("volume", vec![HfAutomationPoint::new(0.0, 0.8)]);
        assert_eq!(
            sample_automation_lane(&lane, 0.0, InterpolationScale::Linear),
            0.8
        );
        assert_eq!(
            sample_automation_lane(&lane, 5.0, InterpolationScale::Linear),
            0.8
        );
    }

    #[test]
    fn test_sample_linear_ramp() {
        let lane = HfAutomationLane::new(
            "volume",
            vec![
                HfAutomationPoint::new(0.0, 0.0),
                HfAutomationPoint::new(2.0, 1.0),
            ],
        );
        assert_eq!(
            sample_automation_lane(&lane, 0.0, InterpolationScale::Linear),
            0.0
        );
        assert_eq!(
            sample_automation_lane(&lane, 1.0, InterpolationScale::Linear),
            0.5
        );
        assert_eq!(
            sample_automation_lane(&lane, 2.0, InterpolationScale::Linear),
            1.0
        );
        assert_eq!(
            sample_automation_lane(&lane, 3.0, InterpolationScale::Linear),
            1.0
        );
    }
}
