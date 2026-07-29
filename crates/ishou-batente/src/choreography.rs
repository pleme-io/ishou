//! `Choreography` — `(defchoreography …)`. **A choreography IS a motion theme.**
//!
//! A plain product struct, one `MotionName` field per `CoreMotion`, so a
//! choreography missing a motion is `E0063` (missing field in struct
//! literal) — a genuine compile error.
//!
//! The shape also makes "two animations fighting over one property"
//! **unrepresentable** rather than resolved by a priority field: there is
//! exactly one slot per motion, so a second claimant has nowhere to go. That
//! is the one property here that pente has no analogue for — two colours
//! cannot fight over a role because resolution is a pure lookup, but two
//! animations racing the same property is the classic UI-motion bug.

use serde::{Deserialize, Serialize};

use crate::beat::MotionName;
use crate::cadence::Cadence;
use crate::error::{BatenteError, Result};
use crate::motion::CoreMotion;

/// A complete motion theme: every `CoreMotion` bound to a motion in a cadence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choreography {
    /// Which choreography this is (`calm`, `snappy`, `reduced`).
    pub name: String,
    /// Which cadence its motions are drawn from.
    pub cadence: String,

    pub cursor_blink: MotionName,
    pub cursor_move: MotionName,
    pub mode_change: MotionName,
    pub scroll: MotionName,
    pub search_pulse: MotionName,
    pub diagnostic_fade: MotionName,
    pub selection_fade: MotionName,
    pub bell: MotionName,
}

impl Choreography {
    /// Every core motion bound to the same name. Test scaffolding, and the
    /// honest shape of a `reduced`/`none` choreography where every motion
    /// collapses to one instant beat.
    #[must_use]
    pub fn uniform(
        name: impl Into<String>,
        cadence: impl Into<String>,
        motion: MotionName,
    ) -> Self {
        let m = || motion.clone();
        Self {
            name: name.into(),
            cadence: cadence.into(),
            cursor_blink: m(),
            cursor_move: m(),
            mode_change: m(),
            scroll: m(),
            search_pulse: m(),
            diagnostic_fade: m(),
            selection_fade: m(),
            bell: m(),
        }
    }

    /// The motion bound to a core slot. Total — no `Option`, no fallback.
    ///
    /// Exhaustive with no `_` arm: a new `CoreMotion` fails to compile here
    /// rather than silently resolving to `cursor_blink`.
    #[must_use]
    pub fn motion(&self, m: CoreMotion) -> &MotionName {
        match m {
            CoreMotion::CursorBlink => &self.cursor_blink,
            CoreMotion::CursorMove => &self.cursor_move,
            CoreMotion::ModeChange => &self.mode_change,
            CoreMotion::Scroll => &self.scroll,
            CoreMotion::SearchPulse => &self.search_pulse,
            CoreMotion::DiagnosticFade => &self.diagnostic_fade,
            CoreMotion::SelectionFade => &self.selection_fade,
            CoreMotion::Bell => &self.bell,
        }
    }

    /// Every (motion, name) pair, in `CoreMotion::ALL` order.
    #[must_use]
    pub fn pairs(&self) -> Vec<(CoreMotion, &MotionName)> {
        CoreMotion::ALL
            .into_iter()
            .map(|m| (m, self.motion(m)))
            .collect()
    }

    /// Validate every bound motion against the cadence it claims to draw
    /// from. The edge that did not exist before: today a motion name is a
    /// bare string nothing checks.
    pub fn validate(&self, cadence: &Cadence) -> Result<()> {
        for (_m, name) in self.pairs() {
            if !cadence.contains(name) {
                return Err(BatenteError::UnknownMotion {
                    cadence: cadence.name.clone(),
                    motion: name.clone(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beat::{Beat, CurveName, Millis, Repeat};

    #[test]
    fn motion_is_total_over_every_core_motion() {
        let c = Choreography::uniform("ch", "cd", MotionName::new("x"));
        assert_eq!(c.pairs().len(), CoreMotion::ALL.len());
        for m in CoreMotion::ALL {
            assert_eq!(c.motion(m).as_str(), "x");
        }
    }

    #[test]
    fn validate_rejects_a_motion_absent_from_the_cadence() {
        let mut cad = Cadence::new("cd");
        cad.insert(
            "real",
            Beat::Struck {
                duration: Millis::new(100).unwrap(),
                curve: CurveName::Linear,
                repeat: Repeat::Once,
            },
        )
        .unwrap();

        assert!(
            Choreography::uniform("ch", "cd", MotionName::new("real"))
                .validate(&cad)
                .is_ok()
        );
        assert!(matches!(
            Choreography::uniform("ch", "cd", MotionName::new("raal"))
                .validate(&cad)
                .unwrap_err(),
            BatenteError::UnknownMotion { .. }
        ));
    }

    #[test]
    fn one_slot_per_motion_makes_a_second_claimant_homeless() {
        // The structural property: there is exactly one field per CoreMotion,
        // so "two animations fighting over one property" has no way to be
        // expressed. Asserted by construction — assigning twice is just a
        // reassignment, never two live claimants.
        let mut c = Choreography::uniform("ch", "cd", MotionName::new("a"));
        c.scroll = MotionName::new("b");
        assert_eq!(c.motion(CoreMotion::Scroll).as_str(), "b");
        assert_eq!(c.pairs().len(), 8);
    }
}
