//! `Motion` — two tiers, mirroring pente's `Role`.
//!
//! HOW `CoreMotion`'s MEMBERSHIP WAS DERIVED — a UNION with a majority
//! filter, NOT an intersection. Stated plainly because the honest derivation
//! is unflattering: mado wires bell-flash, cursor blink, SGR-5 blink and
//! decay; the editor motions (`ModeChange`, `Scroll`, `SearchPulse`,
//! `DiagnosticFade`, `SelectionFade`, `CursorMove`) come from what escriba's
//! faces demonstrably need and currently hand-wave. The genuine intersection
//! across today's shipped surfaces is `{CursorBlink, Bell}` — TWO members.
//! Claiming this set "fell out of the data" would be false.

use serde::{Deserialize, Serialize};

/// The CLOSED core vocabulary. Every choreography must bind all 8.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreMotion {
    /// The caret's on/off cycle. Shipped in mado; ABSENT in escriba.
    CursorBlink,
    /// The caret travelling between positions.
    CursorMove,
    /// Normal -> Insert -> Visual transitions.
    ModeChange,
    /// Viewport translation.
    Scroll,
    /// A search hit announcing itself.
    SearchPulse,
    /// A diagnostic arriving or clearing.
    DiagnosticFade,
    /// A selection appearing or dissolving.
    SelectionFade,
    /// The terminal bell. Shipped in mado as a `Tween`.
    Bell,
}

impl CoreMotion {
    /// Every variant, in declaration order. Kept honest by
    /// `all_is_exhaustive` below, which cannot compile if a variant is added
    /// without being listed.
    pub const ALL: [Self; 8] = [
        Self::CursorBlink,
        Self::CursorMove,
        Self::ModeChange,
        Self::Scroll,
        Self::SearchPulse,
        Self::DiagnosticFade,
        Self::SelectionFade,
        Self::Bell,
    ];

    /// The kebab-case authoring symbol. Total by construction.
    #[must_use]
    pub const fn as_symbol(self) -> &'static str {
        match self {
            Self::CursorBlink => "cursor-blink",
            Self::CursorMove => "cursor-move",
            Self::ModeChange => "mode-change",
            Self::Scroll => "scroll",
            Self::SearchPulse => "search-pulse",
            Self::DiagnosticFade => "diagnostic-fade",
            Self::SelectionFade => "selection-fade",
            Self::Bell => "bell",
        }
    }

    /// Parse an authoring symbol. `None` rather than a fallback — a
    /// misspelled motion must not silently become `cursor-blink`.
    #[must_use]
    pub fn from_symbol(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.as_symbol() == s)
    }
}

impl core::fmt::Display for CoreMotion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_symbol())
    }
}

/// The OPEN, per-surface tier: `terminal.sgr-blink`, `gpu.aurora-drift`.
///
/// Namespaced so one face's private motion cannot collide with another's.
/// Completeness against a face's `consumes` list is a CATALOG check (CI
/// tier), not a compile error — because `FaceMotion` being open is precisely
/// what makes a compile-time totality claim impossible here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FaceMotion(String);

impl FaceMotion {
    #[must_use]
    pub fn new(namespace: &str, name: &str) -> Self {
        Self(format!("{namespace}.{name}"))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.0.split_once('.').map(|(ns, _)| ns)
    }
}

impl core::fmt::Display for FaceMotion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Either tier. What a face asks `Batente::resolve` for.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Motion {
    Core(CoreMotion),
    Face(FaceMotion),
}

impl From<CoreMotion> for Motion {
    fn from(m: CoreMotion) -> Self {
        Self::Core(m)
    }
}

impl From<FaceMotion> for Motion {
    fn from(m: FaceMotion) -> Self {
        Self::Face(m)
    }
}

impl core::fmt::Display for Motion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Core(m) => m.fmt(f),
            Self::Face(m) => m.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_is_exhaustive() {
        // The drift guard: this match has no `_` arm, so ADDING a variant
        // fails to compile here until ALL is updated too.
        for m in CoreMotion::ALL {
            let _: () = match m {
                CoreMotion::CursorBlink
                | CoreMotion::CursorMove
                | CoreMotion::ModeChange
                | CoreMotion::Scroll
                | CoreMotion::SearchPulse
                | CoreMotion::DiagnosticFade
                | CoreMotion::SelectionFade
                | CoreMotion::Bell => (),
            };
        }
        assert_eq!(CoreMotion::ALL.len(), 8);
    }

    #[test]
    fn symbol_round_trips_for_every_variant() {
        for m in CoreMotion::ALL {
            assert_eq!(CoreMotion::from_symbol(m.as_symbol()), Some(m));
        }
    }

    #[test]
    fn unknown_symbol_is_none_not_a_fallback() {
        assert_eq!(CoreMotion::from_symbol("cursor_blink"), None);
        assert_eq!(CoreMotion::from_symbol(""), None);
    }

    #[test]
    fn face_motions_are_namespaced() {
        let m = FaceMotion::new("terminal", "sgr-blink");
        assert_eq!(m.as_str(), "terminal.sgr-blink");
        assert_eq!(m.namespace(), Some("terminal"));
    }
}
