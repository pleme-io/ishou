//! `Origin` — the atom that makes stray hexes homeless.
//!
//! `Origin::Born` is the ONLY variant in the entire algebra that carries a
//! colour literal. Every downstream type — `Binding`, `FaceSpec`, `Pente`,
//! every emitter — is expressed in `TokenName`s and `Role`s. A hand-typed
//! hex outside a ramp therefore has **no field to live in**.
//!
//! That is a structural property for crates that link this one. For the
//! ishou-free faces (egaku, egaku-term, garasu, engawa) it can only be a CI
//! forcing-function; state the boundary, never the bare property.

use ishou_tokens::Srgb;
use serde::{Deserialize, Serialize};

use crate::error::{PenteError, Result};

/// A name in a ramp's token universe (`polar_night_0`, `frost_2`, …).
///
/// Deliberately a newtype over `String` rather than a closed enum: ramps are
/// authored data and a brand ramp may introduce token names this crate has
/// never seen. Validity is not a property of the *name* but of the
/// name-against-a-ramp, and that is checked in [`crate::Ramp::resolve`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TokenName(String);

impl TokenName {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for TokenName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for TokenName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// A number provably within `[0.0, 1.0]`.
///
/// Parse-don't-validate: the invariant is established once at construction
/// and every consumer downstream reads `.get()` without re-checking.
///
/// This exists rather than reusing `ishou_tokens::Refined<f64, _>` for one
/// decisive reason: `Refined::new` **clamps** out-of-range input
/// (`Percent::new(-0.5).get() == 0.0`) and the crate ships no non-test
/// `Bounds<f64>` impl to extend. Clamping is the wrong semantics here — an
/// authored `:alpha 5.0` is an author error, and silently painting it as
/// `1.0` produces a plausible wrong colour instead of a loud failure.
///
/// TIER: parse-time-rejected. Not unrepresentable — `UnitInterval::new`
/// returns a `Result`, and a caller may still ignore it.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct UnitInterval(f64);

impl UnitInterval {
    /// Reject anything outside `[0.0, 1.0]`, including NaN.
    ///
    /// NaN is rejected explicitly: `!(0.0..=1.0).contains(&nan)` is true, so
    /// a naive range check already excludes it, but the intent is worth
    /// pinning in a test rather than resting on float comparison folklore.
    pub fn new(value: f64) -> Result<Self> {
        if value.is_nan() || !(0.0..=1.0).contains(&value) {
            return Err(PenteError::AlphaOutOfRange { value });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for UnitInterval {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        let raw = f64::deserialize(d)?;
        Self::new(raw).map_err(serde::de::Error::custom)
    }
}

/// How a token gets its colour.
///
/// `Born` is the only hex-bearing constructor in the algebra.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Origin {
    /// A literal colour. The ONE place a hex may be authored.
    Born { srgb: Srgb },
    /// `with` composited over `over` at `alpha`, in LINEAR space.
    ///
    /// Reuses `ishou_tokens::blend_linear` verbatim — the byte-exact recipe
    /// `ishou/tests/vellum_matrix.rs` already pins at zero tolerance. This
    /// is what stops `selection`, `search-others` and the glass tokens from
    /// being simultaneously authored AND blended in three places.
    Blend {
        over: TokenName,
        with: TokenName,
        alpha: UnitInterval,
    },
    /// A second name for an existing token.
    Alias { of: TokenName },
    /// A linear interpolation between two tokens.
    Mix {
        a: TokenName,
        b: TokenName,
        t: UnitInterval,
    },
}

impl Origin {
    /// The tokens this origin depends on. Drives cycle detection and the
    /// topological order in [`crate::Ramp::resolve`].
    #[must_use]
    pub fn depends_on(&self) -> Vec<&TokenName> {
        match self {
            Self::Born { .. } => Vec::new(),
            Self::Blend { over, with, .. } => vec![over, with],
            Self::Alias { of } => vec![of],
            Self::Mix { a, b, .. } => vec![a, b],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_interval_rejects_rather_than_clamps() {
        // The whole reason this type exists instead of ishou_tokens::Refined.
        assert!(UnitInterval::new(-0.5).is_err());
        assert!(UnitInterval::new(5.0).is_err());
        assert!(UnitInterval::new(f64::NAN).is_err());
        assert!(UnitInterval::new(f64::INFINITY).is_err());
        // Boundaries are legal.
        assert_eq!(UnitInterval::new(0.0).unwrap().get(), 0.0);
        assert_eq!(UnitInterval::new(1.0).unwrap().get(), 1.0);
    }

    #[test]
    fn out_of_range_alpha_names_its_value() {
        // A diagnostic that does not say WHICH value was wrong is not
        // actionable on a 38-token ramp.
        let e = UnitInterval::new(5.0).unwrap_err();
        assert_eq!(e, PenteError::AlphaOutOfRange { value: 5.0 });
    }

    #[test]
    fn only_born_carries_a_literal() {
        // The structural claim, asserted rather than asserted-in-prose:
        // every non-Born origin is expressed purely in TokenNames.
        let born = Origin::Born {
            srgb: Srgb::new(0x2E, 0x34, 0x40),
        };
        assert!(born.depends_on().is_empty());

        let blend = Origin::Blend {
            over: "bg".into(),
            with: "accent".into(),
            alpha: UnitInterval::new(0.5).unwrap(),
        };
        assert_eq!(blend.depends_on().len(), 2);
    }
}
