//! # ishou-batente
//!
//! *Batente* (Brazilian-Portuguese): the loom's **beater** — the swinging
//! frame that carries the reed and drives each weft thread home. The reed
//! (**pente**) decides *where* every thread sits; the beater decides *when*
//! every thread lands.
//!
//! **Pente governs uniformity in space. Batente governs uniformity in time.**
//! Canonical spec: `pleme-io/theory/BATENTE.md`. Read `PENTE.md` first — every
//! structural argument here is that argument one axis over.
//!
//! ## The thesis
//!
//! The fleet does not lack a motion source: `mado/src/motion/` is a real,
//! property-tested algebra with a WebKit `UnitBezier` solve, genuinely wired
//! to bell-flash, cursor blink and decay. It does not lack a motion
//! vocabulary either: `ishou_tokens::motion` ships 5 durations and 5 béziers,
//! already emitted to CSS/Tailwind.
//!
//! What it lacks is an **authoring border**, a **closed vocabulary**, and a
//! **forced consumption path** — the identical three things PENTE names for
//! colour. Measured: mado is the ONLY consumer of `ishou_tokens::motion`
//! fleet-wide; escriba has zero animation and never reads the `elapsed`/`dt`
//! its own `RenderContext` hands it.
//!
//! ## The axis pente does not have
//!
//! Every face can paint a colour, so pente needs no capability notion. Faces
//! genuinely differ in what motion they can express — a terminal cell-grid
//! cannot move by a third of a cell. An algebra ignoring that yields the
//! worst failure available: an authored smooth-scroll that *silently
//! degrades*, so the author believes the fleet is uniform when it is not.
//!
//! So [`capability`] is a phantom typestate ladder
//! (`Static < Discrete < CellQuantized < Continuous`) and an over-demanding
//! animation has **no apply path** — `E0277`, not a runtime downgrade. Same
//! construction as `ayatsuri/src/kabe`'s privilege ladder.
//!
//! ## Invariants at their HONEST tiers
//!
//! | Invariant | Technique | Tier |
//! |---|---|---|
//! | Choreography missing a motion | product struct | **truly-unrepresentable** (`E0063`) |
//! | Animation exceeding face capability (static) | `AtLeast<R>` | **truly-unrepresentable** (`E0277`) |
//! | Face resolving a raw motion name | absent method | **truly-unrepresentable** |
//! | New `CoreMotion` silently ignored | exhaustive match | **truly-unrepresentable** |
//! | Duration literal outside a cadence | only `Beat::Struck` has the field | **truly-unrepresentable** *in batente-linking crates* |
//! | Two animations owning one property | one field per motion | **truly-unrepresentable** |
//! | Negative duration | `Millis::new` rejects (mado's bound CLAMPS) | parse-time-rejected |
//! | `Forever` over zero duration | rejected at insert AND after flattening | parse-time-rejected |
//! | Cyclic beat | tri-state DFS | parse-time-rejected |
//! | Animation exceeding capability (dynamic) | `BatenteError::InsufficientCapability` | **only-mitigated** |
//! | reduce-motion / power budget | `garasu::RuntimeBudget` | **runtime-gated** — honestly the weakest row |
//!
//! ## Tier-honest scope of THIS milestone (M0)
//!
//! - **batente does NOT evaluate.** mado owns the one evaluator and is
//!   single-tenant by explicit decision, gated at its 3rd consumer. batente
//!   is #2, so it holds the border and delegates. Re-implementing the bézier
//!   solve here would create the second evaluator this doctrine exists to
//!   prevent.
//! - **No `#[derive(DeriveTataraDomain)]`, no loader.** Same flag-day as
//!   pente: ishou pins no tatara-lisp and the two lineages are incompatible.
//!   `specs/fleet.batente.tlisp` documents the destination form.
//! - **escriba still has zero animation.** Nothing here changes that. M0
//!   ships scaffolding and must be called that; M1's exit criterion is
//!   escriba's GPU face reading `RenderContext.dt` for the first time.

pub mod beat;
pub mod cadence;
pub mod capability;
pub mod choreography;
pub mod error;
pub mod motion;

pub use beat::{Beat, CurveName, Millis, MotionName, Repeat};
pub use cadence::{Animation, Cadence, ResolvedCadence};
pub use capability::{AtLeast, CellQuantized, Continuous, Discrete, MotionClass, Rung, Static};
pub use choreography::Choreography;
pub use error::{BatenteError, Result};
pub use motion::{CoreMotion, FaceMotion, Motion};

use std::collections::BTreeMap;

/// A face's motion declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FaceSpec {
    pub name: String,
    /// What this surface can express. The dynamic reflection of the
    /// typestate rung.
    pub class: MotionClass,
    pub consumes: Vec<FaceMotion>,
    pub face_beats: BTreeMap<FaceMotion, MotionName>,
}

/// The assembled motion spine: one cadence, one choreography, N faces.
#[derive(Debug, Clone)]
pub struct Batente {
    cadence: ResolvedCadence,
    choreography: Choreography,
    faces: BTreeMap<String, FaceSpec>,
}

