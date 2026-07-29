//! Motion tokens — easing curves and durations.
//!
//! Curves are inspired by blackmatter-ghostty's shader motion (sonic-boom,
//! stardust, prompt-saber) so UI animation feels continuous with TUI effects.
//!
//! `Deserialize` is derived alongside `Serialize`, and that pairing is
//! load-bearing rather than incidental. This module was `Serialize`-only
//! until 2026-07-29, which made the motion vocabulary **structurally
//! un-authorable**: tokens could be rendered OUT to CSS/Tailwind but no
//! authored `(defcadence …)` form, config file or catalog could ever be read
//! back IN. A one-way token layer cannot be the source of truth for a
//! spine — `ishou-batente` names these curves, so the round trip has to
//! close for the authoring border to exist at all.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Motion {
    pub duration: Durations,
    pub easing: Easings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Durations {
    pub instant_ms: u16,
    pub fast_ms: u16,
    pub base_ms: u16,
    pub slow_ms: u16,
    pub hero_ms: u16,
}

/// CSS cubic-bezier tuples.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cubic(pub f32, pub f32, pub f32, pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Easings {
    pub standard: Cubic,
    pub decelerate: Cubic,
    pub accelerate: Cubic,
    /// "Sonic boom" — quick attack, long settle. Matches the shader.
    pub sonic_boom: Cubic,
    /// "Saber swoop" — curved in/out, steady middle. prompt-saber shader.
    pub saber: Cubic,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            duration: Durations {
                instant_ms: 80,
                fast_ms: 150,
                base_ms: 250,
                slow_ms: 450,
                hero_ms: 800,
            },
            easing: Easings {
                standard: Cubic(0.4, 0.0, 0.2, 1.0),
                decelerate: Cubic(0.0, 0.0, 0.2, 1.0),
                accelerate: Cubic(0.4, 0.0, 1.0, 1.0),
                sonic_boom: Cubic(0.12, 0.8, 0.3, 1.0),
                saber: Cubic(0.65, 0.0, 0.35, 1.0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The motion vocabulary must ROUND TRIP, not merely serialize.
    ///
    /// This module was `Serialize`-only until 2026-07-29, which made it
    /// structurally un-authorable: tokens rendered OUT to CSS/Tailwind but no
    /// authored form could ever be read back IN. `ishou-batente` names these
    /// curves, so a one-way vocabulary cannot be its source of truth.
    ///
    /// Asserting the round trip rather than just `Deserialize`-compiles is
    /// the point: a derive that exists but loses a field would satisfy the
    /// compiler and still break authoring.
    #[test]
    fn motion_tokens_round_trip() {
        let src = Motion::default();
        let json = serde_json::to_string(&src).expect("serialize");
        let back: Motion = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(back.duration.instant_ms, src.duration.instant_ms);
        assert_eq!(back.duration.hero_ms, src.duration.hero_ms);

        // Every named curve survives, control point for control point — a
        // bezier that loses one coordinate is a different animation.
        for (a, b) in [
            (back.easing.standard, src.easing.standard),
            (back.easing.decelerate, src.easing.decelerate),
            (back.easing.accelerate, src.easing.accelerate),
            (back.easing.sonic_boom, src.easing.sonic_boom),
            (back.easing.saber, src.easing.saber),
        ] {
            assert_eq!((a.0, a.1, a.2, a.3), (b.0, b.1, b.2, b.3));
        }
    }
}
