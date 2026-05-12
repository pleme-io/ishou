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
pub mod ghostty;
pub mod glsl;
pub mod json;
pub mod nix;
pub mod rust;
pub mod scss;
pub mod stylix;
pub mod stylix_fonts;
pub mod svg;
pub mod tailwind;
pub mod tui;

/// Every renderable target ishou-cli understands. The string here matches the
/// CLI flag users type (`ishou render --target css`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Css,
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
}

impl Target {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "css" => Self::Css,
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
            _ => return None,
        })
    }

    pub fn render(&self, tokens: &ishou_tokens::TokenSet) -> String {
        match self {
            Self::Css => css::render(tokens),
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
        }
    }

    pub fn all() -> [Target; 12] {
        [
            Self::Css,
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
        ]
    }
}
