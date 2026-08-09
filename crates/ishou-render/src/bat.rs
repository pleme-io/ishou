//! bat renderer — emits a base16 Sublime-Text `.tmTheme` (XML plist) that
//! `bat` (the `cat` clone) consumes as a custom theme.
//!
//! This is the FIRST per-app *config-file* renderer in ishou (every prior
//! target emits a palette, a token dump, or a scheme YAML). It is the M0
//! proof that ishou can natively emit a per-app config file, toward ishou
//! replacing stylix as the fleet theming engine: instead of stylix
//! generating the `bat` theme from its base16 scheme, ishou renders the
//! `.tmTheme` directly from the same typed `TokenSet` the base16 scheme
//! comes from.
//!
//! Output format: a Sublime-Text `.tmTheme` property list (the format
//! `programs.bat.themes.<name>.src` / bat's `~/.config/bat/themes/*.tmTheme`
//! consume). bat parses it with `syntect`; the format is a `<plist>` whose
//! `settings` array holds one global-settings dict followed by one dict per
//! scope-selector rule.
//!
//! ## base16 → `.tmTheme` mapping
//!
//! The mapping follows the canonical base16 `.tmTheme` template used by
//! `tinted-theming`/`base16-textmate` (the same slot roles the base16
//! `bat`/`base16-stylix` theme uses), so this render displaces that
//! generated theme byte-role-for-byte:
//!
//! | tmTheme setting        | base16 slot | ishou colour     | role |
//! |------------------------|-------------|------------------|------|
//! | `background`           | base00      | `polar_night_0`  | default background |
//! | `foreground`/`caret`   | base05      | `snow_storm_1`   | default foreground |
//! | `invisibles`           | base03      | `polar_night_3`  | comments |
//! | `lineHighlight`        | base01      | `polar_night_1`  | line highlight |
//! | `selection`            | base02      | `polar_night_2`  | selection bg |
//! | `gutterForeground`     | base03      | `polar_night_3`  | gutter fg |
//! | Comment                | base03      | `polar_night_3`  | |
//! | Variables / deleted    | base08      | `aurora_red`     | |
//! | Integers / constants   | base09      | `aurora_orange`  | |
//! | Classes / bold         | base0A      | `aurora_yellow`  | |
//! | Strings / inserted     | base0B      | `aurora_green`   | |
//! | Escapes / regex        | base0C      | `frost_1`        | |
//! | Functions / headings   | base0D      | `frost_2`        | |
//! | Keywords / italic      | base0E      | `aurora_purple`  | |
//! | Embedded / deprecated  | base0F      | `frost_3`        | |
//!
//! Hex values are `#RRGGBB` (upper-case-insensitive; we emit lowercase),
//! **with** a leading `#` — the tmTheme plist requires the `#` prefix
//! (unlike stylix base16 YAML, which forbids it).

use ishou_tokens::{Rgb, TokenSet};

/// Render the base16 `.tmTheme` bat consumes.
///
/// Pure function — same `TokenSet` always produces byte-identical output.
/// Determinism is a test invariant: no timestamps, no map iteration; the
/// scope rules are emitted in a fixed, hand-ordered sequence.
#[must_use]
pub fn render(t: &TokenSet) -> String {
    let c = &t.color;

    // The 16 base16 slots, mapped onto Nord exactly as `stylix::render`
    // maps them (single source of the slot→colour contract in this crate).
    let base00 = c.polar_night_0;
    let base01 = c.polar_night_1;
    let base02 = c.polar_night_2;
    let base03 = c.polar_night_3;
    let base05 = c.snow_storm_1;
    let base08 = c.aurora_red;
    let base09 = c.aurora_orange;
    let base0a = c.aurora_yellow;
    let base0b = c.aurora_green;
    let base0c = c.frost_1;
    let base0d = c.frost_2;
    let base0e = c.aurora_purple;
    let base0f = c.frost_3;

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
    );
    out.push_str("<plist version=\"1.0\">\n");
    out.push_str("<dict>\n");
    out.push_str("\t<key>name</key>\n");
    out.push_str("\t<string>Nord (pleme-io / ishou)</string>\n");
    out.push_str("\t<key>author</key>\n");
    out.push_str("\t<string>Arctic Ice Studio; mapped by pleme-io ishou</string>\n");
    out.push_str("\t<key>semanticClass</key>\n");
    out.push_str("\t<string>theme.dark.nord_pleme_io_ishou</string>\n");
    out.push_str("\t<key>colorSpaceName</key>\n");
    out.push_str("\t<string>sRGB</string>\n");
    out.push_str("\t<key>settings</key>\n");
    out.push_str("\t<array>\n");

    // Global settings dict (no `scope` / `name`).
    out.push_str("\t\t<dict>\n");
    out.push_str("\t\t\t<key>settings</key>\n");
    out.push_str("\t\t\t<dict>\n");
    push_setting(&mut out, "background", base00);
    push_setting(&mut out, "caret", base05);
    push_setting(&mut out, "foreground", base05);
    push_setting(&mut out, "invisibles", base03);
    push_setting(&mut out, "lineHighlight", base01);
    push_setting(&mut out, "selection", base02);
    push_setting(&mut out, "gutterForeground", base03);
    out.push_str("\t\t\t</dict>\n");
    out.push_str("\t\t</dict>\n");

    // Per-scope rules, in a fixed order (deterministic — never sorted at
    // runtime; the order below IS the canonical order).
    push_rule(&mut out, "Comment", "comment", base03);
    push_rule(&mut out, "String", "string", base0b);
    push_rule(&mut out, "Number", "constant.numeric", base09);
    push_rule(&mut out, "Built-in constant", "constant.language", base09);
    push_rule(
        &mut out,
        "User-defined constant",
        "constant.character, constant.other",
        base09,
    );
    push_rule(&mut out, "Variable", "variable", base08);
    push_rule(&mut out, "Keyword", "keyword", base0e);
    push_rule(&mut out, "Storage", "storage", base0e);
    push_rule(&mut out, "Storage type", "storage.type", base0d);
    push_rule(&mut out, "Class name", "entity.name.class", base0a);
    push_rule(
        &mut out,
        "Inherited class",
        "entity.other.inherited-class",
        base0c,
    );
    push_rule(&mut out, "Function name", "entity.name.function", base0d);
    push_rule(&mut out, "Function argument", "variable.parameter", base09);
    push_rule(&mut out, "Tag name", "entity.name.tag", base08);
    push_rule(
        &mut out,
        "Tag attribute",
        "entity.other.attribute-name",
        base0a,
    );
    push_rule(&mut out, "Library function", "support.function", base0d);
    push_rule(&mut out, "Library constant", "support.constant", base0c);
    push_rule(
        &mut out,
        "Library class/type",
        "support.type, support.class",
        base0a,
    );
    push_rule(&mut out, "Invalid", "invalid", base08);
    push_rule(&mut out, "Invalid deprecated", "invalid.deprecated", base0f);

    out.push_str("\t</array>\n");
    out.push_str("\t<key>uuid</key>\n");
    out.push_str("\t<string>nord-pleme-io-ishou</string>\n");
    out.push_str("</dict>\n");
    out.push_str("</plist>\n");
    out
}

