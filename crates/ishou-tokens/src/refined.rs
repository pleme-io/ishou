//! `Refined<T, B>` — typed values with statically-known bounds.
//!
//! # Why this exists
//!
//! Every pleme-io GUI app has values where "raw `T`" is the wrong
//! type because not every value of `T` is valid:
//!
//! - `font_size: f32` — must be in `[6.0, 64.0]`
//! - `padding: u32` — must be in `[0, 256]`
//! - `opacity: f32` — must be in `[0.0, 1.0]`
//! - `scrollback_lines: usize` — must be in `[100, 10_000_000]`
//! - `blink_rate_ms: u32` — must be in `[50, 5000]`
//! - `font_scale: f32` — must be in `[0.5, 4.0]`
//!
//! Today each app hand-clamps these at every call site:
//!
//! ```ignore
//! let new = (renderer.font_size() + 1.0).min(64.0);   // mado FontIncrease
//! let new = (cfg.padding + 4).min(256);               // mado pad-bump
//! ```
//!
//! Three real problems:
//!
//! 1. **Bounds checking is opt-in.** Any future caller that forgets
//!    to clamp can put the renderer into a state nobody expects.
//! 2. **The bound is duplicated everywhere.** Bumping mado's
//!    FONT_MAX from 64 → 96 means hunting every `.min(64.0)`.
//! 3. **Per-app invariants don't compose.** Two GUI apps that both
//!    want `font_size ∈ [6, 64]` write the same code twice.
//!
//! `Refined<T, B>` solves all three: the bound is encoded as a
//! type parameter, every mutation goes through `Refined::new`
//! which clamps, and consumers share the bound by sharing the
//! `B: Bounds<T>` impl.
//!
//! # Shape
//!
//! ```rust
//! use ishou_tokens::{Refined, Bounds};
//!
//! pub struct FontSizeBounds;
//! impl Bounds<f32> for FontSizeBounds {
//!     fn min() -> f32 { 6.0 }
//!     fn max() -> f32 { 64.0 }
//!     fn default() -> f32 { 14.0 }
//! }
//!
//! pub type BoundedFontSize = Refined<f32, FontSizeBounds>;
//!
//! let s = BoundedFontSize::new(14.0);
//! let s = s.inc_by(1.0);    // clamps at MAX
//! let s = s.dec_by(1.0);    // clamps at MIN
//! ```
//!
//! # Sharing bounds across apps
//!
//! Apps that should agree on a bound (every mado-class terminal
//! uses `[6.0, 64.0]` for font size) export ONE `Bounds` impl
//! and every app pulls it in:
//!
//! ```ignore
//! // ishou-tokens or a downstream "fleet UI bounds" crate
//! pub struct TerminalFontSizeBounds;
//! impl Bounds<f32> for TerminalFontSizeBounds { ... }
//!
//! // mado, namimado, escriba — all use:
//! type BoundedFontSize = Refined<f32, TerminalFontSizeBounds>;
//! ```
//!
//! Changing the fleet bound is one source touch; every consumer
//! tracks on next compile. Pillar 12.
//!
//! # Historical
//!
//! Extracted from `mado/src/font_size.rs` (2026-05-21, mado@68d74c0)
//! after the runaway-font incident. The mado-local `BoundedFontSize`
//! becomes a one-line typealias over this primitive next.

use std::fmt;
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize};

/// Statically-known bounds for a refined type. Implementors are
/// zero-sized marker types — they exist only as type parameters
/// to `Refined<T, B>`.
///
/// Functions (not consts) because Rust's const evaluation doesn't
/// support f32 arithmetic in all contexts. The compiler still
/// monomorphizes these at zero runtime cost.
pub trait Bounds<T> {
    /// Lower bound (inclusive).
    fn min() -> T;
    /// Upper bound (inclusive).
    fn max() -> T;
    /// Default value. Should satisfy `min() <= default() <= max()`.
    fn default() -> T;
}

