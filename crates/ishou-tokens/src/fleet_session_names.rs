//! Fleet session names — artistic, memorable session identities.
//!
//! tear mints a `SessionId` as a BLAKE3→8-byte hex (`e4be4b01840970a4`) — fine
//! as a stable internal key, terrible as a thing a human glances at and recalls.
//! This atlas gives every session a **name** instead: a curated glyph + a short
//! evocative word (`🌊 tide`, `❄ frost`, `🔋 volt`). Deterministic from a seed,
//! uniquified against the live set, so the same session always reads the same.
//!
//! Two STYLES (the operator chooses; default `Emoji`):
//! * `Emoji` — wide emoji + word. The one playful surface that intentionally
//!   departs from the fleet "clean single-width Unicode, no wide emoji" rule —
//!   session identities are meant to be glanceable and characterful.
//! * `Glyph` — Nerd-Font / single-width glyph + word, for operators who keep
//!   the strict clean-Unicode aesthetic everywhere.
//!
//! The word pool is shared; only the leading glyph differs by style, so a
//! session's *word* is stable across a style switch.

use core::fmt;

/// How a session name renders its leading mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionNameStyle {
    /// Wide emoji + word (default — `🌊 tide`).
    #[default]
    Emoji,
    /// Clean single-width glyph + word (`✦ tide`) — strict fleet aesthetic.
    Glyph,
}

/// One curated session identity: a stable word with both a wide-emoji and a
/// clean-glyph mark, so the *word* survives a style change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionIdentity {
    pub emoji: &'static str,
    pub glyph: &'static str,
    pub word: &'static str,
}

/// A resolved session name (an identity + the style to render it in). `Display`
/// is the ONLY render surface (typed-emission discipline — no ad-hoc `format!`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionName {
    pub identity: SessionIdentity,
    pub style: SessionNameStyle,
}

impl fmt::Display for SessionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mark = match self.style {
            SessionNameStyle::Emoji => self.identity.emoji,
            SessionNameStyle::Glyph => self.identity.glyph,
        };
        write!(f, "{mark} {}", self.identity.word)
    }
}

/// The curated atlas. Elemental / celestial / natural words in the Vellum
/// register (warm, calm, a little wild) — each paired with an emoji and a
/// clean single-width Nerd-Font/Unicode glyph alternative.
pub struct FleetSessionNames;

impl FleetSessionNames {
    /// The pool. Order is stable — `pick` indexes into it deterministically.
    pub const POOL: &'static [SessionIdentity] = &[
        SessionIdentity { emoji: "🌊", glyph: "\u{2248}", word: "tide" },     // ≈
        SessionIdentity { emoji: "❄",  glyph: "\u{2744}", word: "frost" },    // ❄
        SessionIdentity { emoji: "🔥", glyph: "\u{2588}", word: "ember" },    // █
        SessionIdentity { emoji: "🌲", glyph: "\u{25b2}", word: "cedar" },    // ▲
        SessionIdentity { emoji: "🔋", glyph: "\u{26a1}", word: "volt" },     // ⚡ (single-width)
        SessionIdentity { emoji: "🌙", glyph: "\u{263d}", word: "luna" },     // ☽
        SessionIdentity { emoji: "🪨", glyph: "\u{25c6}", word: "stone" },    // ◆
        SessionIdentity { emoji: "🍃", glyph: "\u{2767}", word: "fern" },     // ❧
        SessionIdentity { emoji: "☄",  glyph: "\u{2736}", word: "comet" },    // ✶
        SessionIdentity { emoji: "🌋", glyph: "\u{25b3}", word: "magma" },    // △
        SessionIdentity { emoji: "🐚", glyph: "\u{273f}", word: "shell" },    // ✿
        SessionIdentity { emoji: "🦊", glyph: "\u{25d5}", word: "fox" },      // ◕
        SessionIdentity { emoji: "🦉", glyph: "\u{25c9}", word: "owl" },      // ◉
        SessionIdentity { emoji: "🧭", glyph: "\u{2316}", word: "compass" },  // ⌖
        SessionIdentity { emoji: "🌾", glyph: "\u{2058}", word: "wheat" },    // ⁘
        SessionIdentity { emoji: "🪵", glyph: "\u{2261}", word: "timber" },   // ≡
        SessionIdentity { emoji: "🌒", glyph: "\u{25d1}", word: "dusk" },     // ◑
        SessionIdentity { emoji: "🌅", glyph: "\u{25d0}", word: "dawn" },     // ◐
        SessionIdentity { emoji: "🏔", glyph: "\u{25b4}", word: "ridge" },    // ▴
        SessionIdentity { emoji: "🌀", glyph: "\u{29bf}", word: "vortex" },   // ⦿
        SessionIdentity { emoji: "🪶", glyph: "\u{2710}", word: "quill" },    // ✐
        SessionIdentity { emoji: "🔭", glyph: "\u{2295}", word: "scope" },    // ⊕
        SessionIdentity { emoji: "⚓", glyph: "\u{2693}", word: "anchor" },   // ⚓ (single-width)
        SessionIdentity { emoji: "🕯", glyph: "\u{2020}", word: "taper" },    // †
        SessionIdentity { emoji: "🌿", glyph: "\u{273d}", word: "sage" },     // ✽
        SessionIdentity { emoji: "🪐", glyph: "\u{2641}", word: "orbit" },    // ♁
        SessionIdentity { emoji: "🦌", glyph: "\u{2638}", word: "stag" },     // ☸
        SessionIdentity { emoji: "🐋", glyph: "\u{223f}", word: "whale" },    // ∿
        SessionIdentity { emoji: "🜂", glyph: "\u{25b3}", word: "spark" },    // △
        SessionIdentity { emoji: "🌑", glyph: "\u{25cf}", word: "void" },     // ●
        SessionIdentity { emoji: "🪷", glyph: "\u{2740}", word: "lotus" },    // ❀
        SessionIdentity { emoji: "🦅", glyph: "\u{25b7}", word: "hawk" },     // ▷
    ];

    /// Deterministic identity for a seed (e.g. tear's session counter or a hash
    /// of the SessionId). Same seed → same identity, always.
    #[must_use]
    pub fn identity(seed: u64) -> SessionIdentity {
        Self::POOL[(seed as usize) % Self::POOL.len()]
    }

    /// A resolved [`SessionName`] for a seed in a style.
    #[must_use]
    pub fn name(seed: u64, style: SessionNameStyle) -> SessionName {
        SessionName { identity: Self::identity(seed), style }
    }

    /// Pick a name UNIQUE against a set of already-used words, scanning forward
    /// from the seed. Falls back to the seed's identity once the pool is
    /// exhausted (collisions then disambiguated by the caller via the hex id).
    #[must_use]
    pub fn pick_unique(seed: u64, taken: &[&str], style: SessionNameStyle) -> SessionName {
        let n = Self::POOL.len() as u64;
        for off in 0..n {
            let id = Self::POOL[((seed + off) % n) as usize];
            if !taken.contains(&id.word) {
                return SessionName { identity: id, style };
            }
        }
        Self::name(seed, style)
    }

    /// Deterministic identity for a project ROOT path — the automation core.
    /// The same project always maps to the same identity (`~/code/.../mado`
    /// → always `🌊 tide`), so cd-driven auto-attach is stable + memorable
    /// across daemon restarts. Seeds from a STABLE hash (NOT std's
    /// per-process-randomized `DefaultHasher`, which would re-name a project
    /// on every restart).
    #[must_use]
    pub fn identity_for_path(path: &std::path::Path) -> SessionIdentity {
        Self::identity(stable_seed(path.to_string_lossy().as_bytes()))
    }

    /// Deterministic [`SessionName`] for a project root, in a style.
    #[must_use]
    pub fn from_project_path(path: &std::path::Path, style: SessionNameStyle) -> SessionName {
        SessionName { identity: Self::identity_for_path(path), style }
    }
}

