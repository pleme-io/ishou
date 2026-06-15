//! # ishou-emoji
//!
//! The comprehensive, typed, named, searchable catalog of **every** Unicode
//! emoji — generated at build time by joining two vendored datasets in
//! `build.rs`:
//!
//! - the official Unicode
//!   [`emoji-test.txt`](https://www.unicode.org/Public/emoji/latest/emoji-test.txt)
//!   (the full RGI set + canonical names), and
//! - GitHub's official
//!   [`gemoji`](https://github.com/github/gemoji) `emoji.json` (the
//!   GitHub/Slack shortcodes + keyword tags, MIT).
//!
//! Both parse into a `&'static [Emoji]`. **Shortcodes are GitHub/gemoji-primary
//! with a Unicode-name-derived fallback:** where gemoji assigns codes (~1,870 of
//! the ~3,950 entries), its `aliases` lead (`:white_check_mark:`, `:tada:`,
//! `:+1:`, `:joy:`) and its `tags` enrich the keywords, while the Unicode-name
//! slug (`:check_mark_button:`) is retained as an additional fallback so both
//! styles resolve. The remaining ~2,080 entries keep only their Unicode-name
//! slugs.
//!
//! This is the **foundation layer** beneath the curated
//! [`ishou_tokens::FleetSignals`] vocabulary: `FleetSignals` (and per-app sets)
//! curate the "best" subset; this crate is the *entire* keyboard. Apps pull any
//! emoji by name / shortcode / keyword from here.
//!
//! ## Why a separate crate
//!
//! The catalog is on the order of a megabyte of baked-in `&'static` data. It
//! lives in its own crate so it never bloats `ishou-tokens`, which is consumed
//! fleet-wide and must stay light. Depend on `ishou-emoji` only where you need
//! the full set.
//!
//! ## Generation, not composition
//!
//! No emoji or shortcode is hand-typed. `build.rs` owns the data (the vendored
//! `emoji-test.txt` + `gemoji.json`, joined → `OUT_DIR` codegen → `include!`),
//! so an update is a one-line re-fetch of either `data/` file followed by a
//! rebuild — no source edits.
//!
//! ## Examples
//!
//! ```
//! use ishou_emoji::{by_shortcode, by_name, search, Group};
//!
//! assert_eq!(by_shortcode("rocket").unwrap().ch, "🚀");
//! assert_eq!(by_shortcode(":rocket:").unwrap().ch, "🚀"); // colons are stripped
//! // GitHub/Slack shortcodes (from gemoji) resolve:
//! assert_eq!(by_shortcode("white_check_mark").unwrap().ch, "✅");
//! assert_eq!(by_shortcode("tada").unwrap().ch, "🎉");
//! assert_eq!(by_shortcode("+1").unwrap().ch, "👍");
//! // …and the Unicode-name slug still works as a fallback:
//! assert_eq!(by_shortcode("check_mark_button").unwrap().ch, "✅");
//!
//! let heart = by_name("red heart").unwrap();
//! assert_eq!(heart.ch, "❤️");
//!
//! // Substring search over name + shortcodes + keywords, ranked.
//! let hearts = search("heart");
//! assert!(hearts.iter().any(|e| e.ch == "❤️"));
//!
//! // Iterate one Unicode top-level group.
//! let flags: Vec<_> = ishou_emoji::by_group(Group::Flags).collect();
//! assert!(!flags.is_empty());
//! ```

#![forbid(unsafe_code)]