/// A value of type `T` proven (by type) to satisfy `B::min() <= value <= B::max()`.
///
/// `Copy` + `PartialEq` are derived when `T: Copy + PartialEq`,
/// so consumers can pass `Refined<f32, _>` around like an `f32`.
pub struct Refined<T, B: Bounds<T>> {
    value: T,
    _bounds: PhantomData<B>,
}

// Manual derives that don't propagate the B's Clone/Copy bound
// (PhantomData is always Copy + Clone, but the derives would
// require B: Copy + Clone unnecessarily).

impl<T: Copy, B: Bounds<T>> Copy for Refined<T, B> {}
impl<T: Clone, B: Bounds<T>> Clone for Refined<T, B> {
    fn clone(&self) -> Self {
        Self { value: self.value.clone(), _bounds: PhantomData }
    }
}
impl<T: PartialEq, B: Bounds<T>> PartialEq for Refined<T, B> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: fmt::Debug, B: Bounds<T>> fmt::Debug for Refined<T, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Don't print the PhantomData — just the value.
        f.debug_tuple("Refined").field(&self.value).finish()
    }
}
impl<T: fmt::Display, B: Bounds<T>> fmt::Display for Refined<T, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: Copy + PartialOrd, B: Bounds<T>> Refined<T, B> {
    /// Construct from an arbitrary `T`. Always succeeds — out-of-range
    /// values are clamped to `B::min()` or `B::max()`, and values that are
    /// not comparable to the bounds at all (see below) become
    /// `B::default()`.
    ///
    /// Use [`Self::try_new`] when you want to detect clamping.
    ///
    /// ## Why this tests for IN-range rather than OUT-of-range
    ///
    /// `T: PartialOrd` is a **partial** order: two values may be
    /// incomparable, in which case `<`, `>`, `<=` and `>=` are ALL false.
    /// `f32::NAN` is the everyday instance.
    ///
    /// This used to read `if value < B::min() … else if value > B::max() …
    /// else { value }`, which let every incomparable value fall through the
    /// `else` **unrefined**. `Refined::<f32, _>::new(f32::NAN)` returned
    /// `NaN`, and [`Self::get`]'s promise that the result "always satisfies
    /// `B::min() <= v <= B::max()`" was false for it — a refinement type
    /// that silently passed through the one value most likely to poison
    /// arithmetic downstream.
    ///
    /// Requiring positive proof of in-range fixes the whole class rather
    /// than special-casing floats: no `is_finite`, no `T: Float` bound. For
    /// a total order (every integer type) the incomparable arm is simply
    /// unreachable.
    #[must_use]
    pub fn new(value: T) -> Self {
        let refined = if value >= B::min() && value <= B::max() {
            value
        } else if value < B::min() {
            B::min()
        } else if value > B::max() {
            B::max()
        } else {
            // Incomparable to both bounds — NaN and friends. There is no
            // meaningful clamp for a value that is not on the order at all,
            // so take the declared default.
            B::default()
        };
        Self { value: refined, _bounds: PhantomData }
    }

    /// Construct from a `T`, returning `Err(value)` when the input is not
    /// already within bounds.
    ///
    /// # Errors
    /// Returns the original value when it is out of range **or not
    /// comparable to the bounds** (e.g. `f32::NAN`) — see [`Self::new`] for
    /// why those are the same case.
    pub fn try_new(value: T) -> Result<Self, T> {
        if value >= B::min() && value <= B::max() {
            Ok(Self { value, _bounds: PhantomData })
        } else {
            Err(value)
        }
    }

    /// The wrapped value. Always satisfies `B::min() <= v <= B::max()`.
    #[must_use]
    pub fn get(self) -> T {
        self.value
    }

    /// `true` when the value is at the upper bound.
    #[must_use]
    pub fn at_max(self) -> bool {
        !(self.value < B::max())
    }

    /// `true` when the value is at the lower bound.
    #[must_use]
    pub fn at_min(self) -> bool {
        !(self.value > B::min())
    }
}

impl<T, B: Bounds<T>> Default for Refined<T, B> {
    /// The default value provided by `B::default()`. Wrapped in
    /// `Refined` without bounds re-validation (the trait contract
    /// requires `min() <= default() <= max()`).
    fn default() -> Self {
        Self { value: B::default(), _bounds: PhantomData }
    }
}

