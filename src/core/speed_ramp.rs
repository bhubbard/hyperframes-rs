use super::audio_automation::{HfAutomationLane, InterpolationScale, sample_automation_lane};

const CELLS_PER_SEGMENT: usize = 48;

/// Either a constant speed multiplier or an automation lane over clip-local time.
#[derive(Debug, Clone, PartialEq)]
pub enum RateSpec {
    Constant(f64),
    Lane(HfAutomationLane),
}

impl From<f64> for RateSpec {
    fn from(r: f64) -> Self {
        RateSpec::Constant(r)
    }
}

impl From<HfAutomationLane> for RateSpec {
    fn from(l: HfAutomationLane) -> Self {
        RateSpec::Lane(l)
    }
}

/// Piecewise integral table for mapping clip-local time to source media time.
#[derive(Debug, Clone)]
pub struct RateTable {
    ts: Vec<f64>,
    ss: Vec<f64>,
    first_rate: f64,
    last_rate: f64,
}

impl RateTable {
    pub fn build(lane: &HfAutomationLane) -> Self {
        let pts = &lane.points;
        if pts.is_empty() {
            return Self {
                ts: vec![0.0],
                ss: vec![0.0],
                first_rate: 1.0,
                last_rate: 1.0,
            };
        }

        let first_rate = pts[0].v;
        let last_rate = pts[pts.len() - 1].v;
        let mut ts = vec![0.0];
        let mut ss = vec![0.0];

        let push = |t: f64, ts: &mut Vec<f64>, ss: &mut Vec<f64>| {
            let prev_t = *ts.last().unwrap();
            if t <= prev_t {
                return;
            }
            let prev_rate = sample_automation_lane(lane, prev_t, InterpolationScale::Log);
            let curr_rate = sample_automation_lane(lane, t, InterpolationScale::Log);
            let dt = t - prev_t;
            let avg_rate = (prev_rate + curr_rate) / 2.0;
            let prev_s = *ss.last().unwrap();
            ts.push(t);
            ss.push(prev_s + avg_rate * dt);
        };

        let first_t = pts[0].t.max(0.0);
        push(first_t, &mut ts, &mut ss);

        for i in 1..pts.len() {
            let a = pts[i - 1].t;
            let b = pts[i].t;
            if b <= 0.0 {
                continue;
            }
            for k in 1..=CELLS_PER_SEGMENT {
                let t = a.max(0.0) + ((b - a) * k as f64) / (CELLS_PER_SEGMENT as f64);
                push(t, &mut ts, &mut ss);
            }
        }

        Self {
            ts,
            ss,
            first_rate,
            last_rate,
        }
    }

    /// Source seconds consumed by clip-local time `t` (integral of rate).
    pub fn source_time_at(&self, t: f64) -> f64 {
        if t <= 0.0 {
            return t * self.first_rate;
        }
        let last_t = *self.ts.last().unwrap_or(&0.0);
        if t >= last_t {
            let last_s = *self.ss.last().unwrap_or(&0.0);
            return last_s + (t - last_t) * self.last_rate;
        }

        interpolate(&self.ts, &self.ss, t)
    }

    /// Clip-local time that reaches source media time `s`.
    pub fn time_at_source_time(&self, s: f64) -> f64 {
        if s <= 0.0 {
            return if self.first_rate > 0.0 {
                s / self.first_rate
            } else {
                0.0
            };
        }
        let last_s = *self.ss.last().unwrap_or(&0.0);
        if s >= last_s {
            let last_t = *self.ts.last().unwrap_or(&0.0);
            return if self.last_rate > 0.0 {
                last_t + (s - last_s) / self.last_rate
            } else {
                last_t
            };
        }

        interpolate(&self.ss, &self.ts, s)
    }
}

fn interpolate(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    if xs.len() == 1 {
        return ys[0];
    }
    let mut lo = 0;
    let mut hi = xs.len() - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) >> 1;
        if xs[mid] <= x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let x_span = xs[hi] - xs[lo];
    if x_span <= 0.0 {
        return ys[lo];
    }
    ys[lo] + ((ys[hi] - ys[lo]) * (x - xs[lo])) / x_span
}

/// Instantaneous speed multiplier at clip-local time `t`.
pub fn rate_at(spec: &RateSpec, t: f64) -> f64 {
    match spec {
        RateSpec::Constant(r) => *r,
        RateSpec::Lane(lane) => {
            if lane.points.is_empty() {
                1.0
            } else {
                sample_automation_lane(lane, t, InterpolationScale::Log)
            }
        }
    }
}

/// Source seconds consumed by clip-local time `t`.
pub fn source_time_at(spec: &RateSpec, t: f64) -> f64 {
    match spec {
        RateSpec::Constant(r) => t * r,
        RateSpec::Lane(lane) => {
            let table = RateTable::build(lane);
            table.source_time_at(t)
        }
    }
}

/// Clip-local time that reaches source media time `s`.
pub fn time_at_source_time(spec: &RateSpec, s: f64) -> f64 {
    match spec {
        RateSpec::Constant(r) => {
            if *r > 0.0 {
                s / r
            } else {
                0.0
            }
        }
        RateSpec::Lane(lane) => {
            let table = RateTable::build(lane);
            table.time_at_source_time(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::audio_automation::HfAutomationPoint;

    #[test]
    fn test_constant_rate_spec() {
        let spec = RateSpec::Constant(2.0);
        assert_eq!(rate_at(&spec, 3.0), 2.0);
        assert_eq!(source_time_at(&spec, 3.0), 6.0);
        assert_eq!(time_at_source_time(&spec, 6.0), 3.0);
    }

    #[test]
    fn test_ramp_integral() {
        let lane = HfAutomationLane::new(
            "rate",
            vec![
                HfAutomationPoint::new(0.0, 1.0),
                HfAutomationPoint::new(2.0, 2.0),
            ],
        );
        let spec = RateSpec::Lane(lane);
        let s2 = source_time_at(&spec, 2.0);
        // Log-scale rate 1.0 -> 2.0 over 2s has exact integral 2/ln(2) ≈ 2.8854
        let expected = 2.0 / 2.0_f64.ln();
        assert!((s2 - expected).abs() < 0.01);
    }
}
