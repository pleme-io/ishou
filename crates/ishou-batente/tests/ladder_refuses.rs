//! The load-bearing NEGATIVE proof for the capability ladder.
//!
//! `an_incapable_face_is_refused_not_silently_degraded` (in lib.rs) covers
//! the DYNAMIC path, where a face's class is data and an over-demand is a
//! typed `Result::Err` — mitigation.
//!
//! The STATIC path's negative proof lives in `capability.rs` as
//! `compile_fail` DOCTESTS on the `AtLeast` trait, not here. Doctests run
//! only on the LIB target -- a `compile_fail` block in a tests/ file is
//! never executed, and this file originally carried them: `cargo test --doc`
//! reported `0 tests` while the header claimed the property was proven.
//! That is the vacuous guard the header itself warns about, so it is
//! recorded rather than quietly fixed.
//!
//! What remains here is the POSITIVE half: every upward edge of the ladder
//! is reachable, so a rename breaks this loudly instead of silently turning
//! the doctests into tautologies.
//!
//! VACUITY MATTERS HERE. A `compile_fail` test passes if the snippet fails
//! for ANY reason — a typo, a missing import, a renamed type — so a stale
//! one silently stops testing what it names. Each negative below is
//! therefore paired with a positive that must compile, so a rename breaks
//! the pair rather than quietly turning the negative into a tautology.

use ishou_batente::capability::{
    AtLeast, CellQuantized, Continuous, Discrete, MotionClass, Rung, Static,
};

fn class_of<R: Rung, W: AtLeast<R>>(_face: W) -> MotionClass {
    W::CLASS
}

/// The POSITIVE half, as a real test rather than only a doctest: every
/// upward edge of the ladder is reachable. If a rename breaks these, the
/// `compile_fail` blocks above stop proving anything and this goes red
/// first — which is the point of pairing them.
#[test]
fn every_upward_edge_is_satisfiable() {
    assert_eq!(class_of::<Static, _>(Static), MotionClass::Static);
    assert_eq!(class_of::<Static, _>(Discrete), MotionClass::Discrete);
    assert_eq!(
        class_of::<Static, _>(CellQuantized),
        MotionClass::CellQuantized
    );
    assert_eq!(class_of::<Static, _>(Continuous), MotionClass::Continuous);

    assert_eq!(class_of::<Discrete, _>(Discrete), MotionClass::Discrete);
    assert_eq!(
        class_of::<Discrete, _>(CellQuantized),
        MotionClass::CellQuantized
    );
    assert_eq!(class_of::<Discrete, _>(Continuous), MotionClass::Continuous);

    assert_eq!(
        class_of::<CellQuantized, _>(CellQuantized),
        MotionClass::CellQuantized
    );
    assert_eq!(
        class_of::<CellQuantized, _>(Continuous),
        MotionClass::Continuous
    );

    assert_eq!(
        class_of::<Continuous, _>(Continuous),
        MotionClass::Continuous
    );
}
