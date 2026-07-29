//! # ishou-pente
//!
//! *Pente* (Brazilian-Portuguese): the loom's **reed** — the comb every warp
//! thread passes through, holding each at fixed even spacing and beating every
//! weft home. The single component in a loom whose entire job is *uniformity
//! across the whole width of the cloth*.
//!
//! Every colour, every role, every face the fleet renders passes through one
//! pente. Canonical spec: `pleme-io/theory/PENTE.md`.
//!
//! ## The one rule that makes it land
//!
//! The fleet does not lack a token source — `irodori::NORD` ->
//! `ishou_tokens::ColorPalette::pleme()` is a correct derivation. What it
//! lacks is **an authoring border, a closed vocabulary, and a forced
//! consumption path**. This crate is those three.
//!
//! ## The invariants, at their HONEST tiers
//!
//! Graded per `theory/UNREPRESENTABILITY.md`. A `Result::Err` is *mitigation*;
//! a compile error or an absent method is *unrepresentability*. Nothing here
//! is rounded up.
//!
//! | Invariant | Technique | Tier |
//! |---|---|---|
//! | A theme missing a role cannot be built | `Binding` is a product struct | **truly-unrepresentable** (`E0063`) — but INHERITED from `SemanticRoles`, not invented here |
//! | A face cannot resolve a raw token | no public `Pente` method accepts a `TokenName` | **truly-unrepresentable** (absent method) — *for crates that link this one* |
//! | A new `CoreRole` cannot be silently ignored | exhaustive matches, no `_` arms | **truly-unrepresentable** (compile error) |
//! | A colour literal outside a ramp | only `Origin::Born` has an `Srgb` field | **truly-unrepresentable** *inside pente-linking crates*; CI-caught elsewhere |
//! | An alpha outside `[0,1]` | `UnitInterval::new` rejects (never clamps) | **parse-time-rejected** |
//! | A binding naming a token absent from its ramp | `Binding::validate` | **parse-time-rejected** |
//! | A cyclic token definition | tri-state DFS in `Ramp::resolve` | **parse-time-rejected** |
//! | Face-role completeness vs `consumes` | catalog test | **CI-caught** |
//!
//! ## Tier-honest scope of THIS milestone (M0, step 1 of 6)
//!
//! What is here: the typed algebra + its tests.
//!
//! What is NOT here, and is named rather than implied:
//! - **No `#[derive(DeriveTataraDomain)]` border and no `(deframp …)` /
//!   `(defbinding …)` / `(defface …)` loader.** The ishou workspace pins no
//!   tatara-lisp today, and the two lineages are mutually incompatible
//!   (`KeywordSexp` exists only on the standalone line; `parse_kwargs_strict`
//!   only on the monorepo line). Adding that dep to a crate escriba, garasu
//!   and madori all depend on is a flag-day, so it is **M1**, deliberately.
//!   `specs/fleet.pente.tlisp` documents the destination form.
//! - **No authored fleet ramp yet.** `nord` and `vellum` are transcribed in
//!   M0 step 2, against a parity oracle. Until that lands this crate has the
//!   algebra and no data.
//! - **No renderer and no deletions.** M0's exit criterion is a DELETION
//!   (`nix/lib/skim-theme.nix` and skim-tab's colour consts), and nothing is
//!   deleted yet. An emission without its deletion is a 23rd palette home.

pub mod binding;
pub mod error;
pub mod origin;
pub mod ramp;
pub mod role;

pub use binding::Binding;
pub use error::{PenteError, Result};
pub use origin::{Origin, TokenName, UnitInterval};
pub use ramp::{Ramp, ResolvedRamp};
pub use role::{CoreRole, FaceRole, Role};

use std::collections::BTreeMap;

use ishou_tokens::Srgb;

/// A face's declaration: which surface it is and which face-roles it consumes.
///
/// Declaration as data, projection as code — the projection (how a face turns
/// resolved colours into its own artifact) is a renderer, not part of the
/// algebra.
#[derive(Debug, Clone, PartialEq)]
pub struct FaceSpec {
    /// `gpu`, `terminal`, `web`, `shell`, `vt`, `nix`, `cli`.
    pub name: String,
    /// Face-roles this face may bind. Completeness is CI-checked.
    pub consumes: Vec<FaceRole>,
    /// Face-role -> token, for the open tier only. Core roles come from the
    /// `Binding`, so a face cannot disagree with the theme about them.
    pub face_tokens: BTreeMap<FaceRole, TokenName>,
}

/// The assembled spine: one ramp, one binding, N faces.
///
/// The ONLY public resolution entry point is [`Pente::resolve`], and it takes
/// a [`Role`]. There is deliberately no public method accepting a
/// [`TokenName`], which is what makes "a face may name a Role and nothing
/// else" an *absent method* rather than a convention.
#[derive(Debug, Clone)]
pub struct Pente {
    ramp: ResolvedRamp,
    binding: Binding,
    faces: BTreeMap<String, FaceSpec>,
}

