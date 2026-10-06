use super::{Curve, Seconds};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glide {
    from: f32,
    to: f32,
    start: f32,
    duration: Seconds,
    curve: Curve,
}

impl Glide {
    #[must_use]
    pub fn new(value: f32, duration: Seconds, curve: Curve) -> Self {
        Self {
            from: value,
            to: value,
            start: f32::NEG_INFINITY,
            duration,
            curve,
        }
    }

    #[must_use]
    pub fn sample(&self, now: f32) -> f32 {
        let d = self.duration.get();
        if d <= 0.0 || !(now < self.start + d) {
            return self.to;
        }
        let t = ((now - self.start) / d).clamp(0.0, 1.0);
        self.from + (self.to - self.from) * self.curve.ease(t)
    }

    #[must_use]
    pub fn target(&self) -> f32 {
        self.to
    }

    #[must_use]
    pub fn in_flight(&self, now: f32) -> bool {
        let d = self.duration.get();
        d > 0.0 && now < self.start + d && self.from != self.to
    }

    pub fn retarget(&mut self, to: f32, now: f32) {
        if to == self.to {
            return;
        }
        self.from = self.sample(now);
        self.to = to;
        self.start = now;
    }

    pub fn snap(&mut self, to: f32) {
        self.from = to;
        self.to = to;
        self.start = f32::NEG_INFINITY;
    }

    pub fn set_duration(&mut self, duration: Seconds) {
        self.duration = duration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::{EasingKind, secs};
    use proptest::prelude::*;

    fn glide(v: f32) -> Glide {
        Glide::new(v, secs(0.1), Curve::named(EasingKind::Decelerate))
    }

    #[test]
    fn a_new_glide_rests_at_its_value() {
        let g = glide(3.0);
        assert_eq!(g.sample(0.0), 3.0);
        assert_eq!(g.sample(1e9), 3.0);
        assert!(!g.in_flight(0.0));
    }

    #[test]
    fn retarget_eases_from_the_start_value_to_the_target_and_lands_exactly() {
        let mut g = glide(0.0);
        g.retarget(10.0, 5.0);
        assert_eq!(g.sample(5.0), 0.0);
        let mid = g.sample(5.05);
        assert!(mid > 0.0 && mid < 10.0, "{mid}");
        assert!(g.in_flight(5.05));
        assert_eq!(g.sample(5.1), 10.0);
        assert!(!g.in_flight(5.1));
    }

    #[test]
    fn retargeting_mid_flight_continues_from_where_it_is_without_a_jump() {
        let mut g = glide(0.0);
        g.retarget(10.0, 0.0);
        let here = g.sample(0.04);
        g.retarget(-4.0, 0.04);
        assert_eq!(g.sample(0.04), here);
        assert_eq!(g.sample(0.2), -4.0);
    }

    #[test]
    fn retargeting_to_the_current_target_does_not_restart_the_clock() {
        let mut g = glide(0.0);
        g.retarget(1.0, 0.0);
        g.retarget(1.0, 0.09);
        assert_eq!(g.sample(0.1), 1.0);
    }

    #[test]
    fn snap_and_zero_duration_never_animate() {
        let mut g = glide(0.0);
        g.retarget(5.0, 0.0);
        g.snap(7.0);
        assert_eq!(g.sample(0.01), 7.0);
        assert!(!g.in_flight(0.01));
        let mut z = Glide::new(0.0, secs(0.0), Curve::Linear);
        z.retarget(9.0, 1.0);
        assert_eq!(z.sample(1.0), 9.0);
        assert!(!z.in_flight(1.0));
    }

    proptest! {
        #[test]
        fn a_glide_never_leaves_the_span_of_its_endpoints(
            a in -1e3f32..1e3, b in -1e3f32..1e3, at in 0.0f32..0.2
        ) {
            let mut g = glide(a);
            g.retarget(b, 0.0);
            let v = g.sample(at);
            let (lo, hi) = (a.min(b), a.max(b));
            prop_assert!(v >= lo - 1e-3 && v <= hi + 1e-3, "{v} outside [{lo}, {hi}]");
        }

        #[test]
        fn sampling_is_pure(a in -1e3f32..1e3, b in -1e3f32..1e3, at in 0.0f32..0.2) {
            let mut g = glide(a);
            g.retarget(b, 0.0);
            prop_assert_eq!(g.sample(at), g.sample(at));
        }
    }
}
