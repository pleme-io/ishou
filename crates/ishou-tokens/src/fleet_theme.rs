//! Fleet-wide canonical theme — the typed primitive every operator-
//! facing pleme-io app embeds in its config.
//!
//! # Why
//!
//! Mado, escriba, tear, frost, frostmourne, ayatsuri, namimado, and
//! every future pleme-io GPU/TUI app needs a "what does this app
//! look like by default" answer. Each one previously hand-rolled
//! its own theme strings (mado's `#2e3440` background, escriba's
//! Nord-tinted palette, etc.) — duplication that drifts.
//!
//! `FleetTheme` is the one typed knob each app exposes:
//!
//! ```rust,ignore
//! pub struct MadoConfig {
//!     // ...
//!     pub theme: ishou_tokens::FleetTheme,
//! }
//! ```
//!
//! The app calls `theme.resolve()` at render-init time and reads
//! the resolved colors + fonts. One theme switch on the operator
//! side (`theme: bare` vs `theme: pleme_dark`) flips every fleet
//! app the same way.
//!
//! # Tiers
//!
//! * `FleetTheme::Bare` — monochrome black/white, system mono font,
//!   no brand. The deliberate floor per shikumi `TieredConfig::bare`.
//!
//! * `FleetTheme::PlemeDark` — the legacy pleme-io look: Nord
//!   Polar Night background, Snow Storm foreground, Frost accent,
//!   ishou pleme typography. Retained for continuity.
//!
//! * `FleetTheme::Vellum` — the warm aged-paper Nord-matte fleet
//!   theme. The prescribed default per
//!   `TieredConfig::prescribed_default`.
//!
//! Future: `VellumLight`, `VellumHighContrast`, operator-supplied
//! `Custom(ResolvedTheme)` for inline overrides without forking
//! the enum.

use serde::{Deserialize, Serialize};

use crate::color::{ColorPalette, SemanticRoles};
use crate::typography::Typography;
use crate::vellum::VellumPalette;

/// Operator-facing theme selector. Apps embed this as a typed
/// config field; the renderer reads `resolve()` at init time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetTheme {
    /// Zero-opinion: black background, white foreground, no fonts
    /// preference, no brand. The bare-tier floor.
    Bare,
    /// The legacy pleme-io look: Nord Polar Night dark palette
    /// + ishou typography + brand integration. Retained for
    /// continuity; superseded as the prescribed default by
    /// `Vellum`.
    PlemeDark,
    /// **Vellum** — the fleet theme, warm aged-paper Nord-matte.
    /// An aged-parchment ground + muted matte ink above it. **The
    /// prescribed default** every fleet app lands on without operator
    /// intervention.
    #[default]
    Vellum,
}

impl FleetTheme {
    /// Tier 0 — `FleetTheme::Bare`. Named here for symmetry with
    /// `shikumi::TieredConfig::bare`.
    #[must_use]
    pub const fn bare() -> Self {
        Self::Bare
    }

    /// Tier 2 prescribed default — `FleetTheme::Vellum`.
    #[must_use]
    pub const fn prescribed_default() -> Self {
        Self::Vellum
    }

    /// The theme LIBRARY — every variant the fleet ships, in tier order.
    /// This is the registry surface: tooling (`<app> config-show`, theme
    /// pickers, the verification matrix) iterates it mechanically. Adding
    /// a variant to the enum without listing it here trips the
    /// `library_is_complete` forcing-function test.
    #[must_use]
    pub const fn all() -> &'static [FleetTheme] {
        &[Self::Bare, Self::PlemeDark, Self::Vellum]
    }

    /// Stable, serde-matching name for a theme without resolving it.
    /// The match is EXHAUSTIVE on purpose: a new `FleetTheme` variant
    /// fails to compile until it is named here — the compiler is the
    /// forcing function that keeps the registry total.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Bare => "bare",
            Self::PlemeDark => "pleme_dark",
            Self::Vellum => "vellum",
        }
    }

    /// Resolve to concrete color hex + font names. Consumers read
    /// this struct at render-init time and never re-resolve unless
    /// the operator changes `theme` (shikumi hot-reload triggers).
    #[must_use]
    pub fn resolve(&self) -> ResolvedTheme {
        match self {
            Self::Bare => ResolvedTheme::bare(),
            Self::PlemeDark => ResolvedTheme::pleme_dark(),
            Self::Vellum => ResolvedTheme::vellum(),
        }
    }
}

