//! The capability ladder — the axis pente does not have.
//!
//! Every face can paint a colour, so pente needs no notion of a face being
//! unable to honour a binding. Faces genuinely differ in what MOTION they can
//! express: a GPU surface interpolates a scroll offset continuously; a
//! terminal cell-grid cannot move by a third of a cell; a rendered Nix file
//! cannot animate at all.
//!
//! An algebra that ignores this produces the worst failure available — an
//! authored smooth-scroll that *silently degrades* to a jump on the TUI, so
//! the author believes the fleet is uniform when it is not. Silent divergence
//! is exactly what the visual spine exists to end.
//!
//! So the ladder is a PHANTOM TYPESTATE, not a runtime check: an animation
//! demanding more than a face can express has **no apply path**. This is the
//! `AtLeast<Rung>` construction already proven in `ayatsuri/src/kabe` for the
//! macOS compositor privilege ladder, one domain over.

use serde::{Deserialize, Serialize};

/// What a surface can express, ordered.
///
/// `Static < Discrete < CellQuantized < Continuous`.
///
/// TIER-HONEST: this membership is a DESIGN PROPOSAL derived from four
/// surfaces (nix render, SGR blink, TUI scroll, GPU ease), not a measured
/// taxonomy. A fifth surface may split a rung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MotionClass {
    /// Cannot animate at all — a rendered file, a committed Nix artifact.
    Static,
    /// On/off at a coarse tick. SGR-5 blink, a bell flash.
    Discrete,
    /// Moves, but quantised to a cell and a frame. TUI scroll.
    CellQuantized,
    /// Sub-cell interpolation at display refresh. GPU.
    Continuous,
}

// ── The typestate ladder ────────────────────────────────────────────
//
// Each rung is a zero-sized type. `AtLeast<R>` is implemented ONLY for the
// rungs that genuinely dominate `R`, so the bound is not satisfiable by a
// weaker face and the failure is E0277 at the call site.

/// Rung marker: cannot animate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Static;
/// Rung marker: on/off at a coarse tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Discrete;
/// Rung marker: cell- and frame-quantised movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellQuantized;
/// Rung marker: sub-cell interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Continuous;

/// A rung of the ladder, with its runtime reflection.
pub trait Rung: Copy {
    const CLASS: MotionClass;
}

impl Rung for Static {
    const CLASS: MotionClass = MotionClass::Static;
}
impl Rung for Discrete {
    const CLASS: MotionClass = MotionClass::Discrete;
}
impl Rung for CellQuantized {
    const CLASS: MotionClass = MotionClass::CellQuantized;
}
impl Rung for Continuous {
    const CLASS: MotionClass = MotionClass::Continuous;
}

/// `Self` can express everything `R` requires.
///
/// Read `W: AtLeast<Discrete>` as "the face W is at least Discrete". There is
/// deliberately NO blanket impl and NO `impl AtLeast<Continuous> for
/// CellQuantized`, so demanding continuous motion from a TUI face is a
/// COMPILE ERROR (E0277), never a runtime downgrade.
///
/// # The negative proof
///
/// This is the doctrine's headline claim, and it cannot be written as a
/// runtime assertion — the whole property is that the program does not
/// compile. `compile_fail` doctests are the only executable form.
///
/// VACUITY. A `compile_fail` doctest passes if the snippet fails for ANY
/// reason, so a stale one silently stops testing what it names. Each
/// negative below is therefore paired with a positive that MUST compile: a
/// rename breaks the pair loudly instead of quietly turning the negative
/// into a tautology.
///
/// A stronger face satisfies a weaker demand — must compile:
/// ```
/// use ishou_batente::capability::{AtLeast, CellQuantized, Continuous, Rung};
/// fn demands<R: Rung, W: AtLeast<R>>(_face: W) {}
/// demands::<CellQuantized, _>(Continuous);
/// ```
///
/// A terminal cannot smooth-scroll — must NOT compile:
/// ```compile_fail
/// use ishou_batente::capability::{AtLeast, CellQuantized, Continuous, Rung};
/// fn demands<R: Rung, W: AtLeast<R>>(_face: W) {}
/// demands::<Continuous, _>(CellQuantized);
/// ```
///
/// A rendered Nix file cannot blink — must NOT compile:
/// ```compile_fail
/// use ishou_batente::capability::{AtLeast, Discrete, Rung, Static};
/// fn demands<R: Rung, W: AtLeast<R>>(_face: W) {}
/// demands::<Discrete, _>(Static);
/// ```
///
/// The ladder is not accidentally symmetric — must NOT compile:
/// ```compile_fail
/// use ishou_batente::capability::{AtLeast, CellQuantized, Discrete, Rung};
/// fn demands<R: Rung, W: AtLeast<R>>(_face: W) {}
/// demands::<CellQuantized, _>(Discrete);
/// ```
pub trait AtLeast<R: Rung>: Rung {}

// Reflexive.
impl AtLeast<Static> for Static {}
impl AtLeast<Discrete> for Discrete {}
impl AtLeast<CellQuantized> for CellQuantized {}
impl AtLeast<Continuous> for Continuous {}

// Upward only. Each row says "this rung also satisfies every weaker demand".
impl AtLeast<Static> for Discrete {}
impl AtLeast<Static> for CellQuantized {}
impl AtLeast<Static> for Continuous {}

impl AtLeast<Discrete> for CellQuantized {}
impl AtLeast<Discrete> for Continuous {}

impl AtLeast<CellQuantized> for Continuous {}

#[cfg(test)]
mod tests {
    use super::*;

    // A demand expressed as a bound. The whole point is that this function
    // cannot be called with a face weaker than `R`.
    fn demands<R: Rung, W: AtLeast<R>>(_face: W) -> MotionClass {
        W::CLASS
    }

    #[test]
    fn a_stronger_face_satisfies_a_weaker_demand() {
        assert_eq!(demands::<Discrete, _>(Continuous), MotionClass::Continuous);
        assert_eq!(demands::<Static, _>(CellQuantized), MotionClass::CellQuantized);
        assert_eq!(demands::<CellQuantized, _>(Continuous), MotionClass::Continuous);
    }

    #[test]
    fn reflexive_holds() {
        assert_eq!(demands::<Continuous, _>(Continuous), MotionClass::Continuous);
        assert_eq!(demands::<Static, _>(Static), MotionClass::Static);
    }

    // The NEGATIVE case is load-bearing and cannot be a runtime assertion —
    // it is a compile error. It lives as `compile_fail` DOCTESTS on the
    // `AtLeast` trait above, because doctests run only on the lib target: a
    // `compile_fail` block in a `//` comment (as this one used to be) or in a
    // tests/ file is never executed at all.
    //
    // Vacuity-probed 2026-07-29: adding `impl AtLeast<Continuous> for
    // CellQuantized` turned the line-105 doctest FAILED while the other two
    // stayed green, so the guard discriminates the specific edge it names.
    #[test]
    fn ladder_is_ordered() {
        assert!(MotionClass::Static < MotionClass::Discrete);
        assert!(MotionClass::Discrete < MotionClass::CellQuantized);
        assert!(MotionClass::CellQuantized < MotionClass::Continuous);
    }
}