/// A single Unicode emoji entry.
///
/// Every field is `&'static` — the whole catalog is baked into the binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Emoji {
    /// The emoji character (one or more Unicode scalar values, incl. ZWJ
    /// sequences, variation selectors, and skin-tone modifiers).
    pub ch: &'static str,
    /// The canonical Unicode name, e.g. `"grinning face"` or
    /// `"waving hand: light skin tone"`.
    pub name: &'static str,
    /// The Unicode top-level group this emoji belongs to.
    pub group: Group,
    /// The Unicode subgroup label, e.g. `"face-smiling"`.
    pub subgroup: &'static str,
    /// Shortcode aliases (no surrounding colons), in resolution-priority order:
    /// the GitHub/gemoji `aliases` first (the codes people type, e.g.
    /// `"white_check_mark"`, `"tada"`, `"+1"`), then a slug of the full Unicode
    /// name (`"check_mark_button"`), then — for skin-tone / qualified variants —
    /// a slug of the tone-less base name so `by_shortcode("waving_hand")`
    /// resolves. Entries with no gemoji match carry only the name slugs.
    pub shortcodes: &'static [&'static str],
    /// Lowercase keyword tokens for search: the GitHub/gemoji `tags` first (the
    /// human-curated terms, e.g. `"hooray"`, `"party"` for 🎉), then the
    /// name-derived tokens (stopwords removed).
    pub keywords: &'static [&'static str],
    /// The Unicode/emoji version this entry was introduced in, e.g. `"1.0"`,
    /// `"15.1"`. Empty string if the source omitted it.
    pub unicode_version: &'static str,
    /// `true` if this is a skin-tone variant (name contains `"skin tone"`).
    pub has_skin_tone: bool,
    /// The tone-less base name; equals [`Emoji::name`] for non-tone entries.
    pub base_name: &'static str,
}

/// The Unicode top-level emoji groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    /// Smileys & Emotion.
    SmileysEmotion,
    /// People & Body.
    PeopleBody,
    /// Animals & Nature.
    AnimalsNature,
    /// Food & Drink.
    FoodDrink,
    /// Travel & Places.
    TravelPlaces,
    /// Activities.
    Activities,
    /// Objects.
    Objects,
    /// Symbols.
    Symbols,
    /// Flags.
    Flags,
    /// Component (skin tones, hair styles — building blocks, not standalone
    /// emoji in the usual sense).
    Component,
}

impl Group {
    /// All groups, in Unicode order.
    pub const ALL: [Group; 10] = [
        Group::SmileysEmotion,
        Group::PeopleBody,
        Group::AnimalsNature,
        Group::FoodDrink,
        Group::TravelPlaces,
        Group::Activities,
        Group::Objects,
        Group::Symbols,
        Group::Flags,
        Group::Component,
    ];

    /// The canonical Unicode label for this group, e.g. `"Smileys & Emotion"`.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Group::SmileysEmotion => "Smileys & Emotion",
            Group::PeopleBody => "People & Body",
            Group::AnimalsNature => "Animals & Nature",
            Group::FoodDrink => "Food & Drink",
            Group::TravelPlaces => "Travel & Places",
            Group::Activities => "Activities",
            Group::Objects => "Objects",
            Group::Symbols => "Symbols",
            Group::Flags => "Flags",
            Group::Component => "Component",
        }
    }
}

// The generated `pub static CATALOG: &[Emoji]`.
include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

/// The number of emoji in the catalog.
#[must_use]
pub fn len() -> usize {
    CATALOG.len()
}

/// `true` if the catalog is empty (it never is — present for API completeness).
#[must_use]
pub fn is_empty() -> bool {
    CATALOG.is_empty()
}

/// Iterate over every emoji in the catalog, in Unicode order.
pub fn iter() -> impl Iterator<Item = &'static Emoji> {
    CATALOG.iter()
}

/// Iterate over every emoji in a given [`Group`].
pub fn by_group(group: Group) -> impl Iterator<Item = &'static Emoji> {
    CATALOG.iter().filter(move |e| e.group == group)
}

/// Strip surrounding `:` and lowercase a shortcode query, *without* rewriting
/// separators. This preserves the gemoji literal codes that are not slugs —
/// notably `+1` / `-1` (👍 / 👎), where mapping `-` → `_` would otherwise
/// destroy the token.
fn normalize_shortcode_literal(q: &str) -> String {
    let trimmed = q.trim().trim_matches(':');
    let mut out = String::with_capacity(trimmed.len());
    for c in trimmed.chars() {
        out.extend(c.to_lowercase());
    }
    out
}