impl Pente {
    /// Assemble and validate. Every edge is checked here so that `resolve`
    /// is total afterwards.
    pub fn compile(ramp: &Ramp, binding: Binding, faces: Vec<FaceSpec>) -> Result<Self> {
        // 1. Resolve the ramp — catches cycles and dangling references.
        let resolved = ramp.resolve()?;
        // 2. Validate the binding against it — catches typo'd role tokens.
        binding.validate(ramp)?;
        // 3. Validate each face's tokens, and that it declared what it binds.
        let mut map = BTreeMap::new();
        for face in faces {
            for (role, token) in &face.face_tokens {
                if !face.consumes.contains(role) {
                    return Err(PenteError::UndeclaredFaceRole {
                        face: face.name.clone(),
                        role: role.to_string(),
                    });
                }
                if !ramp.contains(token) {
                    return Err(PenteError::UnknownToken {
                        ramp: ramp.name.clone(),
                        token: token.clone(),
                    });
                }
            }
            map.insert(face.name.clone(), face);
        }
        Ok(Self {
            ramp: resolved,
            binding,
            faces: map,
        })
    }

    /// Resolve a role, for a face, to a concrete colour.
    ///
    /// Core roles resolve from the binding and are therefore identical across
    /// every face — that uniformity is the whole point of the reed. Face
    /// roles resolve from that face's own map.
    pub fn resolve(&self, face: &str, role: &Role) -> Result<Srgb> {
        match role {
            Role::Core(core) => {
                let token = self.binding.token(*core);
                self.ramp.get(token).ok_or_else(|| PenteError::UnknownToken {
                    ramp: self.ramp.name.clone(),
                    token: token.clone(),
                })
            }
            Role::Face(fr) => {
                let spec = self.faces.get(face).ok_or_else(|| PenteError::UnboundRole {
                    face: face.to_string(),
                    role: fr.to_string(),
                })?;
                let token = spec.face_tokens.get(fr).ok_or_else(|| PenteError::UnboundRole {
                    face: face.to_string(),
                    role: fr.to_string(),
                })?;
                self.ramp.get(token).ok_or_else(|| PenteError::UnknownToken {
                    ramp: self.ramp.name.clone(),
                    token: token.clone(),
                })
            }
        }
    }

    /// The binding's name — `nord`, `vellum-night`.
    #[must_use]
    pub fn binding_name(&self) -> &str {
        &self.binding.name
    }

    /// Core-role coverage, for the catalog test that reports the
    /// CoreRole/FaceRole ratio per release so vocabulary drift into seven
    /// per-face dialects is visible rather than silent.
    #[must_use]
    pub fn face_role_count(&self) -> usize {
        self.faces.values().map(|f| f.face_tokens.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp_with(tokens: &[(&str, u8)]) -> Ramp {
        let mut r = Ramp::new("r");
        for (n, v) in tokens {
            r.insert(
                *n,
                Origin::Born {
                    srgb: Srgb::new(*v, *v, *v),
                },
            )
            .unwrap();
        }
        r
    }

    fn binding_on(token: &str) -> Binding {
        let t = || TokenName::new(token);
        Binding {
            name: "b".into(),
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
    fn core_roles_resolve_identically_across_faces() {
        // The reed property: uniformity across the whole width of the cloth.
        let ramp = ramp_with(&[("base", 7)]);
        let faces = vec![
            FaceSpec {
                name: "gpu".into(),
                consumes: vec![],
                face_tokens: BTreeMap::new(),
            },
            FaceSpec {
                name: "terminal".into(),
                consumes: vec![],
                face_tokens: BTreeMap::new(),
            },
        ];
        let p = Pente::compile(&ramp, binding_on("base"), faces).unwrap();
        let role = Role::Core(CoreRole::Accent);
        assert_eq!(
            p.resolve("gpu", &role).unwrap(),
            p.resolve("terminal", &role).unwrap()
        );
    }

    #[test]
    fn a_face_binding_an_undeclared_face_role_is_rejected() {
        let ramp = ramp_with(&[("base", 7), ("ansi", 9)]);
        let mut face_tokens = BTreeMap::new();
        face_tokens.insert(FaceRole::new("terminal", "ansi-0"), TokenName::new("ansi"));
        let faces = vec![FaceSpec {
            name: "terminal".into(),
            consumes: vec![], // declared nothing
            face_tokens,
        }];
        assert!(matches!(
            Pente::compile(&ramp, binding_on("base"), faces),
            Err(PenteError::UndeclaredFaceRole { .. })
        ));
    }

    #[test]
    fn compile_rejects_a_binding_whose_ramp_lacks_the_token() {
        let ramp = ramp_with(&[("base", 7)]);
        assert!(matches!(
            Pente::compile(&ramp, binding_on("missing"), vec![]),
            Err(PenteError::UnknownToken { .. })
        ));
    }

    #[test]
    fn face_roles_are_per_face_not_global() {
        let ramp = ramp_with(&[("base", 7), ("ansi", 9)]);
        let role = FaceRole::new("terminal", "ansi-0");
        let mut face_tokens = BTreeMap::new();
        face_tokens.insert(role.clone(), TokenName::new("ansi"));
        let faces = vec![
            FaceSpec {
                name: "terminal".into(),
                consumes: vec![role.clone()],
                face_tokens,
            },
            FaceSpec {
                name: "gpu".into(),
                consumes: vec![],
                face_tokens: BTreeMap::new(),
            },
        ];
        let p = Pente::compile(&ramp, binding_on("base"), faces).unwrap();
        let r = Role::Face(role);
        assert!(p.resolve("terminal", &r).is_ok());
        // The GPU face never declared it, so asking is a typed error rather
        // than a silent fallback to some other colour.
        assert!(p.resolve("gpu", &r).is_err());
    }
}
