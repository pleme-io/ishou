//! `Oscillator` — a periodic phase driven by elapsed time.
//!
//! The oscillator arm of the algebra: a value that repeats forever — a
//! cursor blink, a text-`BLINK`-attribute toggle, a pulsing highlight.
//! Unlike the finite arms (tween, decay) an oscillator never rests; it is
//! a pure function of elapsed time modulo its period.
//!
//! ## Stateless first (the migration target)
//!
//! mado derives the cursor blink from the *global* render clock, not from
//! an accumulated per-cursor timer: `blink_phase_on(elapsed)` with
//! `period = blink_rate_ms/1000·2`. That exact computation is currently
//! open-coded at three sites in `render.rs`. [`blink_on`] is the single
//! primitive those three sites collapse into — a stateless
//! `(elapsed, period) → bool` so there is one blink law, not three
//! hand-kept copies. [`Oscillator`] wraps it with an accumulated clock
//! for callers that own their own time.

use std::time::Duration;

use super::{Advance, Seconds};

/// Whether a blink is in its *on* half at `elapsed_secs`, given the full
/// on-off `period_secs` (on for the first half of each period). A
/// non-positive period is always on (blink disabled). This is the one
/// law the three `render.rs` cursor-blink sites share; it is
/// [`blink_phase`]'s `on`, so a caller that sleeps until the phase's
/// `flips_in` wakes to the flip this answer then shows.
#[must_use]
pub fn blink_on(elapsed_secs: f32, period_secs: f32) -> bool {
    blink_phase(elapsed_secs, period_secs).on
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlinkPhase {
    pub on: bool,
    pub flips_in: Option<Duration>,
}

#[must_use]
pub fn blink_phase(elapsed_secs: f32, period_secs: f32) -> BlinkPhase {
    if period_secs <= 0.0 || !period_secs.is_finite() {
        return BlinkPhase {
            on: true,
            flips_in: None,
        };
    }
    let half = f64::from(period_secs) * 0.5;
    let at = f64::from(elapsed_secs);
    let index = (at / half).floor();
    BlinkPhase {
        on: index.rem_euclid(2.0) == 0.0,
        flips_in: wait_until(elapsed_secs, (index + 1.0) * half),
    }
}

#[must_use]
pub fn wait_until(elapsed_secs: f32, at_secs: f64) -> Option<Duration> {
    let magnitude = elapsed_secs.abs();
    let resolution = f64::from(magnitude.next_up() - magnitude);
    let wait = (at_secs - f64::from(elapsed_secs)).max(resolution);
    Duration::try_from_secs_f64(wait).ok()
}

/// A periodic oscillator with an accumulated clock. `phase_on` gives the
/// square-wave blink; `wave` gives a smooth `[0, 1]` sine for pulsing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oscillator {
    period: Seconds,
    elapsed: Seconds,
}

impl Oscillator {
    /// An oscillator with the given full on-off period, phase 0.
    #[must_use]
    pub fn new(period: Seconds) -> Self {
        Self {
            period,
            elapsed: Seconds::new(0.0),
        }
    }

    /// The square-wave blink state (on for the first half of each period).
    #[must_use]
    pub fn phase_on(&self) -> bool {
        blink_on(self.elapsed.get(), self.period.get())
    }

    /// A smooth `[0, 1]` sine over the period — for a pulse rather than a
    /// hard blink. `0.5` at phase 0, peaks at a quarter period.
    #[must_use]
    pub fn wave(&self) -> f32 {
        let p = self.period.get();
        if p <= 0.0 {
            return 1.0;
        }
        let theta = std::f32::consts::TAU * self.elapsed.get() / p;
        0.5 * theta.sin() + 0.5
    }
}

impl Advance for Oscillator {
    fn advance(&mut self, dt: f32) -> f32 {
        if dt > 0.0 {
            self.elapsed = self.elapsed.inc_by(dt);
        }
        self.value()
    }

    /// The oscillator's scalar reading is its `[0, 1]` sine wave.
    fn value(&self) -> f32 {
        self.wave()
    }