/// As [`normalize_shortcode_literal`] but also maps spaces and `-` to `_`, so
/// `"red heart"` / `"red-heart"` both resolve to the `red_heart` slug.
fn normalize_shortcode_slug(q: &str) -> String {
    let trimmed = q.trim().trim_matches(':');
    let mut out = String::with_capacity(trimmed.len());
    for c in trimmed.chars() {
        if c == ' ' || c == '-' {
            out.push('_');
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

/// Look up an emoji by shortcode. Accepts the bare form (`"rocket"`) or the
/// colon-wrapped form (`":rocket:"`); spaces and `-` are treated as `_` so the
/// Unicode-name slugs resolve, while the gemoji literal codes `+1` / `-1`
/// (which are not slugs) are matched verbatim.
///
/// Shortcodes lead with the GitHub/gemoji aliases (`white_check_mark`, `tada`,
/// `+1`), then fall back to the Unicode-name slug (`check_mark_button`), so both
/// styles resolve. Returns the first catalog entry that lists the shortcode.
#[must_use]
pub fn by_shortcode(shortcode: &str) -> Option<&'static Emoji> {
    // Try the literal form first so `+1` / `-1` match the gemoji aliases
    // verbatim; then the separator-normalized slug form for `red-heart` etc.
    let literal = normalize_shortcode_literal(shortcode);
    if !literal.is_empty()
        && let Some(e) = CATALOG
            .iter()
            .find(|e| e.shortcodes.iter().any(|s| *s == literal))
    {
        return Some(e);
    }
    let slug = normalize_shortcode_slug(shortcode);
    if slug.is_empty() || slug == literal {
        return None;
    }
    CATALOG
        .iter()
        .find(|e| e.shortcodes.iter().any(|s| *s == slug))
}

/// Look up an emoji by its exact canonical Unicode name (case-insensitive),
/// e.g. `"grinning face"` or `"red heart"`.
#[must_use]
pub fn by_name(name: &str) -> Option<&'static Emoji> {
    let needle = name.trim().to_lowercase();
    if needle.is_empty() {
        return None;
    }
    CATALOG.iter().find(|e| e.name.to_lowercase() == needle)
}

/// Look up an emoji by its character, e.g. `"🚀"`.
#[must_use]
pub fn by_char(ch: &str) -> Option<&'static Emoji> {
    CATALOG.iter().find(|e| e.ch == ch)
}

/// Search rank — higher is a better match. Used to order [`search`] results.
fn rank(e: &Emoji, q: &str) -> Option<u32> {
    let name = e.name.to_lowercase();
    if name == q {
        return Some(100);
    }
    if e.shortcodes.contains(&q) {
        return Some(90);
    }
    if name.starts_with(q) || e.shortcodes.iter().any(|s| s.starts_with(q)) {
        return Some(70);
    }
    if name.contains(q) {
        return Some(50);
    }
    if e.shortcodes.iter().any(|s| s.contains(q)) {
        return Some(40);
    }
    if e.keywords.contains(&q) {
        return Some(35);
    }
    if e.keywords.iter().any(|k| k.contains(q)) {
        return Some(20);
    }
    None
}

/// Case-insensitive substring search over name + shortcodes + keywords.
///
/// Results are ranked: exact-name > exact-shortcode > prefix > name-contains >
/// shortcode-contains > keyword. Ties preserve Unicode catalog order. Returns
/// an empty `Vec` for an empty query.
#[must_use]
pub fn search(query: &str) -> Vec<&'static Emoji> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(u32, usize, &'static Emoji)> = CATALOG
        .iter()
        .enumerate()
        .filter_map(|(i, e)| rank(e, &q).map(|r| (r, i, e)))
        .collect();
    // Sort by descending rank, then ascending catalog index (stable order).
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, _, e)| e).collect()
}

