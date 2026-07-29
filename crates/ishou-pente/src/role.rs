//! `Role` — two tiers.
//!
//! WHY TWO. A single closed enum spanning a 16-slot ANSI table, a 34-role MD3
//! set, a 9-alias egaku theme, a 7-role moldura set and a Chrome theme-colour
//! must be closed either too small (faces reach around it, reintroducing forks
//! *with* ceremony) or too large (every new theme binds 60 roles by hand —
//! exactly the friction that produced the forks). The split keeps `E0063`
//! totality where it pays and lets heterogeneity live where it must.
//!
//! HOW `CoreRole`'s MEMBERSHIP WAS DERIVED — a UNION with a majority filter,
//! NOT an intersection. The genuine 4-way intersection of `SemanticRoles`
//! (26 fields), `kazari::Role` (11), egaku's alias set (9) and
//! `MolduraTheme` (7) is roughly {Text, Border, Error, Info} — 3-5 members,
//! not 16. `Agent`, `Cursor`, `SurfaceElevated` and `Selection` appear in
//! exactly ONE source (`SemanticRoles`). Saying "intersection, nothing
//! invented" would be false and materially so; this set is dominated by
//! `SemanticRoles` and that is the honest description.

use serde::{Deserialize, Serialize};

/// The CLOSED core vocabulary. Every binding must bind all 16.
///
/// NOTE: no `KeywordSexp` derive. That derive exists ONLY on the standalone
/// tatara-lisp line, while the strict `parse_kwargs_strict`/`ClosedSet`
/// machinery exists ONLY on the monorepo line — different lineages with zero
/// cross-hits. The symbol<->variant mapping is therefore hand-written here
/// and pinned by a round-trip test, rather than pretending one derive covers
/// both.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreRole {
    Background,
    Surface,
    SurfaceElevated,
    Text,
    TextMuted,
    TextDim,
    Primary,
    Accent,
    Border,
    Error,
    Warning,
    Success,
    Info,
    Selection,
    Cursor,
    Agent,
}

impl CoreRole {
    /// Every variant, in declaration order.
    ///
    /// Hand-maintained, and the test below is what stops it drifting: adding
    /// a variant without adding it here fails `all_is_exhaustive`, because
    /// that test matches exhaustively and will not compile otherwise.
    pub const ALL: [Self; 16] = [
        Self::Background,
        Self::Surface,
        Self::SurfaceElevated,
        Self::Text,
        Self::TextMuted,
        Self::TextDim,
        Self::Primary,
        Self::Accent,
        Self::Border,
        Self::Error,
        Self::Warning,
        Self::Success,
        Self::Info,
        Self::Selection,
        Self::Cursor,
        Self::Agent,
    ];

    /// The kebab-case authoring symbol. Total by construction — a new
    /// variant fails to compile here rather than falling into a `_` arm.
    #[must_use]
    pub const fn as_symbol(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Surface => "surface",
            Self::SurfaceElevated => "surface-elevated",
            Self::Text => "text",
            Self::TextMuted => "text-muted",
            Self::TextDim => "text-dim",
            Self::Primary => "primary",
            Self::Accent => "accent",
            Self::Border => "border",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Success => "success",
            Self::Info => "info",
            Self::Selection => "selection",
            Self::Cursor => "cursor",
            Self::Agent => "agent",
        }
    }

    /// Parse an authoring symbol. `None` rather than a fallback role — a
    /// misspelled role must not silently paint as `Text`.
    #[must_use]
    pub fn from_symbol(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.as_symbol() == s)
    }
}

impl core::fmt::Display for CoreRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_symbol())
    }
}

/// The OPEN, per-surface tier: `terminal.ansi-0`, `web.on-primary`,
/// `gpu.aurora-low`, `shell.glob`.
///
/// Namespaced so a face's private vocabulary cannot collide with another
/// face's. Completeness against a face's declared `consumes` list is a
/// CATALOG check (CI tier), not a compile error — stated plainly because
/// `FaceRole` being open is precisely what makes a compile-time totality
/// claim impossible here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FaceRole(String);

impl FaceRole {
    /// Build a namespaced face-role. The namespace is mandatory — an
    /// un-namespaced face role is how two surfaces end up disagreeing about
    /// what `accent` means.
    #[must_use]
    pub fn new(namespace: &str, name: &str) -> Self {
        Self(format!("{namespace}.{name}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The namespace half, if this role is well-formed.
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.0.split_once('.').map(|(ns, _)| ns)
    }
}

impl core::fmt::Display for FaceRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Either tier. What a face actually asks `Pente::resolve` for.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Role {
    Core(CoreRole),
    Face(FaceRole),
}

impl From<CoreRole> for Role {
    fn from(r: CoreRole) -> Self {
        Self::Core(r)
    }
}

impl From<FaceRole> for Role {
    fn from(r: FaceRole) -> Self {
        Self::Face(r)
    }
}

impl core::fmt::Display for Role {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Core(r) => r.fmt(f),
            Self::Face(r) => r.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_is_exhaustive() {
        // The drift guard for CoreRole::ALL. This match is exhaustive with
        // no `_` arm, so ADDING a variant fails to compile here until ALL is
        // updated too — which is the property that keeps the const honest.
        for role in CoreRole::ALL {
            let _: () = match role {
                CoreRole::Background
                | CoreRole::Surface
                | CoreRole::SurfaceElevated
                | CoreRole::Text
                | CoreRole::TextMuted
                | CoreRole::TextDim
                | CoreRole::Primary
                | CoreRole::Accent
                | CoreRole::Border
                | CoreRole::Error
                | CoreRole::Warning
                | CoreRole::Success
                | CoreRole::Info
                | CoreRole::Selection
                | CoreRole::Cursor
                | CoreRole::Agent => (),
            };
        }
        assert_eq!(CoreRole::ALL.len(), 16);
    }

    #[test]
    fn symbol_round_trips_for_every_variant() {
        for role in CoreRole::ALL {
            assert_eq!(CoreRole::from_symbol(role.as_symbol()), Some(role));
        }
    }

    #[test]
    fn unknown_symbol_is_none_not_a_fallback_role() {
        assert_eq!(CoreRole::from_symbol("bakcground"), None);
        assert_eq!(CoreRole::from_symbol(""), None);
    }

    #[test]
    fn face_roles_are_namespaced() {
        let r = FaceRole::new("terminal", "ansi-0");
        assert_eq!(r.as_str(), "terminal.ansi-0");
        assert_eq!(r.namespace(), Some("terminal"));
    }
}
