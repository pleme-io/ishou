//! `Beat` — the atom that makes stray durations homeless.
//!
//! `Beat::Struck` is the ONLY variant carrying a literal duration, exactly as
//! `Origin::Born` is the only hex-bearing arm in pente. A hand-typed
//! `Duration::from_millis(500)` inside a batente-linking crate has no field
//! to live in.

use serde::{Deserialize, Serialize};

use crate::error::{BatenteError, Result};

/// A name in a cadence's motion universe (`cursor-blink`, `mode-flash`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MotionName(String);

impl MotionName {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for MotionName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for MotionName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// A duration in milliseconds, provably non-negative and finite.
///
/// Parse-don't-validate. This does NOT reuse `mado::motion::NonNegSecs`, and
/// the reason is the same one that forced `UnitInterval` in pente: that bound
/// is a `Refined` whose `new` **clamps** rather than rejects. A clamped
/// duration is a plausible wrong animation; a rejected one is a loud author
/// error, and the author is the only one who can fix it.
///
/// TIER: parse-time-rejected. Not unrepresentable — `new` returns a `Result`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Millis(u32);

impl Millis {
    /// Reject a negative or non-finite duration.
    pub fn new(ms: i64) -> Result<Self> {
        if ms < 0 {
            return Err(BatenteError::NegativeDuration { millis: ms });
        }
        u32::try_from(ms)
            .map(Self)
            .map_err(|_| BatenteError::NegativeDuration { millis: ms })
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Seconds, for handing to an evaluator whose contract is `dt` in seconds
    /// (mado's `Advance::advance`).
    #[must_use]
    pub fn as_secs_f32(self) -> f32 {
        self.0 as f32 / 1000.0
    }
}

impl<'de> Deserialize<'de> for Millis {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        let raw = i64::deserialize(d)?;
        Self::new(raw).map_err(serde::de::Error::custom)
    }
}

/// How many times a beat plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Repeat {
    Once,
    Times(u32),
    Forever,
}

/// A named easing curve, resolved against `ishou_tokens::motion::Easings`.
///
/// `ishou_tokens` STAYS the curve source of truth — batente names the curve,
/// it does not own the control points. Same relationship pente has with
/// `irodori::NORD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CurveName {
    Linear,
    Standard,
    Decelerate,
    Accelerate,
    SonicBoom,
    Saber,
}

impl CurveName {
    pub const ALL: [Self; 6] = [
        Self::Linear,
        Self::Standard,
        Self::Decelerate,
        Self::Accelerate,
        Self::SonicBoom,
        Self::Saber,
    ];

    /// The four cubic-bézier control coordinates, sourced from ishou.
    /// `Linear` is the identity and has no bézier.
    #[must_use]
    pub fn cubic(self) -> Option<(f32, f32, f32, f32)> {
        let e = ishou_tokens::motion::Motion::default().easing;
        let c = match self {
            Self::Linear => return None,
            Self::Standard => e.standard,
            Self::Decelerate => e.decelerate,
            Self::Accelerate => e.accelerate,
            Self::SonicBoom => e.sonic_boom,
            Self::Saber => e.saber,
        };
        Some((c.0, c.1, c.2, c.3))
    }

    #[must_use]
    pub const fn as_symbol(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Standard => "standard",
            Self::Decelerate => "decelerate",
            Self::Accelerate => "accelerate",
            Self::SonicBoom => "sonic-boom",
            Self::Saber => "saber",
        }
    }

    #[must_use]
    pub fn from_symbol(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_symbol() == s)
    }
}

/// How a motion gets its shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Beat {
    /// A literal duration + curve. The ONE place a duration may be authored.
    Struck {
        duration: Millis,
        curve: CurveName,
        #[serde(default = "once")]
        repeat: Repeat,
    },
    /// A second name for an existing motion.
    Held { of: MotionName },
    /// An exponential decay wrapper.
    Damped { of: MotionName, half_life: Millis },
    /// Replay an existing motion N times.
    Repeated { of: MotionName, count: Repeat },
}

