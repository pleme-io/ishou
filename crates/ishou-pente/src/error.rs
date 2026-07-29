//! Typed failures of the pente algebra.
//!
//! Tier-honesty note, stated once and relied on throughout: every variant
//! here is a `Result::Err`, which is **mitigation**, not unrepresentability.
//! The properties this crate claims as *unrepresentable* are the ones with
//! no code path at all — a missing `CoreRole` is `E0063` (a compile error),
//! and a face resolving a raw token is an **absent method**. Where the
//! strongest available technique is a rejection, it lives here and is
//! graded `parse-time-rejected`, never rounded up.

use thiserror::Error;

use crate::TokenName;

/// Every way authored pente data can be wrong.
// No `Eq`: `AlphaOutOfRange` carries an `f64`, which is not `Eq`. Keeping
// `PartialEq` is what the tests need; deriving `Eq` would force the error
// payload to lose the offending value, and a diagnostic that cannot say
// WHICH alpha was wrong is not actionable on a 38-token ramp.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum PenteError {
    /// A token referenced by a binding, a blend, an alias or a mix does not
    /// exist in the ramp it was resolved against.
    ///
    /// This is §V.1 gap 2: today a binding names token strings that nothing
    /// validates against the palette, so a typo resolves to a fallback
    /// colour at render time instead of failing at compile time.
    #[error("unknown token `{token}` in ramp `{ramp}`")]
    UnknownToken { ramp: String, token: TokenName },

    /// `Origin` references form a cycle, so `resolve` would not terminate.
    ///
    /// Reported with the full cycle path rather than just the entry point,
    /// because a bare "cycle detected" on a 38-token ramp is not actionable.
    #[error("cyclic token definition: {}", .path.join(" -> "))]
    Cycle { path: Vec<String> },

    /// An alpha / mix parameter was outside `[0.0, 1.0]`.
    ///
    /// This REJECTS rather than clamps, deliberately. `ishou_tokens::Refined`
    /// clamps (`Percent::new(-0.5).get() == 0.0`), which would silently turn
    /// an authored `:alpha 5.0` into `1.0` — a wrong value in the one
    /// authored field where a wrong value is expressible.
    #[error("alpha {value} out of range: must be within [0.0, 1.0]")]
    AlphaOutOfRange { value: f64 },

    /// A ramp declared two `Origin`s for the same token name.
    #[error("duplicate token `{token}` in ramp `{ramp}`")]
    DuplicateToken { ramp: String, token: TokenName },

    /// A face named a `FaceRole` it did not declare in its `consumes` list.
    ///
    /// CI tier, honestly: face-role completeness is a catalog check, not a
    /// compile error, because `FaceRole` is deliberately OPEN (§V.4).
    #[error("face `{face}` binds undeclared face-role `{role}`")]
    UndeclaredFaceRole { face: String, role: String },

    /// A face was asked for a role it does not bind.
    #[error("face `{face}` does not bind `{role}`")]
    UnboundRole { face: String, role: String },
}

/// Result alias for the pente algebra.
pub type Result<T> = core::result::Result<T, PenteError>;
