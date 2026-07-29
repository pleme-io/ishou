//! `Ramp` — `(deframp …)`. A named, closed `TokenName -> Origin` map with a
//! topological `resolve()` that is total after validation.
//!
//! Extends `VellumPalette::get(name) -> Option<Rgb>` + `entries()`, which is
//! already a name-keyed universe. What it adds is the validation edge: every
//! token a `Binding` names is checked against the ramp at compile time, so
//! §V.1's gap 2 (a typo'd token silently resolving to a fallback at render
//! time) stops being expressible.

use std::collections::{BTreeMap, HashMap};

use ishou_tokens::{Rgb, Srgb, blend_linear};
use serde::{Deserialize, Serialize};

use crate::error::{PenteError, Result};
use crate::origin::{Origin, TokenName};

/// A named universe of tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ramp {
    /// `nord`, `vellum`, or a brand ramp. Appears in every diagnostic.
    pub name: String,
    /// BTreeMap, not HashMap: resolution order must be deterministic so a
    /// cycle diagnostic names the same path on every run and any rendered
    /// artifact is byte-stable.
    pub tokens: BTreeMap<TokenName, Origin>,
}

impl Ramp {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tokens: BTreeMap::new(),
        }
    }

    /// Add a token. Rejects a redefinition rather than last-write-wins.
    ///
    /// Last-write-wins is how a palette acquires two disagreeing definitions
    /// of the same name and nobody notices; the fleet's domain registry has
    /// exactly that defect (`register` is `insert`).
    pub fn insert(&mut self, name: impl Into<TokenName>, origin: Origin) -> Result<&mut Self> {
        let name = name.into();
        if self.tokens.contains_key(&name) {
            return Err(PenteError::DuplicateToken {
                ramp: self.name.clone(),
                token: name,
            });
        }
        self.tokens.insert(name, origin);
        Ok(self)
    }

    /// Whether a token exists. The edge a `Binding` is validated against.
    #[must_use]
    pub fn contains(&self, token: &TokenName) -> bool {
        self.tokens.contains_key(token)
    }

    /// Resolve every token to a concrete colour.
    ///
    /// Total after validation: an unknown reference is `UnknownToken`, a
    /// cyclic one is `Cycle` with the full path. Both are compile-time
    /// failures of the authored data, never a render-time fallback.
    pub fn resolve(&self) -> Result<ResolvedRamp> {
        let mut out: HashMap<TokenName, Srgb> = HashMap::with_capacity(self.tokens.len());
        let mut state: HashMap<&TokenName, Visit> = HashMap::new();
        let mut stack: Vec<&TokenName> = Vec::new();

        for name in self.tokens.keys() {
            self.visit(name, &mut state, &mut stack, &mut out)?;
        }

        Ok(ResolvedRamp {
            name: self.name.clone(),
            colors: out,
        })
    }

    /// Depth-first resolution with an explicit tri-state mark, so a cycle is
    /// distinguished from a diamond. A diamond (two tokens aliasing a third)
    /// is perfectly legal and must NOT be reported as a cycle.
    fn visit<'a>(
        &'a self,
        name: &'a TokenName,
        state: &mut HashMap<&'a TokenName, Visit>,
        stack: &mut Vec<&'a TokenName>,
        out: &mut HashMap<TokenName, Srgb>,
    ) -> Result<Srgb> {
        match state.get(name) {
            Some(Visit::Done) => return Ok(out[name]),
            Some(Visit::InProgress) => {
                // Report the cycle from where it closes, not the whole stack.
                let start = stack.iter().position(|n| *n == name).unwrap_or(0);
                let mut path: Vec<String> =
                    stack[start..].iter().map(|n| n.to_string()).collect();
                path.push(name.to_string());
                return Err(PenteError::Cycle { path });
            }
            None => {}
        }

        let origin = self.tokens.get(name).ok_or_else(|| PenteError::UnknownToken {
            ramp: self.name.clone(),
            token: name.clone(),
        })?;

        state.insert(name, Visit::InProgress);
        stack.push(name);

        let srgb = match origin {
            Origin::Born { srgb } => *srgb,
            Origin::Alias { of } => self.visit(of, state, stack, out)?,
            Origin::Blend { over, with, alpha } => {
                let bg = self.visit(over, state, stack, out)?;
                let fg = self.visit(with, state, stack, out)?;
                to_srgb(blend_linear(to_rgb(bg), to_rgb(fg), alpha.get()))
            }
            Origin::Mix { a, b, t } => {
                // Mix is a linear-space interpolation, which is exactly a
                // blend of `b` over `a` at `t` — reuse rather than a second
                // interpolation with its own rounding behaviour.
                let ca = self.visit(a, state, stack, out)?;
                let cb = self.visit(b, state, stack, out)?;
                to_srgb(blend_linear(to_rgb(ca), to_rgb(cb), t.get()))
            }
        };

        stack.pop();
        state.insert(name, Visit::Done);
        out.insert(name.clone(), srgb);
        Ok(srgb)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visit {
    InProgress,
    Done,
}

/// A ramp with every token reduced to a concrete colour.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRamp {
    pub name: String,
    colors: HashMap<TokenName, Srgb>,
}