/// The concrete render-ready theme. Hex color strings (consumers
/// parse with their existing hex-to-rgb path); font family names
/// (consumers look up via cosmic-text / system fontdb).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedTheme {
    /// Window background.
    pub background: String,
    /// Default text color.
    pub foreground: String,
    /// Cursor color (empty = use foreground).
    pub cursor: String,
    /// Selection background.
    pub selection_background: String,
    /// 16-color ANSI palette: indices 0–15 in xterm order
    /// (black/red/green/yellow/blue/magenta/cyan/white +
    /// bright variants).
    pub ansi_16: [String; 16],
    /// Primary monospace font family.
    pub font_family: String,
    /// Italic monospace font family (cosmic-text resolves the
    /// matching face when this name differs from `font_family`).
    pub font_italic: String,
    /// Human-readable name of the theme — useful for diagnostics
    /// and for `mado config-show` to label the resolved tier.
    pub name: String,
}

impl ResolvedTheme {
    /// Monochrome floor: black background, white foreground.
    /// System default mono font (empty string = "let cosmic-text
    /// resolve"). No accent colors.
    #[must_use]
    pub fn bare() -> Self {
        let bw = |hex: &str| hex.to_string();
        Self {
            background: bw("#000000"),
            foreground: bw("#FFFFFF"),
            cursor: String::new(), // empty = use foreground
            selection_background: bw("#808080"),
            ansi_16: [
                bw("#000000"), // 0  black
                bw("#AA0000"), // 1  red
                bw("#00AA00"), // 2  green
                bw("#AAAA00"), // 3  yellow
                bw("#0000AA"), // 4  blue
                bw("#AA00AA"), // 5  magenta
                bw("#00AAAA"), // 6  cyan
                bw("#AAAAAA"), // 7  white
                bw("#555555"), // 8  bright black
                bw("#FF5555"), // 9  bright red
                bw("#55FF55"), // 10 bright green
                bw("#FFFF55"), // 11 bright yellow
                bw("#5555FF"), // 12 bright blue
                bw("#FF55FF"), // 13 bright magenta
                bw("#55FFFF"), // 14 bright cyan
                bw("#FFFFFF"), // 15 bright white
            ],
            font_family: String::new(),
            font_italic: String::new(),
            name: "bare".into(),
        }
    }

    /// Canonical pleme-io dark palette: Nord Polar Night background,
    /// Snow Storm foreground, Frost accent. Sourced from
    /// `ColorPalette::pleme()` + `SemanticRoles::pleme_dark()` so
    /// any future palette tweak in ishou-tokens propagates here
    /// automatically.
    #[must_use]
    pub fn pleme_dark() -> Self {
        let palette = ColorPalette::pleme();
        let roles = SemanticRoles::pleme_dark();
        let typography = Typography::pleme();

        // Helper: resolve a role -> hex.
        let role_hex = |role_key: &str| -> String {
            palette
                .get(role_key)
                .map(|rgb| rgb.hex())
                .unwrap_or_else(|| "#000000".into())
        };

        // Build ANSI 16 from Nord, in xterm index order. Nord's
        // recommended terminal palette: aurora_red/orange/yellow/
        // green/purple + frost_1/2/3 for blue/magenta/cyan + snow
        // for whites + polar night for blacks.
        let ansi_16: [String; 16] = [
            palette.polar_night_0.hex(), // 0  black
            palette.aurora_red.hex(),     // 1  red
            palette.aurora_green.hex(),   // 2  green
            palette.aurora_yellow.hex(),  // 3  yellow
            palette.frost_3.hex(),        // 4  blue (deepest frost)
            palette.aurora_purple.hex(),  // 5  magenta
            palette.frost_1.hex(),        // 6  cyan (frost-teal)
            palette.snow_storm_2.hex(),   // 7  white
            palette.polar_night_3.hex(),  // 8  bright black
            palette.aurora_red.hex(),     // 9  bright red (same as 1)
            palette.aurora_green.hex(),   // 10 bright green
            palette.aurora_yellow.hex(),  // 11 bright yellow
            palette.frost_2.hex(),        // 12 bright blue
            palette.aurora_purple.hex(),  // 13 bright magenta
            palette.frost_0.hex(),        // 14 bright cyan
            palette.snow_storm_2.hex(),   // 15 bright white
        ];

        Self {
            background: role_hex(roles.background),
            foreground: role_hex(roles.text),
            cursor: role_hex(roles.primary),
            selection_background: role_hex(roles.surface_elevated),
            ansi_16,
            font_family: typography.mono_fonts.primary.into(),
            font_italic: typography.mono_fonts.italic.into(),
            name: "pleme_dark".into(),
        }
    }