// ── Arithmetic mutations for numeric types ───────────────────────
//
// These are gated on `T: Copy + PartialOrd + std::ops::Add<Output=T>`
// etc. so they're available for f32 / u32 / i64 / usize but not for
// e.g. `String` (which makes no sense for Refined anyway).

impl<T, B: Bounds<T>> Refined<T, B>
where
    T: Copy + PartialOrd + std::ops::Add<Output = T>,
{
    /// Add `delta`. Saturates at `B::max()`.
    #[must_use]
    pub fn inc_by(self, delta: T) -> Self {
        Self::new(self.value + delta)
    }
}

impl<T, B: Bounds<T>> Refined<T, B>
where
    T: Copy + PartialOrd + std::ops::Sub<Output = T>,
{
    /// Subtract `delta`. Saturates at `B::min()`.
    #[must_use]
    pub fn dec_by(self, delta: T) -> Self {
        Self::new(self.value - delta)
    }
}

// ── serde — round-trip through clamping ──────────────────────────

impl<T: Serialize, B: Bounds<T>> Serialize for Refined<T, B> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(s)
    }
}

impl<'de, T, B> Deserialize<'de> for Refined<T, B>
where
    T: Copy + PartialOrd + Deserialize<'de>,
    B: Bounds<T>,
{
    /// Round-trips through clamping. Operator YAML
    /// `font_size: 9999.0` deserializes to `B::max()`, not a runtime
    /// panic.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = T::deserialize(d)?;
        Ok(Self::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Fixture bounds for tests ────────────────────────────────

    struct PercentBounds;
    impl Bounds<f32> for PercentBounds {
        fn min() -> f32 { 0.0 }
        fn max() -> f32 { 1.0 }
        fn default() -> f32 { 0.5 }
    }
    type Percent = Refined<f32, PercentBounds>;

    struct PortBounds;
    impl Bounds<u32> for PortBounds {
        fn min() -> u32 { 1024 }
        fn max() -> u32 { 65535 }
        fn default() -> u32 { 8080 }
    }
    type Port = Refined<u32, PortBounds>;

    // ── The incomparable-value rows ─────────────────────────────
    //
    // `PartialOrd` is PARTIAL: for `f32::NAN` every one of `<`, `>`, `<=`,
    // `>=` is false. The original `new` tested for OUT-of-range and let
    // anything that answered "no" to both tests through unrefined, so
    // `Refined::<f32,_>::new(NAN)` returned NaN and `get`'s documented
    // promise was false for it.
    //
    // These rows go red against that implementation.

    #[test]
    fn new_refines_nan_to_the_declared_default() {
        let p = Percent::new(f32::NAN);
        assert!(
            !p.get().is_nan(),
            "a refinement type must not pass NaN through — get() promises \
             min() <= v <= max(), which no NaN satisfies"
        );
        assert_eq!(p.get(), PercentBounds::default());
    }

    #[test]
    fn new_clamps_the_infinities() {
        // These ARE comparable, so they clamp normally rather than
        // taking the default — pinned so the new in-range test does not
        // quietly reroute them.
        assert_eq!(Percent::new(f32::INFINITY).get(), 1.0);
        assert_eq!(Percent::new(f32::NEG_INFINITY).get(), 0.0);
    }

    #[test]
    fn try_new_rejects_nan_rather_than_returning_ok() {
        assert!(
            Percent::try_new(f32::NAN).is_err(),
            "try_new's contract is 'Err when not already in bounds', and \
             NaN is never in bounds"
        );
    }

    #[test]
    fn every_constructed_value_satisfies_the_bounds_promise() {
        // The invariant `get()` advertises, over the values most likely to
        // break it.
        for v in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            -1e30,
            1e30,
            -0.0,
            0.5,
        ] {
            let got = Percent::new(v).get();
            assert!(
                got >= PercentBounds::min() && got <= PercentBounds::max(),
                "new({v}) produced {got}, which violates the bounds promise"
            );
        }
    }

    // ── Construction + clamping ─────────────────────────────────

    #[test]
    fn new_clamps_below_min() {
        assert_eq!(Percent::new(-0.5).get(), 0.0);
        assert_eq!(Port::new(80).get(), 1024);
    }

    #[test]
    fn new_clamps_above_max() {
        assert_eq!(Percent::new(2.0).get(), 1.0);
        assert_eq!(Port::new(99_999).get(), 65535);
    }

    #[test]
    fn new_passes_in_range() {
        assert_eq!(Percent::new(0.3).get(), 0.3);
        assert_eq!(Port::new(8080).get(), 8080);
    }

    #[test]
    fn try_new_errors_on_oob() {
        assert!(Percent::try_new(-0.5).is_err());
        assert!(Port::try_new(80).is_err());
    }

    #[test]
    fn try_new_succeeds_in_range() {
        assert!(Percent::try_new(0.5).is_ok());
        assert!(Port::try_new(8080).is_ok());
    }

    // ── Default ─────────────────────────────────────────────────

    #[test]
    fn default_matches_bounds_default() {
        assert_eq!(Percent::default().get(), 0.5);
        assert_eq!(Port::default().get(), 8080);
    }

    // ── Arithmetic + saturation ─────────────────────────────────

    #[test]
    fn inc_by_saturates_at_max() {
        let mut p = Percent::new(0.5);
        for _ in 0..1000 { p = p.inc_by(0.1); }
        assert_eq!(p.get(), 1.0);
        assert!(p.at_max());
    }

    #[test]
    fn dec_by_saturates_at_min() {
        let mut p = Percent::new(0.5);
        for _ in 0..1000 { p = p.dec_by(0.1); }
        assert_eq!(p.get(), 0.0);
        assert!(p.at_min());
    }

    #[test]
    fn inc_dec_inverse_in_safe_range() {
        let p = Percent::new(0.5);
        assert_eq!(p.inc_by(0.1).dec_by(0.1).get(), 0.5);
    }

    // ── Predicates ──────────────────────────────────────────────

    #[test]
    fn at_max_at_min_predicates() {
        assert!(Percent::new(1.0).at_max());
        assert!(Percent::new(0.0).at_min());
        assert!(!Percent::new(0.5).at_max());
        assert!(!Percent::new(0.5).at_min());
    }

    // ── serde round-trip ────────────────────────────────────────

    #[test]
    fn serde_round_trip_clamps_oob() {
        let yaml = "9999.0";
        let parsed: Percent = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.get(), 1.0);
    }

    #[test]
    fn serde_round_trip_preserves_in_range() {
        let p = Percent::new(0.75);
        let s = serde_yaml::to_string(&p).unwrap();
        let back: Percent = serde_yaml::from_str(&s).unwrap();
        assert_eq!(back, p);
    }

    // ── Derived impls (manual) ──────────────────────────────────

    #[test]
    fn copy_clone_roundtrip() {
        let p = Percent::new(0.42);
        let q = p; // Copy
        assert_eq!(p, q);
        let r = p.clone(); // Clone
        assert_eq!(p, r);
    }

    #[test]
    fn debug_does_not_leak_phantom_data() {
        let p = Percent::new(0.42);
        let s = format!("{:?}", p);
        assert!(s.contains("0.42"));
        assert!(!s.contains("PhantomData"));
    }

    #[test]
    fn display_renders_inner_value_only() {
        let p = Percent::new(0.42);
        assert_eq!(format!("{}", p), "0.42");
    }

    // ── Cross-app sharing — proves a fleet bound works ──────────

    #[test]
    fn two_typealiases_over_same_bounds_are_one_type() {
        // If three apps all do `type X = Refined<f32, PercentBounds>;`
        // they get the SAME type — values are interchangeable.
        type AppA = Refined<f32, PercentBounds>;
        type AppB = Refined<f32, PercentBounds>;
        let a: AppA = Refined::new(0.3);
        let b: AppB = a; // typecheck passes
        assert_eq!(b.get(), 0.3);
    }
}
