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

/// The aesthetic register a session identity belongs to. A freshly-created
/// session draws a random name from a chosen theme; project-bound sessions
/// stay deterministic by path (see [`FleetSessionNames::from_project_path`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionTheme {
    /// Elemental / celestial / natural — cool and a little wild
    /// (`🌊 tide`, `❄ frost`, `🌙 luna`). The fleet default.
    #[default]
    Frost,
    /// Warm tropical — a nod to the operator (`🌴 palmeira`, `☀ sol`,
    /// `🥁 samba`, `🦜 arara`).
    Brazil,
}

/// One curated session identity: a stable word with both a wide-emoji and a
/// clean-glyph mark (so the *word* survives a style change), a theme, and a
/// set of search keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionIdentity {
    pub emoji: &'static str,
    pub glyph: &'static str,
    pub word: &'static str,
    /// Aesthetic register (frost / brazil) — the pool a themed-random new
    /// session draws from.
    pub theme: SessionTheme,
    /// Search aliases: typing any of these in the Ctrl-S picker finds this
    /// session (`wave`/`water`/`sea` → `🌊 tide`). The `word` is always
    /// matchable too; keywords add the synonyms a human would actually type.
    pub keywords: &'static [&'static str],
}

impl SessionIdentity {
    /// Does `query` (case-insensitive) match this identity's word or any of
    /// its keywords as a substring? Powers "type `wave`, find `🌊 tide`".
    #[must_use]
    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        self.word.to_lowercase().contains(&q)
            || self.keywords.iter().any(|k| k.to_lowercase().contains(&q))
    }
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

/// What a session is being named FROM — the single input to the fleet's
/// one naming authority, [`FleetSessionNames::resolve`]. Every
/// session-name decision (mado boot, picker create, MCP spawn, the daemon
/// hook) maps a context through `resolve`, so a session has exactly ONE
/// identity, rendered per surface (glyph → the text prompt, emoji → the
/// GUI picker). No surface decides a name on its own.
#[derive(Debug, Clone, Copy)]
pub enum NameContext<'a> {
    /// The operator pinned an explicit name (e.g. `tear.session_name`) —
    /// used verbatim, no curated mark.
    Override(&'a str),
    /// Name derived from a PROJECT ROOT — the deterministic, path-stable
    /// curated identity (same seed as [`FleetSessionNames::from_project_path`]).
    Project(&'a std::path::Path),
    /// No project + no override — the branded fleet
    /// [`FleetSessionNames::DEFAULT`] (`🌑 rime`).
    Default,
}

/// The output of the [`FleetSessionNames::resolve`] authority: either a
/// curated [`SessionIdentity`] (renderable in BOTH styles — the basis for
/// the split "emoji in the GUI / glyph in the prompt" projection) or a
/// literal operator string (one mark-free form).
///
/// This resolved value is the single source of truth for a session's
/// name, minted ONCE at creation. Every surface renders it (never derives
/// an independent one), so the picker and the prompt can never disagree on
/// the session's word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedName {
    /// A curated identity — render with [`Self::render`] in either style.
    Identity(SessionIdentity),
    /// A verbatim operator string — style-agnostic (no leading mark).
    Literal(String),
}

impl ResolvedName {
    /// Render this name in `style`. A [`Self::Literal`] ignores style
    /// (operator strings carry no curated mark); an [`Self::Identity`]
    /// renders its emoji/glyph mark + word via [`SessionName`]'s `Display`.
    #[must_use]
    pub fn render(&self, style: SessionNameStyle) -> String {
        match self {
            ResolvedName::Identity(id) => SessionName { identity: *id, style }.to_string(),
            ResolvedName::Literal(s) => s.clone(),
        }
    }

    /// The style-independent word the picker fuzzy-search ranks on
    /// (`"rime"`, `"tide"`, or the literal string).
    #[must_use]
    pub fn word(&self) -> &str {
        match self {
            ResolvedName::Identity(id) => id.word,
            ResolvedName::Literal(s) => s,
        }
    }
}