    /// An oscillator is always active — it never comes to rest (a period
    /// of 0 is the degenerate "always on" that also reports active).
    fn is_active(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::secs;
    use proptest::prelude::*;

    #[test]
    fn blink_on_is_on_for_the_first_half() {
        // period 1.0s: on in [0, 0.5), off in [0.5, 1.0).
        assert!(blink_on(0.0, 1.0));
        assert!(blink_on(0.49, 1.0));
        assert!(!blink_on(0.5, 1.0));
        assert!(!blink_on(0.99, 1.0));
        // Wraps: 1.0 is a fresh period start → on again.
        assert!(blink_on(1.0, 1.0));
        assert!(blink_on(1.25, 1.0));
    }

    #[test]
    fn non_positive_period_is_always_on() {
        assert!(blink_on(3.7, 0.0), "period 0 = blink disabled = always on");
    }

    /// BYTE-PIN — `blink_on` reproduces EXACTLY the `(elapsed % period) <
    /// period/2` formula the three `render.rs` cursor/SGR-5 sites open-coded,
    /// across the reachable cursor-blink-rate range (rate > 0). `elapsed` is
    /// always ≥ 0 in the render clock (madori `Instant::duration_since`), so
    /// `%` ≡ `rem_euclid`. (rate == 0 is the one intended divergence — always
    /// on vs the legacy `NaN < 0 == false` — covered by `non_positive_period…`.)
    #[test]
    fn blink_on_matches_the_legacy_render_formula_for_rate_gt_0() {
        let legacy = |elapsed: f32, rate_ms: u32| {
            let period = rate_ms as f32 / 1000.0 * 2.0;
            (elapsed % period) < period / 2.0
        };
        for rate_ms in [1u32, 250, 500, 530, 1000, 2000] {
            let period = rate_ms as f32 / 1000.0 * 2.0;
            for &e in &[0.0f32, 0.001, 0.49, 0.5, 0.53, 0.6, 1.0, 1.06, 3.7, 9.9] {
                assert_eq!(
                    blink_on(e, period),
                    legacy(e, rate_ms),
                    "blink_on diverged from the legacy render formula at rate_ms={rate_ms}, elapsed={e}"
                );
            }
        }
    }

    #[test]
    fn oscillator_dt_zero_is_a_noop() {
        let mut o = Oscillator::new(secs(1.0));
        o.advance(0.3);
        let before = o;
        o.advance(0.0);
        assert_eq!(o, before, "dt=0 must not move the oscillator");
    }

    #[test]
    fn oscillator_tracks_the_stateless_law() {
        // The stateful oscillator's phase must equal the stateless
        // blink_on of the same accumulated elapsed — one law, two surfaces.
        let mut o = Oscillator::new(secs(0.8));
        let mut elapsed = 0.0_f32;
        let dt = 1.0 / 60.0;
        for _ in 0..50 {
            o.advance(dt);
            elapsed += dt;
            assert_eq!(o.phase_on(), blink_on(elapsed, 0.8));
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn clock(start: f64, real: f64) -> f32 {
        (start + real) as f32
    }

    #[allow(clippy::cast_precision_loss)]
    fn flips_seen(start: f64, rate_ms: u32, flips: u32) -> (u32, u32) {
        let period = rate_ms as f32 / 1000.0 * 2.0;
        let mut real = 0.0f64;
        let mut shown = blink_on(clock(start, real), period);
        let mut seen = 0;
        let mut wakes = 0;
        while seen < flips {
            let phase = blink_phase(clock(start, real), period);
            assert_eq!(phase.on, shown, "the phase changed without a wake");
            real += phase
                .flips_in
                .expect("a blinking period flips")
                .as_secs_f64();
            wakes += 1;
            assert!(wakes < flips * 4, "{wakes} wakes for {seen} flips");
            let now = blink_on(clock(start, real), period);
            if now != shown {
                seen += 1;
                shown = now;
            }
        }
        (seen, wakes)
    }

    #[test]
    fn sleeping_until_the_flip_wakes_to_the_flip_on_a_long_running_clock() {
        for start in [0.0, 8.0 * 3600.0, 7.0 * 86_400.0, 30.0 * 86_400.0] {
            for rate_ms in [333u32, 500, 530, 600, 1000] {
                let (seen, wakes) = flips_seen(start, rate_ms, 100);
                assert_eq!(seen, 100);
                assert!(
                    wakes <= 200,
                    "rate {rate_ms} ms at {start} s: {wakes} wakes for 100 flips"
                );
            }
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn a_wait_is_never_shorter_than_the_clock_can_show() {
        let week = (7.0 * 86_400.0) as f32;
        let step = week.next_up() - week;
        assert!(step >= 0.06, "a week into an f32 clock it steps {step} s");
        let wait = wait_until(week, f64::from(week) + 0.001).unwrap();
        assert!(wait.as_secs_f64() >= f64::from(step));
        assert_eq!(
            wait_until(1.0, 1.25),
            Some(Duration::from_secs_f64(0.25)),
            "a wait longer than the step is exact"
        );
        assert!(
            blink_phase(f32::INFINITY, 1.0).flips_in.is_none(),
            "an unrepresentable wait is None, never a panic"
        );
        assert_eq!(
            blink_phase(3.7, 0.0),
            BlinkPhase {
                on: true,
                flips_in: None
            }
        );
    }

    proptest! {
        /// The stateless blink law is periodic: `blink_on(e)` equals
        /// `blink_on(e + period)` for any elapsed.
        #[test]
        fn blink_is_periodic(e in 0.0f32..10.0, p in 0.05f32..2.0) {
            prop_assert_eq!(blink_on(e, p), blink_on(e + p, p),
                "blink not periodic at e={}, p={}", e, p);
        }

        /// The pulse wave never leaves [0, 1].
        #[test]
        fn wave_stays_in_unit_range(e in 0.0f32..10.0) {
            let mut o = Oscillator::new(secs(0.7));
            o.advance(e);
            let w = o.wave();
            prop_assert!((0.0..=1.0).contains(&w), "wave {w} escaped [0,1]");
        }
    }
}