/// FNV-1a over bytes — a STABLE (run-to-run identical) 64-bit hash. The
/// automation binds a project root to a session name by this seed; it must
/// never change for a given path, so we cannot use `std`'s `DefaultHasher`
/// (randomized per process — it would re-name every project on restart).
#[must_use]
pub fn stable_seed(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325; // FNV offset basis
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3); // FNV prime
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_deterministic() {
        assert_eq!(FleetSessionNames::identity(7), FleetSessionNames::identity(7));
        assert_eq!(
            FleetSessionNames::identity(7).word,
            FleetSessionNames::identity(7 + FleetSessionNames::POOL.len() as u64).word
        );
    }

    #[test]
    fn word_is_stable_across_styles() {
        let e = FleetSessionNames::name(3, SessionNameStyle::Emoji);
        let g = FleetSessionNames::name(3, SessionNameStyle::Glyph);
        assert_eq!(e.identity.word, g.identity.word);
        assert_ne!(e.to_string(), g.to_string()); // mark differs
    }

    #[test]
    fn display_is_mark_space_word() {
        let n = SessionName {
            identity: SessionIdentity { emoji: "🌊", glyph: "≈", word: "tide" },
            style: SessionNameStyle::Emoji,
        };
        assert_eq!(n.to_string(), "🌊 tide");
        let g = SessionName { style: SessionNameStyle::Glyph, ..n };
        assert_eq!(g.to_string(), "≈ tide");
    }

    #[test]
    fn pick_unique_avoids_taken_words() {
        let taken = ["tide", "frost"];
        let picked = FleetSessionNames::pick_unique(0, &taken, SessionNameStyle::Emoji);
        assert!(!taken.contains(&picked.identity.word));
    }

    #[test]
    fn pool_words_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for id in FleetSessionNames::POOL {
            assert!(seen.insert(id.word), "duplicate word in pool: {}", id.word);
        }
        assert!(FleetSessionNames::POOL.len() >= 24, "pool should be large enough to avoid frequent collisions");
    }

    #[test]
    fn project_path_name_is_stable_and_deterministic() {
        use std::path::Path;
        let p = Path::new("/Users/x/code/github/pleme-io/mado");
        let a = FleetSessionNames::from_project_path(p, SessionNameStyle::Emoji);
        let b = FleetSessionNames::from_project_path(p, SessionNameStyle::Emoji);
        // Same project → same name, every time (stable across "restarts").
        assert_eq!(a.to_string(), b.to_string());
        assert_eq!(a.identity.word, b.identity.word);
        // The seed hash is pure + path-sensitive.
        assert_eq!(stable_seed(b"mado"), stable_seed(b"mado"));
        assert_ne!(stable_seed(b"mado"), stable_seed(b"nix"));
        // A different project resolves to a real identity.
        let other = FleetSessionNames::identity_for_path(Path::new("/Users/x/code/github/pleme-io/nix"));
        assert!(!other.word.is_empty());
    }
}