impl ResolvedRamp {
    #[must_use]
    pub fn get(&self, token: &TokenName) -> Option<Srgb> {
        self.colors.get(token).copied()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}

// `Rgb` and `Srgb` are structurally identical (three u8 channels) but live in
// different modules with different derives. These bridge rather than pick a
// winner, because unifying them is a separate deletion with its own blast
// radius (ledger table #2).
fn to_rgb(s: Srgb) -> Rgb {
    Rgb::new(s.r, s.g, s.b)
}

fn to_srgb(c: Rgb) -> Srgb {
    Srgb::new(c.r, c.g, c.b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::origin::UnitInterval;

    fn born(r: u8, g: u8, b: u8) -> Origin {
        Origin::Born {
            srgb: Srgb::new(r, g, b),
        }
    }

    #[test]
    fn resolves_born_alias_and_blend() {
        let mut ramp = Ramp::new("t");
        ramp.insert("bg", born(0, 0, 0)).unwrap();
        ramp.insert("fg", born(255, 255, 255)).unwrap();
        ramp.insert("alias_bg", Origin::Alias { of: "bg".into() })
            .unwrap();
        ramp.insert(
            "half",
            Origin::Blend {
                over: "bg".into(),
                with: "fg".into(),
                alpha: UnitInterval::new(1.0).unwrap(),
            },
        )
        .unwrap();

        let r = ramp.resolve().unwrap();
        assert_eq!(r.get(&"bg".into()).unwrap(), Srgb::new(0, 0, 0));
        assert_eq!(r.get(&"alias_bg".into()).unwrap(), Srgb::new(0, 0, 0));
        // alpha 1.0 == fully the `with` colour
        assert_eq!(r.get(&"half".into()).unwrap(), Srgb::new(255, 255, 255));
    }

    #[test]
    fn blend_is_byte_identical_to_ishou_tokens() {
        // Pente must not become a SECOND blend implementation; the whole
        // point is that it reuses the recipe vellum_matrix.rs already pins.
        let mut ramp = Ramp::new("t");
        ramp.insert("bg", born(0x2E, 0x34, 0x40)).unwrap();
        ramp.insert("ac", born(0x88, 0xC0, 0xD0)).unwrap();
        ramp.insert(
            "sel",
            Origin::Blend {
                over: "bg".into(),
                with: "ac".into(),
                alpha: UnitInterval::new(0.30).unwrap(),
            },
        )
        .unwrap();

        let got = ramp.resolve().unwrap().get(&"sel".into()).unwrap();
        let want = blend_linear(Rgb::new(0x2E, 0x34, 0x40), Rgb::new(0x88, 0xC0, 0xD0), 0.30);
        assert_eq!((got.r, got.g, got.b), (want.r, want.g, want.b));
    }

    #[test]
    fn unknown_token_is_caught_at_resolve_not_at_render() {
        let mut ramp = Ramp::new("t");
        ramp.insert("a", Origin::Alias { of: "nope".into() }).unwrap();
        let e = ramp.resolve().unwrap_err();
        assert_eq!(
            e,
            PenteError::UnknownToken {
                ramp: "t".into(),
                token: "nope".into()
            }
        );
    }

    #[test]
    fn cycle_reports_the_path_not_just_the_fact() {
        let mut ramp = Ramp::new("t");
        ramp.insert("a", Origin::Alias { of: "b".into() }).unwrap();
        ramp.insert("b", Origin::Alias { of: "a".into() }).unwrap();
        match ramp.resolve().unwrap_err() {
            PenteError::Cycle { path } => {
                assert!(path.len() >= 2, "cycle path should name the loop: {path:?}");
                assert_eq!(path.first(), path.last());
            }
            other => panic!("expected Cycle, got {other:?}"),
        }
    }

    #[test]
    fn a_diamond_is_not_a_cycle() {
        // Two tokens aliasing a third is legal. A naive "already visiting"
        // check reports this as a cycle; the tri-state mark must not.
        let mut ramp = Ramp::new("t");
        ramp.insert("base", born(1, 2, 3)).unwrap();
        ramp.insert("l", Origin::Alias { of: "base".into() }).unwrap();
        ramp.insert("r", Origin::Alias { of: "base".into() }).unwrap();
        ramp.insert(
            "top",
            Origin::Blend {
                over: "l".into(),
                with: "r".into(),
                alpha: UnitInterval::new(0.5).unwrap(),
            },
        )
        .unwrap();
        assert!(ramp.resolve().is_ok());
    }

    #[test]
    fn duplicate_token_is_rejected_not_last_write_wins() {
        let mut ramp = Ramp::new("t");
        ramp.insert("a", born(0, 0, 0)).unwrap();
        assert!(ramp.insert("a", born(1, 1, 1)).is_err());
    }
}