impl Batente {
    pub fn compile(
        cadence: &Cadence,
        choreography: Choreography,
        faces: Vec<FaceSpec>,
    ) -> Result<Self> {
        let resolved = cadence.resolve()?;
        choreography.validate(cadence)?;
        let mut map = BTreeMap::new();
        for face in faces {
            for (fm, name) in &face.face_beats {
                if !face.consumes.contains(fm) {
                    return Err(BatenteError::UndeclaredFaceMotion {
                        face: face.name.clone(),
                        motion: fm.to_string(),
                    });
                }
                if !cadence.contains(name) {
                    return Err(BatenteError::UnknownMotion {
                        cadence: cadence.name.clone(),
                        motion: name.clone(),
                    });
                }
            }
            map.insert(face.name.clone(), face);
        }
        Ok(Self {
            cadence: resolved,
            choreography,
            faces: map,
        })
    }

    /// Resolve a motion for a face.
    ///
    /// The DYNAMIC path: the face's class is data, so an over-demanding
    /// animation is a typed error. At a static call site the same property is
    /// `E0277` via [`capability::AtLeast`] and this branch is unreachable —
    /// both paths are real, only the static one is unrepresentable.
    pub fn resolve(&self, face: &str, motion: &Motion) -> Result<&Animation> {
        let spec = self.faces.get(face).ok_or_else(|| BatenteError::UnboundMotion {
            face: face.to_string(),
            motion: motion.to_string(),
        })?;

        let name = match motion {
            Motion::Core(c) => self.choreography.motion(*c).clone(),
            Motion::Face(f) => spec
                .face_beats
                .get(f)
                .cloned()
                .ok_or_else(|| BatenteError::UnboundMotion {
                    face: face.to_string(),
                    motion: f.to_string(),
                })?,
        };

        let anim = self
            .cadence
            .get(&name)
            .ok_or_else(|| BatenteError::UnknownMotion {
                cadence: self.cadence.name.clone(),
                motion: name.clone(),
            })?;

        if anim.demands > spec.class {
            return Err(BatenteError::InsufficientCapability {
                face: face.to_string(),
                motion: motion.to_string(),
                actual: spec.class,
                required: anim.demands,
            });
        }
        Ok(anim)
    }

    #[must_use]
    pub fn choreography_name(&self) -> &str {
        &self.choreography.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cadence_with(ms: i64, curve: CurveName) -> Cadence {
        let mut c = Cadence::new("c");
        c.insert(
            "base",
            Beat::Struck {
                duration: Millis::new(ms).unwrap(),
                curve,
                repeat: Repeat::Once,
            },
        )
        .unwrap();
        c
    }

    fn choreo_on(name: &str) -> Choreography {
        Choreography::uniform("ch", "c", MotionName::new(name))
    }

    fn face(name: &str, class: MotionClass) -> FaceSpec {
        FaceSpec {
            name: name.into(),
            class,
            consumes: vec![],
            face_beats: BTreeMap::new(),
        }
    }

    #[test]
    fn a_capable_face_resolves() {
        let c = cadence_with(250, CurveName::Standard); // demands CellQuantized
        let b = Batente::compile(
            &c,
            choreo_on("base"),
            vec![face("gpu", MotionClass::Continuous)],
        )
        .unwrap();
        assert!(b.resolve("gpu", &Motion::Core(CoreMotion::CursorBlink)).is_ok());
    }

    #[test]
    fn an_incapable_face_is_refused_not_silently_degraded() {
        // THE headline property. A Damped beat demands Continuous; a
        // Discrete face must be told no, never quietly given a jump.
        let mut c = Cadence::new("c");
        c.insert(
            "base",
            Beat::Struck {
                duration: Millis::new(250).unwrap(),
                curve: CurveName::Linear,
                repeat: Repeat::Once,
            },
        )
        .unwrap();
        c.insert(
            "fade",
            Beat::Damped {
                of: "base".into(),
                half_life: Millis::new(80).unwrap(),
            },
        )
        .unwrap();

        let b = Batente::compile(
            &c,
            choreo_on("fade"),
            vec![face("nixfile", MotionClass::Discrete)],
        )
        .unwrap();

        match b.resolve("nixfile", &Motion::Core(CoreMotion::Scroll)) {
            Err(BatenteError::InsufficientCapability {
                actual, required, ..
            }) => {
                assert_eq!(actual, MotionClass::Discrete);
                assert_eq!(required, MotionClass::Continuous);
            }
            other => panic!("expected refusal, got {other:?}"),
        }
    }

    #[test]
    fn core_motions_resolve_identically_across_capable_faces() {
        // The beater property: every thread lands in the same time.
        let c = cadence_with(250, CurveName::Standard);
        let b = Batente::compile(
            &c,
            choreo_on("base"),
            vec![
                face("gpu", MotionClass::Continuous),
                face("tui", MotionClass::CellQuantized),
            ],
        )
        .unwrap();
        let m = Motion::Core(CoreMotion::ModeChange);
        assert_eq!(b.resolve("gpu", &m).unwrap(), b.resolve("tui", &m).unwrap());
    }

    #[test]
    fn compile_rejects_a_choreography_naming_an_absent_motion() {
        let c = cadence_with(10, CurveName::Linear);
        assert!(matches!(
            Batente::compile(&c, choreo_on("missing"), vec![]),
            Err(BatenteError::UnknownMotion { .. })
        ));
    }
}
