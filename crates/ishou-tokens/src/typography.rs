//! Typography tokens — font stacks + modular scale.
//!
//! Typed font primitive for the pleme-io fleet. The `mono_primary` and
//! `mono_italic` slots are load-bearing for every GPU app (mado,
//! hibikine, kagibako, …) — they must point at families whose glyph
//! coverage matches the operator's shell config (starship symbols,
//! atuin markers, nerd-font powerline). The fallback chains are stable
//! in order from most-specific → most-system-portable.
//!
//! Pair the families slot with `MonoFonts::nerd_font_package_attr()`
//! (renderer-side) to materialise the Nix package paths the HM modules
//! ship.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Typography {
    pub families: FontFamilies,
    pub mono_fonts: MonoFonts,
    pub scale: TypeScale,
    pub weight: FontWeights,
    pub line_height: LineHeights,
    pub features: FontFeatures,
}

#[derive(Debug, Clone, Serialize)]
pub struct FontFamilies {
    pub serif: &'static str,
    pub sans: &'static str,
    pub mono: &'static str,
    pub display: &'static str,
}

/// Typed mono-font surface for GPU consumers. Carries the four
/// orthogonal axes: primary regular, primary italic (whose visual
/// style is configurable — calligraphic, cursive, or matching the
/// primary), bold variant, and the fallback chain glyphon walks when
/// the primary family doesn't carry a glyph.
///
/// `nerd_font_package_attr` names the canonical `pkgs.nerd-fonts.*`
/// attribute name HM modules install when the operator wants the
/// patched font shipped declaratively. Renderers wire this into the
/// HM module's `home.packages` and stylix's `fonts.monospace.package`.
#[derive(Debug, Clone, Serialize)]
pub struct MonoFonts {
    /// Primary monospace family — the operator-facing display name
    /// glyphon resolves at runtime. Defaults to JetBrainsMono Nerd
    /// Font; every glyph starship / atuin / git symbols / nerd-symbol
    /// powerline assume is present.
    pub primary: &'static str,
    /// Italic variant. Operators can point this at a calligraphic
    /// alternative (Maple Mono Italic, Operator Mono Lig italic,
    /// Iosevka Etoile, Cascadia Code Italic) for highlighted italics.
    pub italic: &'static str,
    /// Bold variant. Same family by default — fonts that ship a
    /// dedicated bold face (Berkeley Mono, Iosevka Heavy) override.
    pub bold: &'static str,
    /// Style intent for the italic slot. The renderer can use this to
    /// surface the "matches primary" vs "calligraphic" choice in
    /// editors / mado-side bold/italic toggles without re-parsing the
    /// family string.
    pub italic_style: ItalicStyle,
    /// Ordered fallback chain glyphon walks when the primary doesn't
    /// have a glyph for a codepoint. Order: nerd-symbols first (so
    /// powerline glyphs hit before unrelated emoji), then platform
    /// fallback (Apple Color Emoji / Noto Color Emoji), then
    /// system-mono catch-all.
    pub fallback: &'static [&'static str],
    /// nixpkgs `nerd-fonts.<attr>` name for the HM module install.
    /// `None` when the primary is shipped via the OS or a separate
    /// upstream package.
    pub nerd_font_package_attr: Option<&'static str>,
    /// nixpkgs attribute for the italic-variant package, when not the
    /// same as `nerd_font_package_attr`. Used when calligraphic
    /// italics ship from a different upstream (e.g. Iosevka Etoile,
    /// Maple Mono NF).
    pub italic_package_attr: Option<&'static str>,
}

/// Operator-visible classification of the italic slot. Renderers can
/// pick a different upstream family per intent without the operator
/// authoring the family string by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ItalicStyle {
    /// Italic glyphs from the same family as `primary` — the safe
    /// monospace default. Matches the regular face's vertical metrics.
    MatchesPrimary,
    /// Calligraphic / hand-drawn italic (e.g. Iosevka Etoile, Maple
    /// Mono Italic, Operator Mono Italic). The pleme-io fleet default
    /// — gives code highlights / comments / language-keyword italics
    /// an authored, brand-aligned feel without leaving mono.
    Calligraphic,
    /// Cursive italic — connected glyphs, less common in mono. Off by
    /// default; opt-in for editors that want this stylistic flourish.
    Cursive,
}

/// OpenType / shaping features mado enables by default. Each field is
/// a tri-state: `Some(true)` = force-on, `Some(false)` = force-off,
/// `None` = font-default. Consumers may render this to the
/// font-engine's feature-tag syntax (HarfBuzz `+calt`, etc.).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FontFeatures {
    /// Contextual alternates — required for ligature fonts (Fira Code,
    /// JetBrains Mono ligatures, Cascadia Code).
    pub calt: Option<bool>,
    /// Standard ligatures.
    pub liga: Option<bool>,
    /// Discretionary ligatures — usually opt-in.
    pub dlig: Option<bool>,
    /// Tabular numerals — keeps digit columns aligned in tables / TUI
    /// dashboards. On by default since mado is terminal-first.
    pub tnum: Option<bool>,
    /// Zero-with-slash variant (alternate stylistic set typical to
    /// programming fonts). On by default; the slash disambiguates O / 0.
    pub zero_slashed: Option<bool>,
}