fn once() -> Repeat {
    Repeat::Once
}

impl Beat {
    /// The motions this beat depends on. Drives cycle detection.
    #[must_use]
    pub fn depends_on(&self) -> Vec<&MotionName> {
        match self {
            Self::Struck { .. } => Vec::new(),
            Self::Held { of } | Self::Damped { of, .. } | Self::Repeated { of, .. } => vec![of],
        }
    }

    /// Reject the combinations that are nonsense on their face.
    ///
    /// A `Forever` repeat over a zero duration is a spin, not an animation —
    /// it would burn a core producing no visible change.
    pub fn validate(&self, name: &MotionName) -> Result<()> {
        match self {
            Self::Struck {
                duration, repeat, ..
            } if duration.is_zero() && matches!(repeat, Repeat::Forever) => {
                Err(BatenteError::ZeroLengthForever {
                    motion: name.clone(),
                })
            }
            Self::Repeated {
                count: Repeat::Forever,
                ..
            } => Ok(()), // the inner duration is checked when it resolves
            _ => Ok(()),
        }
    }

    /// The minimum face capability this beat demands.
    ///
    /// A pure on/off with no curve is `Discrete`; anything eased needs at
    /// least cell-quantised movement. This is what the typestate bound is
    /// derived from at a call site.
    #[must_use]
    pub fn demands(&self) -> crate::capability::MotionClass {
        use crate::capability::MotionClass;
        match self {
            Self::Struck {
                curve: CurveName::Linear,
                ..
            } => MotionClass::Discrete,
            Self::Struck { .. } => MotionClass::CellQuantized,
            Self::Damped { .. } => MotionClass::Continuous,
            Self::Held { .. } | Self::Repeated { .. } => MotionClass::Discrete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millis_rejects_negative_rather_than_clamping() {
        // The whole reason this exists instead of mado's Refined bound.
        assert!(Millis::new(-1).is_err());
        assert!(Millis::new(-500).is_err());
        assert_eq!(Millis::new(0).unwrap().get(), 0);
        assert_eq!(Millis::new(250).unwrap().get(), 250);
    }

    #[test]
    fn negative_duration_names_its_value() {
        assert_eq!(
            Millis::new(-7).unwrap_err(),
            BatenteError::NegativeDuration { millis: -7 }
        );
    }

    #[test]
    fn zero_length_forever_is_rejected() {
        // A spin, not an animation.
        let b = Beat::Struck {
            duration: Millis::new(0).unwrap(),
            curve: CurveName::Linear,
            repeat: Repeat::Forever,
        };
        assert!(b.validate(&"m".into()).is_err());

        // Zero-length ONCE is fine — an instant state change.
        let ok = Beat::Struck {
            duration: Millis::new(0).unwrap(),
            curve: CurveName::Linear,
            repeat: Repeat::Once,
        };
        assert!(ok.validate(&"m".into()).is_ok());
    }

    #[test]
    fn curves_source_their_control_points_from_ishou() {
        // batente NAMES the curve; ishou_tokens owns the values. If this
        // drifts, the fleet has two disagreeing definitions of `standard`.
        let want = ishou_tokens::motion::Motion::default().easing.standard;
        let got = CurveName::Standard.cubic().unwrap();
        assert_eq!(got, (want.0, want.1, want.2, want.3));
        assert_eq!(CurveName::Linear.cubic(), None);
    }

    #[test]
    fn curve_symbols_round_trip() {
        for c in CurveName::ALL {
            assert_eq!(CurveName::from_symbol(c.as_symbol()), Some(c));
        }
        assert_eq!(CurveName::from_symbol("sonic_boom"), None); // underscore is not the symbol
    }

    #[test]
    fn only_struck_carries_a_literal() {
        let s = Beat::Struck {
            duration: Millis::new(500).unwrap(),
            curve: CurveName::Saber,
            repeat: Repeat::Once,
        };
        assert!(s.depends_on().is_empty());
        assert_eq!(Beat::Held { of: "x".into() }.depends_on().len(), 1);
    }
}
