//! Build-time code generation for the comprehensive Unicode emoji catalog.
//!
//! This is **generation over composition** (Pillar 12): we never hand-type the
//! ~3,790 emoji. The canonical, machine-readable Unicode `emoji-test.txt`
//! (vendored under `data/`) is parsed here at build time into a typed
//! `&'static [Emoji]` catalog, emitted into `OUT_DIR` and `include!`d by the
//! crate root. The data is OWNED (no runtime dependency) and regenerable on
//! every Unicode release by re-fetching `data/emoji-test.txt`.
//!
//! Emission discipline (TYPED EMISSION): the generated Rust is produced by a
//! small typed writer (`CatalogWriter`) backed by `std::fmt::Write` — every
//! string literal is escaped through `escape_rust_str`, never spliced raw.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// One parsed row of the Unicode emoji catalog.
struct ParsedEmoji {
    ch: String,
    name: String,
    group_variant: &'static str,
    subgroup: String,
    unicode_version: String,
    shortcodes: Vec<String>,
    keywords: Vec<String>,
    has_skin_tone: bool,
    base_name: String,
}

/// Map a Unicode top-level group label to the `Group` enum variant ident.
fn group_variant(label: &str) -> Option<&'static str> {
    match label {
        "Smileys & Emotion" => Some("SmileysEmotion"),
        "People & Body" => Some("PeopleBody"),
        "Animals & Nature" => Some("AnimalsNature"),
        "Food & Drink" => Some("FoodDrink"),
        "Travel & Places" => Some("TravelPlaces"),
        "Activities" => Some("Activities"),
        "Objects" => Some("Objects"),
        "Symbols" => Some("Symbols"),
        "Flags" => Some("Flags"),
        "Component" => Some("Component"),
        _ => None,
    }
}