/// 1.25 modular scale from 0.75rem base-ish, in em-style sizes.
#[derive(Debug, Clone, Serialize)]
pub struct TypeScale {
    pub xs: f32,
    pub sm: f32,
    pub base: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub x2: f32,
    pub x3: f32,
    pub x4: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FontWeights {
    pub light: u16,
    pub regular: u16,
    pub medium: u16,
    pub semibold: u16,
    pub bold: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct LineHeights {
    pub tight: f32,
    pub base: f32,
    pub relaxed: f32,
    pub prose: f32,
}

impl Typography {
    #[must_use]
    pub const fn pleme() -> Self {
        Self {
            families: FontFamilies {
                serif: "'Iowan Old Style', 'Charter', 'Georgia', 'Cambria', Times, serif",
                sans: "'Inter', system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif",
                // Mono stack — Nerd-Font-patched primary, plain JetBrains
                // Mono fallback, then platform-mono catch-alls. Web
                // consumers (lilitu, zuihitsu) read this stack into
                // `font-family`; GPU consumers (mado, hibikine, kagibako)
                // resolve via `mono_fonts.primary` directly.
                mono: "'JetBrainsMono Nerd Font Mono', 'JetBrains Mono', 'SF Mono', 'Menlo', 'Consolas', monospace",
                // Display = used for brand titles / hero headings; swerve-forward.
                display: "'Inter', system-ui, sans-serif",
            },
            mono_fonts: MonoFonts::pleme(),
            scale: TypeScale {
                xs: 0.75,
                sm: 0.875,
                base: 1.0,
                md: 1.125,
                lg: 1.25,
                xl: 1.5,
                x2: 1.875,
                x3: 2.25,
                x4: 3.0,
            },
            weight: FontWeights {
                light: 300,
                regular: 400,
                medium: 500,
                semibold: 600,
                bold: 700,
            },
            line_height: LineHeights {
                tight: 1.15,
                base: 1.5,
                relaxed: 1.65,
                prose: 1.7,
            },
            features: FontFeatures::pleme(),
        }
    }
}

impl MonoFonts {
    /// The canonical pleme-io mono font surface: JetBrainsMono Nerd
    /// Font **Mono** as the primary — the `Mono`-suffixed Nerd Fonts
    /// family forces every glyph (icons included) into a single-cell
    /// advance equal to plain `JetBrains Mono`. The non-`Mono` variant
    /// widens ASCII advance to ~0.83em to accommodate double-cell
    /// icons; that's correct for editor/web monospace but produces
    /// the 2026-05-13 wide-gap rendering bug in mado/ghostty because
    /// terminal cell_width measurement (advance of "MM") then
    /// disagrees with the natural advance of an ASCII glyph in any
    /// downstream layout. For every consumer where a single cell ==
    /// one glyph is the hard invariant (terminals, by definition),
    /// the `Mono` variant is correct. Iosevka stays as the
    /// calligraphic italic (no `Mono` distinction in Iosevka — every
    /// face is already strict-monospace). Symbols Nerd Font **Mono**
    /// + FiraCode Nerd Font **Mono** make up the fallback chain for
    /// glyphs the primary doesn't carry, both forced to single-cell.
    #[must_use]
    pub const fn pleme() -> Self {
        Self {
            primary: "JetBrainsMono Nerd Font Mono",
            italic: "Iosevka",
            bold: "JetBrainsMono Nerd Font Mono",
            italic_style: ItalicStyle::Calligraphic,
            fallback: &[
                "Symbols Nerd Font Mono",
                "FiraCode Nerd Font Mono",
                "JetBrains Mono",
                "SF Mono",
                "Menlo",
                "Apple Color Emoji",
                "monospace",
            ],
            // The Nerd-Font-patched JetBrains Mono is shipped by
            // nixpkgs as `pkgs.nerd-fonts.jetbrains-mono`. Both the
            // `JetBrainsMono Nerd Font` (variable-advance) AND
            // `JetBrainsMono Nerd Font Mono` (strict-monospace) family
            // names resolve from the same package — no install delta.
            nerd_font_package_attr: Some("jetbrains-mono"),
            // Iosevka ships the regular + italic faces from
            // `pkgs.iosevka`; the calligraphic-style "Etoile" subfamily
            // is selected via Family::Name("Iosevka") + Style::Italic
            // in the cosmic-text fontdb.
            italic_package_attr: Some("iosevka"),
        }
    }
}

impl FontFeatures {
    /// The pleme-io defaults: contextual alternates + standard
    /// ligatures + tabular numerals + slashed zero on; discretionary
    /// ligatures left at font-default. Mado is terminal-first, so
    /// digit alignment matters; ligatures earn their keep on prompt
    /// arrows and operator glyphs.
    #[must_use]
    pub const fn pleme() -> Self {
        Self {
            calt: Some(true),
            liga: Some(true),
            dlig: None,
            tnum: Some(true),
            zero_slashed: Some(true),
        }
    }
}
