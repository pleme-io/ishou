//! Motion — the fleet's animation vocabulary AND its evaluator.
//!
//! ## Why both halves live here
//!
//! This module was tokens only: `Cubic`, `Easings`, `Durations`, `Motion` —
//! a curve *vocabulary* with nothing able to sample it. The evaluator that
//! turns those tokens into moving scalars was written in mado
//! (`mado/src/motion/`), whose own module doc named this file as its
//! destination: *"lift this evaluator up into `ishou_tokens::motion` so
//! egaku / quadro / tela / garasu share one motion evaluator … extraction
//! lands at the 3rd consumer."*
//!
//! ★ **It landed at the SECOND consumer, and the reason is structural, not
//! impatience.** mado is an application. A second consumer cannot depend on
//! it — tobira is a launcher; pulling in a terminal emulator to obtain a
//! cubic-bézier sampler is not a dependency, it is a joke. So the choice at
//! consumer two was never "extract now or extract later", it was **extract
//! or copy**, and copying an animation algebra into a second app is the
//! duplication the extraction exists to prevent. The 3rd-consumer rule
//! assumes the 1st consumer is reachable; here it was not.
//!
//! The tokens and the evaluator belong together for the same reason: a
//! `Cubic` nobody can sample is data with no meaning, and a sampler with no
//! named curves is an algorithm with no vocabulary.
//!
//! ## The algebra
//!
//! Every animated scalar is **one pure function of `(typed declaration,
//! elapsed time)`**, never an imperative per-frame `-= 1`. An animation is
//! *data* (a [`Tween`], a [`Decay`], an [`Oscillator`]); a frame is the fold
//! [`Advance::advance`] applies to it. How a value moves lives in the
//! value's type, not scattered across a render loop.
//!
//! ## The determinism contract (load-bearing)
//!
//! Every [`Advance::advance`] MUST be a strict no-op when `dt <= 0.0`: it
//! returns the current value and mutates nothing. Headless determinism
//! ladders render at `elapsed = 0` / `dt = 0` and assert byte-identical
//! frame hashes; an arm that moved at `dt == 0` would break that
//! byte-stability.
//!
//! ## Bounds — tier-honest: only-mitigated, NOT unrepresentable
//!
//! A duration is a [`Seconds`] and normalized progress a [`Unit`], both
//! [`crate::Refined`]. Being precise: this is a **runtime clamp**, not a
//! compile-time refusal. `Seconds::new(-5.0)` is a legal call that succeeds
//! by clamping to 0, and `Refined::default()` can bypass the bound. Per
//! `theory/UNREPRESENTABILITY.md` §III.3, f32-backed `Refined` is graded
//! **only-mitigated** — const generics preclude compile-time f32 bounds
//! today. The honest claim is "an out-of-range value *saturates* at the
//! boundary and never flows into a curve as, say, `1.3`", NOT "illegal
//! motion is unrepresentable."

use crate::{Bounds, Refined};

mod tokens;
pub use tokens::{Cubic, Durations, Easings, Motion};

pub mod curve;
pub mod decay;
pub mod oscillator;
pub mod tween;

pub use curve::{Curve, EasingKind};
pub use decay::{Decay, frame_decay};
pub use oscillator::{Oscillator, blink_on};
pub use tween::Tween;

/// The one contract every CPU motion arm satisfies: a pure step of an
/// animated scalar by `dt` seconds.
///
/// The three methods form a tiny algebra: [`value`](Advance::value)
/// reads the current scalar without moving, [`advance`](Advance::advance)
/// steps and returns the new scalar, and
/// [`is_active`](Advance::is_active) reports whether motion is still
/// pending so the caller can skip finished animations entirely.
///
/// **Determinism:** `advance(dt)` is a strict no-op for `dt <= 0.0`.
pub trait Advance {
    /// Step forward by `dt` seconds and return the new scalar.
    /// A no-op (returns [`value`](Advance::value), mutates nothing)
    /// when `dt <= 0.0`.
    fn advance(&mut self, dt: f32) -> f32;

    /// The current scalar, without stepping.
    fn value(&self) -> f32;

    /// Whether the animation still has motion pending. A finished
    /// animation rests at its terminal value and reports `false`, so
    /// the render loop can drop it from the active set.
    fn is_active(&self) -> bool;
}

// ── Typed bounds — illegal motion is unrepresentable ────────────────

/// Bounds marker: a non-negative duration in seconds. `min = 0` makes a
/// negative duration unconstructible (`Refined::new` clamps it to 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonNegSecs;

impl Bounds<f32> for NonNegSecs {
    fn min() -> f32 {
        0.0
    }
    fn max() -> f32 {
        f32::MAX
    }
    fn default() -> f32 {
        0.0
    }
}

/// A duration in seconds, clamped `>= 0` at construction (only-mitigated
/// — the clamp is a runtime saturation, not a compile-time refusal; see
/// the module "Bounds" note).
pub type Seconds = Refined<f32, NonNegSecs>;

/// Bounds marker: the closed unit interval `[0, 1]`. Progress, alpha,
/// and normalized intensity all live here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitBounds;

impl Bounds<f32> for UnitBounds {
    fn min() -> f32 {
        0.0
    }
    fn max() -> f32 {
        1.0
    }
    fn default() -> f32 {
        0.0
    }
}

/// A scalar clamped to lie in `[0, 1]` — normalized progress or
/// intensity. An out-of-range value saturates at the boundary (a runtime
/// clamp, only-mitigated), so it can never flow into a curve as, say,
/// `1.3`.
pub type Unit = Refined<f32, UnitBounds>;

/// Convenience: build a [`Seconds`] from an `f32` (clamps `< 0` to 0).
#[must_use]
pub fn secs(v: f32) -> Seconds {
    Seconds::new(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_clamps_negative_to_zero() {
        // Only-mitigated, not unrepresentable: the call succeeds by
        // clamping, it does not refuse.
        assert_eq!(secs(-5.0).get(), 0.0, "a negative duration clamps to zero");
        assert_eq!(secs(0.25).get(), 0.25);
    }

    #[test]
    fn unit_saturates_outside_the_interval() {
        assert_eq!(Unit::new(1.3).get(), 1.0);
        assert_eq!(Unit::new(-0.2).get(), 0.0);
        assert_eq!(Unit::new(0.5).get(), 0.5);
    }
}