/// Slugify a name fragment into a shortcode token (lowercase, `_`-joined,
/// alphanumerics only). e.g. "grinning face" -> "`grinning_face`".
fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_us = true; // suppress leading underscores
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_us = false;
        } else if !prev_us {
            out.push('_');
            prev_us = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

/// Split a name into keyword tokens (lowercase alphanumeric words), dropping
/// trivial stopwords so `search` ranking stays meaningful.
fn keywords_from_name(name: &str) -> Vec<String> {
    const STOP: &[&str] = &["a", "an", "the", "of", "with", "and", "in", "on"];
    let mut seen: Vec<String> = Vec::new();
    for raw in name.split(|c: char| !c.is_ascii_alphanumeric()) {
        if raw.is_empty() {
            continue;
        }
        let w = raw.to_ascii_lowercase();
        if STOP.contains(&w.as_str()) {
            continue;
        }
        if !seen.contains(&w) {
            seen.push(w);
        }
    }
    seen
}

/// Escape a string for emission as a Rust string literal between `"`.
fn escape_rust_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Parse the vendored `emoji-test.txt` into typed rows.
///
/// We keep `fully-qualified` and `component` statuses — together these are the
/// canonical RGI emoji set (every displayable emoji incl. every skin-tone /
/// ZWJ-sequence variant). `minimally-qualified` / `unqualified` are dropped:
/// they are duplicate presentation-selector-stripped forms of an entry already
/// captured by its fully-qualified row.
fn parse(text: &str) -> Vec<ParsedEmoji> {
    let mut out = Vec::new();
    let mut cur_group: Option<&'static str> = None;
    let mut cur_subgroup = String::new();

    for line in text.lines() {
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix("# group:") {
            cur_group = group_variant(rest.trim());
            continue;
        }
        if let Some(rest) = line.strip_prefix("# subgroup:") {
            cur_subgroup = rest.trim().to_string();
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Format: `<codepoints> ; <status> # <emoji> E<ver> <name>`
        let Some((_cps, after)) = line.split_once(';') else {
            continue;
        };
        let Some((status, comment)) = after.split_once('#') else {
            continue;
        };
        let status = status.trim();
        if status != "fully-qualified" && status != "component" {
            continue;
        }
        let Some(group) = cur_group else {
            continue;
        };

        // comment = " 😀 E1.0 grinning face"
        let comment = comment.trim_start();
        let mut parts = comment.splitn(2, ' ');
        let Some(ch) = parts.next() else {
            continue;
        };
        let Some(rest) = parts.next() else {
            continue;
        };
        // rest = "E1.0 grinning face"
        let (version, name) = match rest.split_once(' ') {
            Some((v, n)) if v.starts_with('E') => {
                (v.trim_start_matches('E').to_string(), n.trim().to_string())
            }
            _ => (String::new(), rest.trim().to_string()),
        };
        if name.is_empty() {
            continue;
        }

        // Skin-tone detection: the canonical form is `<base>: <tone> skin tone`
        // (optionally with extra qualifiers after a comma). The base name is the
        // portion before the first `:` when the tail mentions a skin tone.
        let has_skin_tone = name.contains("skin tone");
        let base_name = if has_skin_tone {
            name.split_once(':').map_or_else(|| name.clone(), |(b, _)| b.trim().to_string())
        } else {
            name.clone()
        };

        // Shortcodes: full-name slug + base-name slug (deduped). The base-name
        // slug lets `by_shortcode("waving_hand")` resolve the tone-less variant.
        let mut shortcodes: Vec<String> = Vec::new();
        let full_slug = slugify(&name);
        if !full_slug.is_empty() {
            shortcodes.push(full_slug);
        }
        let base_slug = slugify(&base_name);
        if !base_slug.is_empty() && !shortcodes.contains(&base_slug) {
            shortcodes.push(base_slug);
        }

        let keywords = keywords_from_name(&name);

        out.push(ParsedEmoji {
            ch: ch.to_string(),
            name,
            group_variant: group,
            subgroup: cur_subgroup.clone(),
            unicode_version: version,
            shortcodes,
            keywords,
            has_skin_tone,
            base_name,
        });
    }
    out
}

/// Typed writer for the generated catalog source.
struct CatalogWriter {
    buf: String,
}

impl CatalogWriter {
    fn new() -> Self {
        Self { buf: String::with_capacity(1 << 20) }
    }

    fn header(&mut self, count: usize) {
        let _ = writeln!(
            self.buf,
            "// @generated by build.rs from data/emoji-test.txt — DO NOT EDIT.\n\
             // {count} emoji entries.\n"
        );
    }

    /// Emit a `&'static [&'static str]` slice literal of escaped strings.
    fn str_slice(&mut self, items: &[String]) {
        self.buf.push('&');
        self.buf.push('[');
        for (i, it) in items.iter().enumerate() {
            if i > 0 {
                self.buf.push_str(", ");
            }
            self.buf.push('"');
            self.buf.push_str(&escape_rust_str(it));
            self.buf.push('"');
        }
        self.buf.push(']');
    }

    fn entry(&mut self, e: &ParsedEmoji) {
        self.buf.push_str("    Emoji { ch: \"");
        self.buf.push_str(&escape_rust_str(&e.ch));
        self.buf.push_str("\", name: \"");
        self.buf.push_str(&escape_rust_str(&e.name));
        self.buf.push_str("\", group: Group::");
        self.buf.push_str(e.group_variant);
        self.buf.push_str(", subgroup: \"");
        self.buf.push_str(&escape_rust_str(&e.subgroup));
        self.buf.push_str("\", shortcodes: ");
        self.str_slice(&e.shortcodes);
        self.buf.push_str(", keywords: ");
        self.str_slice(&e.keywords);
        self.buf.push_str(", unicode_version: \"");
        self.buf.push_str(&escape_rust_str(&e.unicode_version));
        self.buf.push_str("\", has_skin_tone: ");
        self.buf.push_str(if e.has_skin_tone { "true" } else { "false" });
        self.buf.push_str(", base_name: \"");
        self.buf.push_str(&escape_rust_str(&e.base_name));
        self.buf.push_str("\" },\n");
    }

    fn finish(self) -> String {
        self.buf
    }
}

fn main() {
    let data_path = Path::new("data/emoji-test.txt");
    println!("cargo:rerun-if-changed=data/emoji-test.txt");
    println!("cargo:rerun-if-changed=build.rs");

    let text = fs::read_to_string(data_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", data_path.display()));
    let parsed = parse(&text);
    assert!(
        parsed.len() > 3000,
        "parsed only {} emoji — emoji-test.txt may be malformed",
        parsed.len()
    );

    let mut w = CatalogWriter::new();
    w.header(parsed.len());
    let _ = writeln!(
        w.buf,
        "/// The full Unicode emoji catalog ({} entries), generated from\n\
         /// the official `emoji-test.txt`.\n\
         pub static CATALOG: &[Emoji] = &[",
        parsed.len()
    );
    for e in &parsed {
        w.entry(e);
    }
    w.buf.push_str("];\n");

    let src = w.finish();
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("catalog.rs");
    fs::write(&dest, src).expect("failed to write generated catalog");
}
