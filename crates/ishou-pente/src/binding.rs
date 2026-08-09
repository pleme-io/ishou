//! `Binding` — `(defbinding …)`. **A binding IS a theme.**
//!
//! This is literally today's `SemanticRoles { background: "polar_night_0", … }`
//! promoted from a `const fn` to authored data, with the token<->ramp edge
//! validated. Nord-vs-Vellum becomes one `:binding` line rather than a
//! fleet-wide edit.
//!
//! TIER-HONESTY. `Binding` is a plain product struct, so constructing one
//! without every `CoreRole` is `E0063` (missing field in struct literal) — a
//! genuine compile error. But say plainly that this is an **INHERITED**
//! property, not a pente invention: `ishou_tokens::SemanticRoles` is already
//! exactly this shape (26 non-`Option` fields) and already yields `E0063`
//! today. Pente's two actual contributions are:
//!   (a) the values become `TokenName`s VALIDATED against a named ramp, and
//!   (b) they become authored data rather than a `const fn`.
//!
//! An earlier draft proposed `EnumMap<CoreRole, TokenName>` and graded it
//! `E0063`-unrepresentable. Both halves were wrong: `enum-map` is a brand-new
//! external dependency (zero occurrences fleet-wide), and `EnumMap` is not a
//! struct literal — it implements `Default` and `from_fn`, so an all-defaults
//! map is freely constructible and `E0063` does not apply.

use serde::{Deserialize, Serialize};

use crate::error::{PenteError, Result};
use crate::origin::TokenName;
use crate::ramp::Ramp;
use crate::role::CoreRole;

/// A complete theme: every `CoreRole` bound to a token in some ramp.
///
/// One field per `CoreRole`. Adding a variant to `CoreRole` breaks every
/// binding at compile time — which is the point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    /// Which binding this is (`nord`, `vellum-night`).
    pub name: String,
    /// Which ramp its tokens are drawn from.
    pub ramp: String,

    pub background: TokenName,
    pub surface: TokenName,
    pub surface_elevated: TokenName,
    pub text: TokenName,
    pub text_muted: TokenName,
    pub text_dim: TokenName,
    pub primary: TokenName,
    pub accent: TokenName,
    pub border: TokenName,
    pub error: TokenName,
    pub warning: TokenName,
    pub success: TokenName,
    pub info: TokenName,
    pub selection: TokenName,
    pub cursor: TokenName,
    pub agent: TokenName,
}

impl Binding {
    /// The token bound to a role. Total — no `Option`, no fallback.
    ///
    /// Exhaustive match with no `_` arm: a new `CoreRole` fails to compile
    /// here rather than silently resolving to `text`.
    #[must_use]
    pub fn token(&self, role: CoreRole) -> &TokenName {
        match role {
            CoreRole::Background => &self.background,
            CoreRole::Surface => &self.surface,
            CoreRole::SurfaceElevated => &self.surface_elevated,
            CoreRole::Text => &self.text,
            CoreRole::TextMuted => &self.text_muted,
            CoreRole::TextDim => &self.text_dim,
            CoreRole::Primary => &self.primary,
            CoreRole::Accent => &self.accent,
            CoreRole::Border => &self.border,
            CoreRole::Error => &self.error,
            CoreRole::Warning => &self.warning,
            CoreRole::Success => &self.success,
            CoreRole::Info => &self.info,
            CoreRole::Selection => &self.selection,
            CoreRole::Cursor => &self.cursor,
            CoreRole::Agent => &self.agent,
        }
    }

    /// Every (role, token) pair, in `CoreRole::ALL` order.
    #[must_use]
    pub fn pairs(&self) -> Vec<(CoreRole, &TokenName)> {
        CoreRole::ALL
            .into_iter()
            .map(|r| (r, self.token(r)))
            .collect()
    }

    /// Validate every bound token against the ramp it claims to draw from.
    ///
    /// THIS IS THE EDGE THAT DID NOT EXIST BEFORE. Today a binding names
    /// token strings that nothing checks, so a typo resolves to a fallback
    /// colour at render time. Here it is a typed error before anything paints.
    pub fn validate(&self, ramp: &Ramp) -> Result<()> {
        for (_role, token) in self.pairs() {
            if !ramp.contains(token) {
                return Err(PenteError::UnknownToken {
                    ramp: ramp.name.clone(),
                    token: token.clone(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::origin::Origin;
    use ishou_tokens::Srgb;

    fn binding_all(token: &str) -> Binding {
        let t = || TokenName::new(token);
        Binding {
            name: "t".into(),
            ramp: "r".into(),
            background: t(),
            surface: t(),
            surface_elevated: t(),
            text: t(),
            text_muted: t(),
            text_dim: t(),
            primary: t(),
            accent: t(),
            border: t(),
            error: t(),
            warning: t(),
            success: t(),
            info: t(),
            selection: t(),
            cursor: t(),
            agent: t(),
        }
    }

    #[test]
    fn token_is_total_over_every_core_role() {
        let b = binding_all("x");
        // Every role resolves; none panics, none falls back.
        assert_eq!(b.pairs().len(), CoreRole::ALL.len());
        for role in CoreRole::ALL {
            assert_eq!(b.token(role).as_str(), "x");
        }
    }

    #[test]
    fn validate_rejects_a_token_absent_from_the_ramp() {
        // The gap this crate exists to close: a typo'd token used to reach
        // render as a fallback colour.
        let mut ramp = Ramp::new("r");
        ramp.insert(
            "real",
            Origin::Born {
                srgb: Srgb::new(1, 2, 3),
            },
        )
        .unwrap();

        let good = binding_all("real");
        assert!(good.validate(&ramp).is_ok());

        let typo = binding_all("raal");
        assert_eq!(
            typo.validate(&ramp).unwrap_err(),
            PenteError::UnknownToken {
                ramp: "r".into(),
                token: "raal".into()
            }
        );
    }
}
