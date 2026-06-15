//! # ishou-emoji
//!
//! The comprehensive, typed, named, searchable catalog of **every** Unicode
//! emoji — generated at build time from the official Unicode
//! [`emoji-test.txt`](https://www.unicode.org/Public/emoji/latest/emoji-test.txt)
//! (vendored under `data/`), parsed into a `&'static [Emoji]` by `build.rs`.
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
//! No emoji is hand-typed. `build.rs` owns the data (vendored emoji-test.txt →
//! `OUT_DIR` codegen → `include!`), so a Unicode update is a one-line re-fetch
//! of `data/emoji-test.txt` followed by a rebuild — no source edits.
//!
//! ## Examples
//!
//! ```
//! use ishou_emoji::{by_shortcode, by_name, search, Group};
//!
//! assert_eq!(by_shortcode("rocket").unwrap().ch, "🚀");
//! assert_eq!(by_shortcode(":rocket:").unwrap().ch, "🚀"); // colons are stripped
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
    /// Shortcode aliases (no surrounding colons). Always includes a slug of the
    /// full name; for skin-tone / qualified variants it also includes a slug of
    /// the tone-less base name so `by_shortcode("waving_hand")` resolves.
    pub shortcodes: &'static [&'static str],
    /// Lowercase keyword tokens derived from the name (stopwords removed).
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

/// Normalize a shortcode query: strip surrounding `:` and lowercase, mapping
/// spaces and `-` to `_` so `"red heart"` / `"red-heart"` both match.
fn normalize_shortcode(q: &str) -> String {
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
/// colon-wrapped form (`":rocket:"`); spaces and `-` are treated as `_`.
///
/// Returns the first catalog entry that lists the (normalized) shortcode.
#[must_use]
pub fn by_shortcode(shortcode: &str) -> Option<&'static Emoji> {
    let needle = normalize_shortcode(shortcode);
    if needle.is_empty() {
        return None;
    }
    CATALOG.iter().find(|e| e.shortcodes.iter().any(|s| *s == needle))
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
    fn shortcode_no_collision_on_distinct_chars() {
        // Policy: by_shortcode returns the FIRST (Unicode-order) entry for a
        // shortcode. Verify the common curated codes resolve to the expected
        // canonical character (i.e. the first-wins policy is sane).
        // Shortcodes here are slugs of the canonical Unicode name (this crate's
        // source carries no GitHub/CLDR shortcodes — see crate docs), so e.g.
        // ✅ is "check_mark_button", not GitHub's "white_check_mark".
        let cases = [
            ("fire", "🔥"),
            ("thumbs_up", "👍"),
            ("eyes", "👀"),
            ("warning", "⚠️"),
            ("check_mark_button", "✅"),
        ];
        for (code, ch) in cases {
            assert_eq!(
                by_shortcode(code).map(|e| e.ch),
                Some(ch),
                "shortcode {code:?} should resolve to {ch}"
            );
        }
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