/// The curated atlas. Elemental / celestial / natural words in the Vellum
/// register (warm, calm, a little wild) — each paired with an emoji and a
/// clean single-width Nerd-Font/Unicode glyph alternative.
pub struct FleetSessionNames;

impl FleetSessionNames {
    /// The pool. Order is stable — `pick` indexes into it deterministically.
    pub const POOL: &'static [SessionIdentity] = &[
        SessionIdentity { emoji: "🌊", glyph: "\u{2248}", word: "tide", theme: SessionTheme::Frost, keywords: &["wave","water","sea","ocean","surf"] },
        SessionIdentity { emoji: "❄",  glyph: "\u{2744}", word: "frost", theme: SessionTheme::Frost, keywords: &["snow","ice","cold","winter"] },
        SessionIdentity { emoji: "🔥", glyph: "\u{2588}", word: "ember", theme: SessionTheme::Frost, keywords: &["fire","flame","heat","coal"] },
        SessionIdentity { emoji: "🌲", glyph: "\u{25b2}", word: "cedar", theme: SessionTheme::Frost, keywords: &["tree","pine","forest","wood"] },
        SessionIdentity { emoji: "🔋", glyph: "\u{26a1}", word: "volt", theme: SessionTheme::Frost, keywords: &["power","energy","charge","battery"] },
        SessionIdentity { emoji: "🌙", glyph: "\u{263d}", word: "luna", theme: SessionTheme::Frost, keywords: &["moon","night","crescent"] },
        SessionIdentity { emoji: "🪨", glyph: "\u{25c6}", word: "stone", theme: SessionTheme::Frost, keywords: &["rock","boulder","granite"] },
        SessionIdentity { emoji: "🍃", glyph: "\u{2767}", word: "fern", theme: SessionTheme::Frost, keywords: &["leaf","plant","green","frond"] },
        SessionIdentity { emoji: "☄",  glyph: "\u{2736}", word: "comet", theme: SessionTheme::Frost, keywords: &["star","meteor","tail","space"] },
        SessionIdentity { emoji: "🌋", glyph: "\u{25b3}", word: "magma", theme: SessionTheme::Frost, keywords: &["volcano","lava","eruption"] },
        SessionIdentity { emoji: "🐚", glyph: "\u{273f}", word: "shell", theme: SessionTheme::Frost, keywords: &["sea","conch","beach","spiral"] },
        SessionIdentity { emoji: "🦊", glyph: "\u{25d5}", word: "fox", theme: SessionTheme::Frost, keywords: &["animal","clever","kit","vixen"] },
        SessionIdentity { emoji: "🦉", glyph: "\u{25c9}", word: "owl", theme: SessionTheme::Frost, keywords: &["bird","night","wise","hoot"] },
        SessionIdentity { emoji: "🧭", glyph: "\u{2316}", word: "compass", theme: SessionTheme::Frost, keywords: &["direction","navigate","north","find"] },
        SessionIdentity { emoji: "🌾", glyph: "\u{2058}", word: "wheat", theme: SessionTheme::Frost, keywords: &["grain","field","harvest","gold"] },
        SessionIdentity { emoji: "🪵", glyph: "\u{2261}", word: "timber", theme: SessionTheme::Frost, keywords: &["wood","log","lumber","plank"] },
        SessionIdentity { emoji: "🌒", glyph: "\u{25d1}", word: "dusk", theme: SessionTheme::Frost, keywords: &["evening","twilight","moon"] },
        SessionIdentity { emoji: "🌅", glyph: "\u{25d0}", word: "dawn", theme: SessionTheme::Frost, keywords: &["sunrise","morning","daybreak"] },
        SessionIdentity { emoji: "🏔", glyph: "\u{25b4}", word: "ridge", theme: SessionTheme::Frost, keywords: &["mountain","peak","summit","alp"] },
        SessionIdentity { emoji: "🌀", glyph: "\u{29bf}", word: "vortex", theme: SessionTheme::Frost, keywords: &["spiral","cyclone","swirl","whirl"] },
        SessionIdentity { emoji: "🪶", glyph: "\u{2710}", word: "quill", theme: SessionTheme::Frost, keywords: &["feather","pen","write","plume"] },
        SessionIdentity { emoji: "🔭", glyph: "\u{2295}", word: "scope", theme: SessionTheme::Frost, keywords: &["telescope","look","see","far"] },
        SessionIdentity { emoji: "⚓", glyph: "\u{2693}", word: "anchor", theme: SessionTheme::Frost, keywords: &["ship","sea","harbor","hold"] },
        SessionIdentity { emoji: "🕯", glyph: "\u{2020}", word: "taper", theme: SessionTheme::Frost, keywords: &["candle","flame","light","wax"] },
        SessionIdentity { emoji: "🌿", glyph: "\u{273d}", word: "sage", theme: SessionTheme::Frost, keywords: &["herb","plant","wise","green"] },
        SessionIdentity { emoji: "🪐", glyph: "\u{2641}", word: "orbit", theme: SessionTheme::Frost, keywords: &["planet","saturn","ring","space"] },
        SessionIdentity { emoji: "🦌", glyph: "\u{2638}", word: "stag", theme: SessionTheme::Frost, keywords: &["deer","antler","buck","animal"] },
        SessionIdentity { emoji: "🐋", glyph: "\u{223f}", word: "whale", theme: SessionTheme::Frost, keywords: &["sea","ocean","blue","big"] },
        SessionIdentity { emoji: "🜂", glyph: "\u{25b3}", word: "spark", theme: SessionTheme::Frost, keywords: &["fire","flame","ignite","flash"] },
        SessionIdentity { emoji: "🌑", glyph: "\u{25cf}", word: "void", theme: SessionTheme::Frost, keywords: &["dark","black","empty","newmoon"] },
        SessionIdentity { emoji: "🪷", glyph: "\u{2740}", word: "lotus", theme: SessionTheme::Frost, keywords: &["flower","bloom","pink","water"] },
        SessionIdentity { emoji: "🦅", glyph: "\u{25b7}", word: "hawk", theme: SessionTheme::Frost, keywords: &["bird","eagle","sky","hunt"] },
        // ── Brazil register — warm tropical (a nod to the operator) ──────
        SessionIdentity { emoji: "🌴", glyph: "\u{2663}", word: "palmeira", theme: SessionTheme::Brazil, keywords: &["palm","tree","tropical","beach"] },
        SessionIdentity { emoji: "☀",  glyph: "\u{263c}", word: "sol", theme: SessionTheme::Brazil, keywords: &["sun","sunshine","bright","day"] },
        SessionIdentity { emoji: "🥁", glyph: "\u{266b}", word: "samba", theme: SessionTheme::Brazil, keywords: &["drum","beat","rhythm","dance"] },
        SessionIdentity { emoji: "🦜", glyph: "\u{2748}", word: "arara", theme: SessionTheme::Brazil, keywords: &["macaw","parrot","bird","color"] },
        SessionIdentity { emoji: "🥥", glyph: "\u{25cb}", word: "coco", theme: SessionTheme::Brazil, keywords: &["coconut","tropical","palm"] },
        SessionIdentity { emoji: "☕", glyph: "\u{2615}", word: "cafe", theme: SessionTheme::Brazil, keywords: &["coffee","brew","bean","cafezinho"] },
        SessionIdentity { emoji: "⚽", glyph: "\u{2688}", word: "futebol", theme: SessionTheme::Brazil, keywords: &["soccer","ball","football","goal"] },
        SessionIdentity { emoji: "🏖", glyph: "\u{2602}", word: "praia", theme: SessionTheme::Brazil, keywords: &["beach","sand","shore","coast"] },
        SessionIdentity { emoji: "🌳", glyph: "\u{2660}", word: "floresta", theme: SessionTheme::Brazil, keywords: &["forest","jungle","amazon","tree"] },
        SessionIdentity { emoji: "🐆", glyph: "\u{25c8}", word: "onca", theme: SessionTheme::Brazil, keywords: &["jaguar","leopard","cat","spots"] },
        SessionIdentity { emoji: "🥭", glyph: "\u{25d4}", word: "manga", theme: SessionTheme::Brazil, keywords: &["mango","fruit","tropical","sweet"] },
        SessionIdentity { emoji: "🎸", glyph: "\u{266a}", word: "bossa", theme: SessionTheme::Brazil, keywords: &["guitar","music","violao","strings"] },
    ];

    /// The branded fleet-default identity — **`🌑 rime`**, our own
    /// "void × frost" blend (blackmatter's dark register × the Frost
    /// register; `rime` = hoarfrost). A session with no project to name it
    /// from (mado's boot launch with no spawn cwd) gets THIS instead of an
    /// opaque `mado-<ts>-<pid>` tag — compact, on-theme, and rendered per
    /// surface like any identity. Deliberately NOT a [`Self::POOL`] member:
    /// it is the special home identity, never a random project/seed draw.
    /// The single-width glyph (`◉`) keeps the cursor-sensitive text prompt
    /// grid-aligned; the wide emoji (`🌑`) is for the GUI picker.
    pub const DEFAULT: SessionIdentity = SessionIdentity {
        emoji: "🌑",
        glyph: "\u{25c9}", // ◉ fisheye — dark centre + ring: void×frost, one column
        word: "rime",
        theme: SessionTheme::Frost,
        keywords: &["frost", "ice", "dark", "void", "hoarfrost", "cold", "rime"],
    };

    /// THE single naming authority: map a [`NameContext`] to a
    /// [`ResolvedName`]. Every session-name decision in the fleet flows
    /// through here, so a session is named ONCE and rendered per surface —
    /// the picker and the prompt project the SAME identity, never two
    /// independently-derived names.
    #[must_use]
    pub fn resolve(ctx: NameContext<'_>) -> ResolvedName {
        match ctx {
            NameContext::Override(s) => ResolvedName::Literal(s.to_owned()),
            NameContext::Project(p) => ResolvedName::Identity(Self::identity_for_path(p)),
            NameContext::Default => ResolvedName::Identity(Self::DEFAULT),
        }
    }

    /// The branded fleet-default [`SessionName`] in a style — `🌑 rime`
    /// (Emoji) / `◉ rime` (Glyph). Convenience over `resolve(Default)`.
    #[must_use]
    pub fn default_name(style: SessionNameStyle) -> SessionName {
        SessionName { identity: Self::DEFAULT, style }
    }

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

    /// Deterministic identity drawn from a single THEME — the basis for
    /// "a new ad-hoc session gets a random themed name" (`Frost` or
    /// `Brazil`). Same `(seed, theme)` → same identity; project-bound
    /// sessions instead stay path-stable via [`Self::from_project_path`].
    /// Falls back to the whole-pool [`Self::identity`] if the theme is
    /// somehow empty (it never is).
    #[must_use]
    pub fn in_theme(seed: u64, theme: SessionTheme) -> SessionIdentity {
        let count = Self::POOL.iter().filter(|i| i.theme == theme).count();
        if count == 0 {
            return Self::identity(seed);
        }
        let idx = (seed as usize) % count;
        *Self::POOL
            .iter()
            .filter(|i| i.theme == theme)
            .nth(idx)
            .expect("idx < count by construction")
    }

    /// A themed-random resolved [`SessionName`] for a seed + style.
    #[must_use]
    pub fn name_in_theme(seed: u64, theme: SessionTheme, style: SessionNameStyle) -> SessionName {
        SessionName { identity: Self::in_theme(seed, theme), style }
    }

    /// Every identity whose word or keywords match `query` — the search
    /// surface behind "type `wave`, see `🌊 tide`".
    #[must_use]
    pub fn matching(query: &str) -> impl Iterator<Item = &'static SessionIdentity> {
        let q = query.trim().to_lowercase();
        Self::POOL.iter().filter(move |i| q.is_empty() || i.matches(&q))
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
    fn fleet_default_is_the_void_frost_blend() {
        assert_eq!(FleetSessionNames::DEFAULT.word, "rime");
        assert_eq!(FleetSessionNames::DEFAULT.theme, SessionTheme::Frost);
        assert_eq!(
            FleetSessionNames::default_name(SessionNameStyle::Emoji).to_string(),
            "🌑 rime"
        );
        assert_eq!(
            FleetSessionNames::default_name(SessionNameStyle::Glyph).to_string(),
            "◉ rime"
        );
        // The special home identity — NOT a random-assignment POOL member,
        // so a project/seed can never accidentally collide onto it.
        assert!(
            !FleetSessionNames::POOL.iter().any(|i| i.word == "rime"),
            "DEFAULT must stay out of the POOL so it is never a random draw"
        );
    }

    #[test]
    fn resolve_is_the_single_authority_with_split_projection() {
        // Default → the branded identity, renderable in BOTH styles from
        // ONE resolved value (the basis for emoji-in-GUI / glyph-in-prompt).
        let d = FleetSessionNames::resolve(NameContext::Default);
        assert_eq!(d.render(SessionNameStyle::Emoji), "🌑 rime");
        assert_eq!(d.render(SessionNameStyle::Glyph), "◉ rime");
        assert_eq!(d.word(), "rime", "same word across both renders");

        // Project → the path-stable identity, byte-identical to
        // from_project_path (resolve is the one decider, not a new scheme).
        let p = std::path::Path::new("/code/pleme-io/mado");
        let r = FleetSessionNames::resolve(NameContext::Project(p));
        let direct =
            FleetSessionNames::from_project_path(p, SessionNameStyle::Emoji).to_string();
        assert_eq!(r.render(SessionNameStyle::Emoji), direct);

        // Override → verbatim, style-agnostic (no curated mark either way).
        let o = FleetSessionNames::resolve(NameContext::Override("billing-stack"));
        assert_eq!(o.render(SessionNameStyle::Emoji), "billing-stack");
        assert_eq!(o.render(SessionNameStyle::Glyph), "billing-stack");
        assert_eq!(o.word(), "billing-stack");
    }

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
            identity: SessionIdentity {
                emoji: "🌊",
                glyph: "≈",
                word: "tide",
                theme: SessionTheme::Frost,
                keywords: &["wave"],
            },
            style: SessionNameStyle::Emoji,
        };
        assert_eq!(n.to_string(), "🌊 tide");
        let g = SessionName { style: SessionNameStyle::Glyph, ..n };
        assert_eq!(g.to_string(), "≈ tide");
    }

    #[test]
    fn keyword_search_finds_the_emoji() {
        // The operator's example: type "wave", find 🌊 tide.
        let tide = FleetSessionNames::POOL.iter().find(|i| i.word == "tide").unwrap();
        assert!(tide.matches("wave"), "'wave' should match 🌊 tide via keyword");
        assert!(tide.matches("tide"), "the word itself always matches");
        assert!(tide.matches("WAV"), "matching is case-insensitive + substring");
        assert!(!tide.matches("xyzzy"));
        // `matching` surfaces it from the whole pool.
        assert!(FleetSessionNames::matching("wave").any(|i| i.word == "tide"));
    }

    #[test]
    fn themes_partition_the_pool_and_in_theme_stays_in_register() {
        let frost = FleetSessionNames::POOL.iter().filter(|i| i.theme == SessionTheme::Frost).count();
        let brazil = FleetSessionNames::POOL.iter().filter(|i| i.theme == SessionTheme::Brazil).count();
        assert_eq!(frost + brazil, FleetSessionNames::POOL.len(), "every identity has a theme");
        assert!(brazil >= 8, "brazil register populated");
        // in_theme(seed, Brazil) always returns a Brazil identity, deterministically.
        for seed in 0..50u64 {
            let id = FleetSessionNames::in_theme(seed, SessionTheme::Brazil);
            assert_eq!(id.theme, SessionTheme::Brazil);
            assert_eq!(id, FleetSessionNames::in_theme(seed, SessionTheme::Brazil));
        }
    }

    #[test]
    fn every_identity_has_keywords() {
        for id in FleetSessionNames::POOL {
            assert!(!id.keywords.is_empty(), "{} has no search keywords", id.word);
        }
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