/// Push one `<key>foreground</key><string>#rrggbb</string>` pair inside a
/// settings dict (indent depth = 4 tabs).
fn push_setting(out: &mut String, key: &str, rgb: Rgb) {
    out.push_str(&format!("\t\t\t\t<key>{key}</key>\n"));
    out.push_str(&format!("\t\t\t\t<string>{}</string>\n", hex(rgb)));
}

/// Push one scope rule dict:
///
/// ```xml
/// <dict>
///   <key>name</key><string>Comment</string>
///   <key>scope</key><string>comment</string>
///   <key>settings</key>
///   <dict><key>foreground</key><string>#rrggbb</string></dict>
/// </dict>
/// ```
fn push_rule(out: &mut String, name: &str, scope: &str, rgb: Rgb) {
    out.push_str("\t\t<dict>\n");
    out.push_str("\t\t\t<key>name</key>\n");
    out.push_str(&format!("\t\t\t<string>{name}</string>\n"));
    out.push_str("\t\t\t<key>scope</key>\n");
    out.push_str(&format!("\t\t\t<string>{scope}</string>\n"));
    out.push_str("\t\t\t<key>settings</key>\n");
    out.push_str("\t\t\t<dict>\n");
    out.push_str("\t\t\t\t<key>foreground</key>\n");
    out.push_str(&format!("\t\t\t\t<string>{}</string>\n", hex(rgb)));
    out.push_str("\t\t\t</dict>\n");
    out.push_str("\t\t</dict>\n");
}

/// `#rrggbb`, lowercase, `#`-prefixed (the tmTheme plist requires the `#`).
fn hex(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ishou_tokens::TokenSet;

    #[test]
    fn output_is_deterministic() {
        let t = TokenSet::pleme();
        assert_eq!(render(&t), render(&t));
    }

    #[test]
    fn output_is_non_empty() {
        assert!(!render(&TokenSet::pleme()).is_empty());
    }

    #[test]
    fn output_is_a_tmtheme_plist() {
        let out = render(&TokenSet::pleme());
        assert!(out.starts_with("<?xml version=\"1.0\""), "got:\n{out}");
        assert!(out.contains("<plist version=\"1.0\">"), "got:\n{out}");
        assert!(out.contains("<key>settings</key>"), "got:\n{out}");
        assert!(out.trim_end().ends_with("</plist>"), "got:\n{out}");
    }

    #[test]
    fn background_is_base00_polar_night_0() {
        let out = render(&TokenSet::pleme());
        // background=base00=polar_night_0=#2e3440 — with `#` prefix (unlike
        // the stylix YAML, the tmTheme plist requires it).
        assert!(
            out.contains("<key>background</key>\n\t\t\t\t<string>#2e3440</string>"),
            "background must be base00 (#2e3440); got:\n{out}"
        );
    }

    #[test]
    fn foreground_is_base05_snow_storm_1() {
        let out = render(&TokenSet::pleme());
        assert!(
            out.contains("<key>foreground</key>\n\t\t\t\t<string>#e5e9f0</string>"),
            "foreground must be base05 (#e5e9f0); got:\n{out}"
        );
    }

    #[test]
    fn strings_are_base0b_aurora_green() {
        let out = render(&TokenSet::pleme());
        // The String scope rule maps to base0B (aurora_green, #a3be8c).
        assert!(out.contains("<string>#a3be8c</string>"), "got:\n{out}");
    }

    #[test]
    fn hex_is_hash_prefixed_lowercase() {
        let out = render(&TokenSet::pleme());
        // Every colour value in the plist must be `#` + six lowercase hex
        // digits; a regression (missing `#`, upper-case) silently breaks
        // syntect's colour parse.
        for line in out.lines() {
            let trimmed = line.trim();
            if let Some(v) = trimmed
                .strip_prefix("<string>#")
                .and_then(|r| r.strip_suffix("</string>"))
            {
                assert_eq!(v.len(), 6, "wrong hex length in: {line}");
                assert!(
                    v.chars()
                        .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase()),
                    "expected lowercase hex in: {line}"
                );
            }
        }
    }
}
