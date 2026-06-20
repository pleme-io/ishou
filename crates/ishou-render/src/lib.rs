//! ishou-render — target-specific renderers from `ishou_tokens::TokenSet`.
//!
//! Every renderer is a pure function: `TokenSet → String`. Determinism is a
//! test invariant — same tokens produce byte-identical output.
//!
//! Morphism table (identical in spirit to arch-synthesizer's morphism graph):
//!
//! | Source         | Target                                 | Module       |
//! |----------------|----------------------------------------|--------------|
//! | `TokenSet`     | CSS custom properties + utility classes | `css`        |
//! | `TokenSet`     | tailwind.config.js                     | `tailwind`   |
//! | `TokenSet`     | SCSS variables                         | `scss`       |
//! | `TokenSet`     | Rust `pub const` module                | `rust`       |
//! | `TokenSet`     | W3C Design Tokens JSON                 | `json`       |
//! | `TokenSet`     | GLSL `#define` header                  | `glsl`       |
//! | `TokenSet`     | Ghostty config block                   | `ghostty`    |
//! | `TokenSet`     | TUI ratatui / crossterm Color table    | `tui`        |
//! | `TokenSet`     | SVG (brand mark + swerve)              | `svg`        |
//! | `TokenSet`     | stylix base16 YAML                     | `stylix`     |

pub mod css;
pub mod md3;
pub mod fleet_fonts;
pub mod ghostty;
pub mod glsl;
pub mod json;
pub mod nix;
pub mod nix_ast;
pub mod rust;
pub mod scss;
pub mod stylix;
pub mod stylix_fonts;
pub mod svg;
pub mod tailwind;
pub mod tui;
pub mod vellum;

/// Every renderable target ishou-cli understands. The string here matches the
/// CLI flag users type (`ishou render --target css`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Css,
    /// Material Design 3 system colors (`--md-sys-color-*`) mapped from the
    /// TokenSet — the target Material consumers (`pleme-mui`) bind to so they
    /// can consume ishou instead of forking a hard-coded MD3 palette.
    Md3,
    Tailwind,
    Scss,
    Rust,
    Json,
    Glsl,
    Ghostty,
    Tui,
    Svg,
    Stylix,
    Nix,
    StylixFonts,
    /// Fleet-fonts attrset — `{ primary, italic, bold, symbols, emoji,
    /// fallback_chain }` consumed directly by every pleme-io GPU app
    /// HM module (mado, blackmatter-ghostty, fumi, kagi, …) to name
    /// the canonical family AND install the underlying nixpkgs
    /// package via `home.packages`. Sibling of `StylixFonts` — same
    /// typed Typography source, different consumer-side shape.
    FleetFonts,
    /// Vellum base16 stylix scheme YAML (the prescribed fleet theme).
    /// Sourced from the BORN `VellumPalette`, not the Nord `TokenSet`.
    StylixVellum,
    /// Vellum base24 stylix scheme YAML — base16 + the real two-tier
    /// brights (base10–17).
    StylixVellumBase24,
    /// Vellum palette preview SVG — one labelled chip per BORN token.
    SvgVellumPalette,
    /// Vellum skim/fzf `--color=k:v,…` string — the fleet picker theme.
    /// Generated from the BORN `VellumPalette`, byte-equivalent to the
    /// hand-authored `skim-tab::NORD_COLORS`.
    SkimVellum,
    /// Vellum escriba theme `*.lisp` — `(deftheme)` + `(defpalette)` +
    /// `(defhighlight …)` over escriba's `CANONICAL_GROUPS`, sourced from
    /// the BORN `VellumPalette`. Mirrors `escriba/configs/vellum.lisp`.
    EscribaVellum,
}

impl Target {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "css" => Self::Css,
            "md3" | "material" | "md-sys-color" => Self::Md3,
            "tailwind" => Self::Tailwind,
            "scss" => Self::Scss,
            "rust" => Self::Rust,
            "json" => Self::Json,
            "glsl" => Self::Glsl,
            "ghostty" => Self::Ghostty,
            "tui" => Self::Tui,
            "svg" => Self::Svg,
            "stylix" | "stylix-base16" => Self::Stylix,
            "stylix-fonts" | "fonts-nix" => Self::StylixFonts,
            "nix" | "nord-palette-nix" => Self::Nix,
            "fleet-fonts" | "fleet_fonts" => Self::FleetFonts,
            "stylix-vellum" | "stylix-base16-vellum" => Self::StylixVellum,
            "stylix-vellum-base24" | "stylix-base24-vellum" => Self::StylixVellumBase24,
            "svg-vellum-palette" | "vellum-palette" => Self::SvgVellumPalette,
            "skim-vellum" | "skim" => Self::SkimVellum,
            "escriba-vellum" | "escriba" => Self::EscribaVellum,
            _ => return None,
        })
    }

    pub fn render(&self, tokens: &ishou_tokens::TokenSet) -> String {
        match self {
            Self::Css => css::render(tokens),
            Self::Md3 => md3::render(tokens),
            Self::Tailwind => tailwind::render(tokens),
            Self::Scss => scss::render(tokens),
            Self::Rust => rust::render(tokens),
            Self::Json => json::render(tokens),
            Self::Glsl => glsl::render(tokens),
            Self::Ghostty => ghostty::render(tokens),
            Self::Tui => tui::render(tokens),
            Self::Svg => svg::render(tokens),
            Self::Stylix => stylix::render(tokens),
            Self::Nix => nix::render(tokens),
            Self::StylixFonts => stylix_fonts::render(tokens),
            Self::FleetFonts => fleet_fonts::render(tokens),
            // Vellum targets are their own BORN source — the Nord
            // `TokenSet` argument is unused.
            Self::StylixVellum => vellum::render_base16(),
            Self::StylixVellumBase24 => vellum::render_base24(),
            Self::SvgVellumPalette => vellum::render_svg_palette(),
            Self::SkimVellum => vellum::render_skim(),
            Self::EscribaVellum => vellum::render_escriba_lisp(),
        }
    }

    pub fn all() -> [Target; 19] {
        [
            Self::Css,
            Self::Md3,
            Self::Tailwind,
            Self::Scss,
            Self::Rust,
            Self::Json,
            Self::Glsl,
            Self::Ghostty,
            Self::Tui,
            Self::Svg,
            Self::Stylix,
            Self::Nix,
            Self::StylixFonts,
            Self::FleetFonts,
            Self::StylixVellum,
            Self::StylixVellumBase24,
            Self::SvgVellumPalette,
            Self::SkimVellum,
            Self::EscribaVellum,
        ]
    }
}
