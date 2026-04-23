//! Motion tokens — easing curves and durations.
//!
//! Curves are inspired by blackmatter-ghostty's shader motion (sonic-boom,
//! stardust, prompt-saber) so UI animation feels continuous with TUI effects.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Motion {
    pub duration: Durations,
    pub easing: Easings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Durations {
    pub instant_ms: u16,
    pub fast_ms: u16,
    pub base_ms: u16,
    pub slow_ms: u16,
    pub hero_ms: u16,
}

/// CSS cubic-bezier tuples.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Cubic(pub f32, pub f32, pub f32, pub f32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