    /// Vellum — the warm aged-paper Nord-matte fleet theme. The
    /// prescribed default. ANSI 16, surfaces, and the cursor come from
    /// `VellumPalette` so this resolved theme can never drift from the
    /// BORN tokens. The cursor is `green_bright` (an inverse pair ≥7.0)
    /// — it lives in NO base16 slot and ships here as a first-class
    /// field + ANSI 10.
    #[must_use]
    pub fn vellum() -> Self {
        let p = VellumPalette::vellum();
        let surfaces = p.surfaces();
        let typography = Typography::pleme();

        // The ONE canonical ANSI-16 mapping fleet-wide.
        let ansi_src = p.ansi_16();
        let ansi_16: [String; 16] = core::array::from_fn(|i| ansi_src[i].hex());

        Self {
            background: surfaces.background.hex(),
            foreground: surfaces.foreground.hex(),
            // green_bright — first-class cursor field (not a slot).
            cursor: surfaces.cursor.hex(),
            // The violet glass — byte-exact blend product.
            selection_background: surfaces.selection_background.hex(),
            ansi_16,
            font_family: typography.mono_fonts.primary.into(),
            font_italic: typography.mono_fonts.italic.into(),
            name: "vellum".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_theme_default_is_vellum() {
        assert_eq!(FleetTheme::default(), FleetTheme::Vellum);
        assert_eq!(FleetTheme::prescribed_default(), FleetTheme::Vellum);
    }

    #[test]
    fn fleet_theme_bare_returns_bare_variant() {
        assert_eq!(FleetTheme::bare(), FleetTheme::Bare);
    }

    #[test]
    fn bare_resolved_is_monochrome() {
        let r = ResolvedTheme::bare();
        assert_eq!(r.background, "#000000");
        assert_eq!(r.foreground, "#FFFFFF");
        assert_eq!(r.cursor, "");
        assert_eq!(r.name, "bare");
        assert_eq!(r.font_family, "");
    }

    #[test]
    fn pleme_dark_resolved_uses_nord_palette() {
        let r = ResolvedTheme::pleme_dark();
        // Nord polar_night_0 == "#2E3440" — the canonical dark
        // background across the pleme-io fleet.
        assert_eq!(r.background.to_uppercase(), "#2E3440");
        assert_eq!(r.name, "pleme_dark");
        // Foreground must be one of the snow-storm whites (Nord
        // doesn't use pure #FFFFFF).
        assert!(r.foreground.to_uppercase() != "#FFFFFF");
        assert!(!r.font_family.is_empty());
    }

    #[test]
    fn fleet_theme_round_trips_through_serde() {
        for &t in &[FleetTheme::Bare, FleetTheme::PlemeDark, FleetTheme::Vellum] {
            let s = serde_yaml::to_string(&t).unwrap();
            let back: FleetTheme = serde_yaml::from_str(&s).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn ansi_16_palette_has_no_empty_strings() {
        // Every tier must populate all 16 ANSI slots — terminal
        // apps using indexed color must not see "" as a color.
        for r in [
            ResolvedTheme::bare(),
            ResolvedTheme::pleme_dark(),
            ResolvedTheme::vellum(),
        ] {
            for (i, c) in r.ansi_16.iter().enumerate() {
                assert!(c.starts_with('#'), "ANSI slot {i} in {} is not hex: {c:?}", r.name);
            }
        }
    }

    #[test]
    fn vellum_resolves_from_born_tokens() {
        let r = ResolvedTheme::vellum();
        assert_eq!(r.name, "vellum");
        // Background night0, foreground snow1, cursor green_bright,
        // selection the byte-exact violet glass.
        assert_eq!(r.background, "#16140E");
        assert_eq!(r.foreground, "#E2DBC8");
        assert_eq!(r.cursor, "#ADD7A3");
        assert_eq!(r.selection_background, "#3A343E");
        // ANSI 15 is snow3 (= base07), never #FFFFFF.
        assert_eq!(r.ansi_16[15], "#F4EFE2");
        // ANSI 2 is the signature green.
        assert_eq!(r.ansi_16[2], "#A9BB8C");
        // ANSI 0 is night2 (surface), NEVER base00.
        assert_eq!(r.ansi_16[0], "#2B2820");
    }
}