/// Return every skin-tone variant whose base name matches `base` (the tone-less
/// name, e.g. `"waving hand"`), case-insensitive. Empty if the emoji has no
/// tone variants.
#[must_use]
pub fn skin_tones(base: &str) -> Vec<&'static Emoji> {
    let needle = base.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    CATALOG
        .iter()
        .filter(|e| e.has_skin_tone && e.base_name.to_lowercase() == needle)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_non_empty_and_full_scale() {
        // The full Unicode RGI set is well over 3,000 entries.
        assert!(len() >= 3000, "catalog only has {} emoji", len());
        assert!(!is_empty());
        assert_eq!(len(), iter().count());
    }

    #[test]
    fn rocket_shortcode_resolves() {
        assert_eq!(by_shortcode("rocket").unwrap().ch, "🚀");
        // Colon-wrapped form is accepted.
        assert_eq!(by_shortcode(":rocket:").unwrap().ch, "🚀");
        // Leading/trailing whitespace tolerated.
        assert_eq!(by_shortcode("  rocket  ").unwrap().ch, "🚀");
    }

    #[test]
    fn by_name_round_trips() {
        let e = by_name("grinning face").expect("grinning face present");
        assert_eq!(e.ch, "😀");
        // Round-trip: name -> emoji -> name.
        assert_eq!(by_name(e.name).unwrap().ch, e.ch);
        // by_char round-trips too.
        assert_eq!(by_char(e.ch).unwrap().name, "grinning face");
    }

    #[test]
    fn by_name_is_case_insensitive() {
        assert_eq!(by_name("GRINNING FACE").unwrap().ch, "😀");
        assert_eq!(by_name("Red Heart").unwrap().ch, "❤️");
    }

    #[test]
    fn search_heart_returns_heart_family() {
        let results = search("heart");
        assert!(!results.is_empty());
        // The red heart should be in there.
        assert!(results.iter().any(|e| e.ch == "❤️"));
        // Several heart-family members present.
        assert!(
            results.iter().filter(|e| e.name.contains("heart")).count() >= 5,
            "expected a heart family, got {} matches",
            results.len()
        );
    }

    #[test]
    fn search_ranks_exact_name_first() {
        // Searching the exact name puts that emoji at the front.
        let results = search("rocket");
        assert_eq!(results.first().unwrap().ch, "🚀");
    }

    #[test]
    fn search_empty_query_is_empty() {
        assert!(search("").is_empty());
        assert!(search("   ").is_empty());
        assert!(by_shortcode("").is_none());
        assert!(by_name("").is_none());
    }

    #[test]
    fn every_group_is_represented() {
        for g in Group::ALL {
            assert!(
                by_group(g).next().is_some(),
                "group {g:?} ({}) has no emoji",
                g.label()
            );
        }
        // The non-component groups should be substantial.
        assert!(by_group(Group::SmileysEmotion).count() > 100);
        assert!(by_group(Group::Flags).count() > 100);
    }

    #[test]
    fn group_count_partitions_catalog() {
        let sum: usize = Group::ALL.iter().map(|g| by_group(*g).count()).sum();
        assert_eq!(sum, len(), "group partition must cover the whole catalog");
    }

    #[test]
    fn skin_tone_variants_are_exposed() {
        let tones = skin_tones("waving hand");
        // Five Fitzpatrick tones.
        assert_eq!(tones.len(), 5, "expected 5 skin tones, got {}", tones.len());
        assert!(tones.iter().all(|e| e.has_skin_tone));
        assert!(tones.iter().all(|e| e.base_name == "waving hand"));
    }

    #[test]
    fn base_emoji_has_no_skin_tone_flag() {
        let e = by_name("waving hand").expect("base waving hand present");
        assert!(!e.has_skin_tone);
        assert_eq!(e.base_name, "waving hand");
    }

    #[test]
    fn unicode_name_slugs_still_resolve() {
        // Policy: by_shortcode returns the FIRST (Unicode-order) entry for a
        // shortcode. The Unicode-name slugs are retained as fallback shortcodes
        // even where gemoji also assigns a GitHub code, so the original
        // name-derived codes keep working (no regression).
        let cases = [
            ("fire", "🔥"),
            ("thumbs_up", "👍"),
            ("eyes", "👀"),
            ("warning", "⚠️"),
            // ✅'s Unicode-name slug — resolves alongside GitHub's
            // "white_check_mark" (asserted in `github_shortcodes_resolve`).
            ("check_mark_button", "✅"),
        ];
        for (code, ch) in cases {
            assert_eq!(
                by_shortcode(code).map(|e| e.ch),
                Some(ch),
                "name-slug shortcode {code:?} should resolve to {ch}"
            );
        }
    }

    #[test]
    fn github_shortcodes_resolve() {
        // The canonical GitHub/Slack shortcodes — sourced from gemoji, NOT
        // derivable from Unicode names — must now resolve.
        let cases = [
            ("white_check_mark", "✅"),
            ("tada", "🎉"),
            ("joy", "😂"),
            ("fire", "🔥"),
            ("rocket", "🚀"),
            ("eyes", "👀"),
            ("100", "💯"),
            ("pray", "🙏"),
        ];
        for (code, ch) in cases {
            assert_eq!(
                by_shortcode(code).map(|e| e.ch),
                Some(ch),
                "GitHub shortcode {code:?} should resolve to {ch}"
            );
            // Colon-wrapped form too.
            assert_eq!(by_shortcode(&format!(":{code}:")).map(|e| e.ch), Some(ch));
        }
    }

    #[test]
    fn plus_one_minus_one_special_cases_resolve() {
        // gemoji uses the literal `+1` / `-1` codes for 👍 / 👎. These are not
        // slugs — the `-` must NOT be rewritten to `_`.
        assert_eq!(by_shortcode("+1").map(|e| e.ch), Some("👍"));
        assert_eq!(by_shortcode(":+1:").map(|e| e.ch), Some("👍"));
        assert_eq!(by_shortcode("-1").map(|e| e.ch), Some("👎"));
        assert_eq!(by_shortcode(":-1:").map(|e| e.ch), Some("👎"));
    }

    #[test]
    fn gemoji_keywords_enrich_search() {
        // "hooray" is a gemoji tag for 🎉 (tada) — not present in its Unicode
        // name "party popper". Searching it must surface 🎉.
        let results = search("hooray");
        assert!(
            results.iter().any(|e| e.ch == "🎉"),
            "gemoji tag 'hooray' should surface 🎉 in search"
        );
        // "joy" is a gemoji tag on several smileys; the exact alias 😂 ranks.
        let joy = search("joy");
        assert!(joy.iter().any(|e| e.ch == "😂"));
    }

    #[test]
    fn gemoji_coverage_split_is_reported() {
        // ~1,870 of the ~3,950 catalog entries get GitHub/gemoji shortcodes;
        // the rest keep Unicode-name slugs only. Assert the partition is whole
        // and the gemoji-covered count is in the expected ballpark.
        assert_eq!(
            GEMOJI_COVERED + UNICODE_ONLY,
            len(),
            "coverage split must partition the catalog"
        );
        // The two bounds are over generated `const`s, so they evaluate at
        // compile time — a regression in the join (e.g. zero matches) fails the
        // build, not just the test run.
        const {
            assert!(
                GEMOJI_COVERED > 1500,
                "expected >1500 gemoji-covered entries"
            );
            assert!(
                UNICODE_ONLY > 1000,
                "expected a substantial Unicode-only remainder"
            );
        }
        // Sanity: gemoji-covered entries are a large-but-minority slice.
        assert!(GEMOJI_COVERED < len());
    }

    #[test]
    fn all_entries_have_a_nonempty_char_and_name() {
        for e in iter() {
            assert!(!e.ch.is_empty(), "empty ch for {:?}", e.name);
            assert!(!e.name.is_empty(), "empty name for {}", e.ch);
            assert!(!e.shortcodes.is_empty(), "no shortcodes for {}", e.name);
        }
    }
}
