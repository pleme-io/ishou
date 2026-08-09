//! `Cadence` — `(defcadence …)`. The motion ramp.
//!
//! Structurally identical to pente's `Ramp`, one axis over: a named, closed
//! `MotionName -> Beat` map with a topological `resolve()` that is total
//! after validation.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::beat::{Beat, CurveName, Millis, MotionName, Repeat};
use crate::capability::MotionClass;
use crate::error::{BatenteError, Result};

/// A named universe of motions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cadence {
    pub name: String,
    /// BTreeMap so resolution order is deterministic: a cycle diagnostic
    /// names the same path every run, and rendered artifacts stay byte-stable.
    pub beats: BTreeMap<MotionName, Beat>,
}

impl Cadence {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            beats: BTreeMap::new(),
        }
    }

    /// Add a beat. Rejects redefinition rather than last-write-wins — that is
    /// how a vocabulary acquires two disagreeing definitions of one name and
    /// nobody notices.
    pub fn insert(&mut self, name: impl Into<MotionName>, beat: Beat) -> Result<&mut Self> {
        let name = name.into();
        if self.beats.contains_key(&name) {
            return Err(BatenteError::DuplicateMotion {
                cadence: self.name.clone(),
                motion: name,
            });
        }
        beat.validate(&name)?;
        self.beats.insert(name, beat);
        Ok(self)
    }

    #[must_use]
    pub fn contains(&self, m: &MotionName) -> bool {
        self.beats.contains_key(m)
    }

    /// Flatten every beat to a concrete animation.
    pub fn resolve(&self) -> Result<ResolvedCadence> {
        let mut out: HashMap<MotionName, Animation> = HashMap::with_capacity(self.beats.len());
        let mut state: HashMap<&MotionName, Visit> = HashMap::new();
        let mut stack: Vec<&MotionName> = Vec::new();
        for name in self.beats.keys() {
            self.visit(name, &mut state, &mut stack, &mut out)?;
        }
        Ok(ResolvedCadence {
            name: self.name.clone(),
            animations: out,
        })
    }

    fn visit<'a>(
        &'a self,
        name: &'a MotionName,
        state: &mut HashMap<&'a MotionName, Visit>,
        stack: &mut Vec<&'a MotionName>,
        out: &mut HashMap<MotionName, Animation>,
    ) -> Result<Animation> {
        match state.get(name) {
            Some(Visit::Done) => return Ok(out[name].clone()),
            Some(Visit::InProgress) => {
                let start = stack.iter().position(|n| *n == name).unwrap_or(0);
                let mut path: Vec<String> =
                    stack[start..].iter().map(ToString::to_string).collect();
                path.push(name.to_string());
                return Err(BatenteError::Cycle { path });
            }
            None => {}
        }

        let beat = self
            .beats
            .get(name)
            .ok_or_else(|| BatenteError::UnknownMotion {
                cadence: self.name.clone(),
                motion: name.clone(),
            })?;

        state.insert(name, Visit::InProgress);
        stack.push(name);

        let anim = match beat {
            Beat::Struck {
                duration,
                curve,
                repeat,
            } => Animation {
                duration: *duration,
                curve: *curve,
                repeat: *repeat,
                half_life: None,
                demands: beat.demands(),
            },
            Beat::Held { of } => self.visit(of, state, stack, out)?,
            Beat::Damped { of, half_life } => {
                let inner = self.visit(of, state, stack, out)?;
                Animation {
                    half_life: Some(*half_life),
                    demands: MotionClass::Continuous,
                    ..inner
                }
            }
            Beat::Repeated { of, count } => {
                let inner = self.visit(of, state, stack, out)?;
                // Re-check the spin condition AFTER flattening: `Repeated`
                // over a zero-length `Struck` is the same spin as an inline
                // `Forever`, and only becomes visible here.
                if inner.duration.is_zero() && matches!(count, Repeat::Forever) {
                    return Err(BatenteError::ZeroLengthForever {
                        motion: name.clone(),
                    });
                }
                Animation {
                    repeat: *count,
                    ..inner
                }
            }
        };

        stack.pop();
        state.insert(name, Visit::Done);
        out.insert(name.clone(), anim.clone());
        Ok(anim)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visit {
    InProgress,
    Done,
}

/// A flattened motion, ready to hand to an evaluator.
///
/// batente does NOT evaluate. `mado/src/motion/` is the fleet's one animation
/// evaluator (property-tested, WebKit UnitBezier solve) and is single-tenant
/// by explicit decision, gated at its 3rd consumer. batente is #2, so it
/// holds the border and delegates — re-implementing the solve here would
/// create the second evaluator this whole doctrine exists to prevent.
#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub duration: Millis,
    pub curve: CurveName,
    pub repeat: Repeat,
    pub half_life: Option<Millis>,
    /// The minimum face capability required to play this.
    pub demands: MotionClass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCadence {
    pub name: String,
    animations: HashMap<MotionName, Animation>,
}

impl ResolvedCadence {
    #[must_use]
    pub fn get(&self, m: &MotionName) -> Option<&Animation> {
        self.animations.get(m)
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.animations.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn struck(ms: i64, c: CurveName) -> Beat {
        Beat::Struck {
            duration: Millis::new(ms).unwrap(),
            curve: c,
            repeat: Repeat::Once,
        }
    }

    #[test]
    fn resolves_struck_held_and_damped() {
        let mut c = Cadence::new("t");
        c.insert("base", struck(250, CurveName::Standard)).unwrap();
        c.insert("alias", Beat::Held { of: "base".into() }).unwrap();
        c.insert(
            "fade",
            Beat::Damped {
                of: "base".into(),
                half_life: Millis::new(100).unwrap(),
            },
        )
        .unwrap();

        let r = c.resolve().unwrap();
        assert_eq!(r.get(&"alias".into()).unwrap().duration.get(), 250);
        assert_eq!(r.get(&"fade".into()).unwrap().half_life.unwrap().get(), 100);
        // Damping forces the strongest capability demand.
        assert_eq!(
            r.get(&"fade".into()).unwrap().demands,
            MotionClass::Continuous
        );
    }

    #[test]
    fn unknown_motion_is_caught_at_resolve() {
        let mut c = Cadence::new("t");
        c.insert("a", Beat::Held { of: "nope".into() }).unwrap();
        assert!(matches!(
            c.resolve().unwrap_err(),
            BatenteError::UnknownMotion { .. }
        ));
    }

    #[test]
    fn cycle_reports_the_path() {
        let mut c = Cadence::new("t");
        c.insert("a", Beat::Held { of: "b".into() }).unwrap();
        c.insert("b", Beat::Held { of: "a".into() }).unwrap();
        match c.resolve().unwrap_err() {
            BatenteError::Cycle { path } => assert_eq!(path.first(), path.last()),
            o => panic!("expected Cycle, got {o:?}"),
        }
    }

    #[test]
    fn a_diamond_is_not_a_cycle() {
        let mut c = Cadence::new("t");
        c.insert("base", struck(10, CurveName::Linear)).unwrap();
        c.insert("l", Beat::Held { of: "base".into() }).unwrap();
        c.insert("r", Beat::Held { of: "base".into() }).unwrap();
        assert!(c.resolve().is_ok());
    }

    #[test]
    fn repeated_forever_over_zero_length_is_caught_after_flattening() {
        // The spin condition is invisible at insert time here — `Repeated`
        // does not carry the duration, the thing it points at does.
        let mut c = Cadence::new("t");
        c.insert("zero", struck(0, CurveName::Linear)).unwrap();
        c.insert(
            "spin",
            Beat::Repeated {
                of: "zero".into(),
                count: Repeat::Forever,
            },
        )
        .unwrap();
        assert!(matches!(
            c.resolve().unwrap_err(),
            BatenteError::ZeroLengthForever { .. }
        ));
    }

    #[test]
    fn duplicate_motion_is_rejected() {
        let mut c = Cadence::new("t");
        c.insert("a", struck(1, CurveName::Linear)).unwrap();
        assert!(c.insert("a", struck(2, CurveName::Linear)).is_err());
    }
}
