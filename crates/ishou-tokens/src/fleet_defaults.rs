//! Fleet defaults — the canonical cross-app config baseline.
//!
//! # Why
//!
//! Every operator-facing pleme-io app (mado, escriba, tear, frost,
//! frostmourne, ayatsuri, namimado) makes the same set of choices:
//! what font, what size, what theme, what cursor, what behavior
//! toggles. Without a shared source these answers drift — mado's
//! "default font" diverges from frost's, both diverge from
//! escriba's, the fleet stops feeling like one product.
//!
//! `FleetDefaults` is the **single typed source** every fleet app's
//! `shikumi::TieredConfig::prescribed_default()` reads from. Apps
//! that pick a font, picks `FleetDefaults::prescribed().font_family`.
//! Apps that pick a cursor, pick `FleetDefaults::prescribed().cursor`.
//! Changing the fleet font size = one line touch here = every app
//! converges next launch.
//!
//! # The bare/prescribed contract (mirrors FleetTheme)
//!
//! Every field has a `bare` value (zero-opinion floor) AND a
//! `prescribed` value (the pleme-io developer-prescribed default).
//! Apps that want bare semantics use `FleetDefaults::bare()`; the
//! 90% case that wants the prescribed fleet look uses
//! `FleetDefaults::prescribed()`.
//!
//! # Migration template for fleet apps
//!
//! ```rust,ignore
//! use ishou_tokens::FleetDefaults;
//!
//! impl shikumi::TieredConfig for MyAppConfig {
//!     fn bare() -> Self {
//!         let fb = FleetDefaults::bare();
//!         Self {
//!             font_family: fb.font_family.into(),
//!             font_size: fb.font_size,
//!             // ...
//!         }
//!     }
//!     fn prescribed_default() -> Self {
//!         let fp = FleetDefaults::prescribed();
//!         Self {
//!             font_family: fp.font_family.into(),
//!             font_size: fp.font_size,
//!             theme: fp.theme.resolve().name,
//!             // ...
//!         }
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::fleet_theme::FleetTheme;

/// The cross-app baseline every fleet app references.
///
/// **Two constructors**:
///
/// * `FleetDefaults::bare()` — zero-opinion floor: empty strings,
///   smallest reasonable numerics, no opinions. Apps with bare
///   semantics inherit this.
/// * `FleetDefaults::prescribed()` — const value carrying the
///   developer-prescribed defaults: JetBrainsMono Nerd Font Mono,
///   PlemeDark theme, sane cursor + behavior toggles. Touching
///   this struct propagates to every fleet app on next launch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetDefaults {
    // ── Typography ───────────────────────────────────────────────
    pub theme: FleetTheme,
    pub font_family: String,
    pub font_italic: String,
    pub font_size: f32,
    pub line_height: f32,

    // ── Window / chrome ──────────────────────────────────────────
    pub padding: u32,
    pub decorations_macos: bool,
    pub decorations_linux: bool,

    // ── Cursor ───────────────────────────────────────────────────
    /// `"block"` | `"block_hollow"` | `"bar"` | `"underline"`.
    pub cursor_style: String,
    pub cursor_blink: bool,
    pub cursor_blink_rate_ms: u32,

    // ── Terminal behavior ────────────────────────────────────────
    /// Detect + linkify URLs in the rendered grid.
    pub link_url_detect: bool,
    /// Forward mouse events to TUI apps that report them.
    pub mouse_reporting: bool,
    /// Hide the OS cursor while the user is actively typing.
    pub mouse_hide_while_typing: bool,
    /// Pre-cap (lines). 0 = unlimited; default 10k = ~5MB at 80×24.
    pub scrollback_lines: usize,

    // ── Performance ──────────────────────────────────────────────
    pub vsync: bool,

    // ── Keybinding modifier convention ───────────────────────────
    /// Operator-facing primary modifier. macOS = "cmd", elsewhere = "ctrl".
    /// Apps consume this to render hotkey strings in their docs.
    pub primary_modifier: String,

    // ── Accessibility ────────────────────────────────────────────
    pub reduce_motion: bool,
    pub font_scale: f32,
}

impl FleetDefaults {
    /// **Tier 0 — bare**: zero-opinion floor every field. Apps with
    /// bare semantics inherit these. Matches the `bare()` contract
    /// of `shikumi::TieredConfig`.
    #[must_use]
    pub fn bare() -> Self {
        Self {
            // Typography.
            theme: FleetTheme::Bare,
            font_family: String::new(),
            font_italic: String::new(),
            font_size: 12.0,
            line_height: 1.0,
            // Window.
            padding: 0,
            decorations_macos: false,
            decorations_linux: false,
            // Cursor.
            cursor_style: "block".into(),
            cursor_blink: false,
            cursor_blink_rate_ms: 0,
            // Behavior.
            link_url_detect: false,
            mouse_reporting: false,
            mouse_hide_while_typing: false,
            scrollback_lines: 0,
            // Performance.
            vsync: false,
            // Keybinding.
            primary_modifier: String::new(),
            // Accessibility.
            reduce_motion: false,
            font_scale: 1.0,
        }
    }

    /// **Tier 2 — prescribed**: the canonical pleme-io look + feel.
    /// Every fleet app's `prescribed_default()` references this so
    /// visual + behavioral consistency is enforced by construction.
    ///
    /// Touching this function propagates fleet-wide on next launch.
    #[must_use]
    pub fn prescribed() -> Self {
        Self {
            // Typography — pleme-io fleet canonical. The theme is the
            // FleetTheme prescribed default (Vellum) by reference,
            // so a future prescribed-theme change propagates here on the
            // next compile rather than being re-asserted.
            theme: FleetTheme::prescribed_default(),
            // ── Font alignment to ghostty's known-good look ──────────
            // The operator's ground-truth baseline is ghostty rendering
            // JetBrains Mono at size 13 with +25% cell height. These
            // four values reproduce that rhythm fleet-wide:
            //
            // * `font_family` — the NON-Mono Nerd variant. Text
            //   advances are identical to the Mono variant (M / E0B0 /
            //   F300 all measure 60/100em on both faces), so this does
            //   NOT regress monospacing; the only difference is that the
            //   non-Mono face lets WIDE Nerd icons keep their designed
            //   double-width proportions instead of squishing them to a
            //   single cell. Closer to ghostty's inline-glyph look.
            // * `font_italic` — set EQUAL to `font_family`. ghostty has
            //   no italic-family override; it synthesizes the slant from
            //   the same JetBrainsMono face. Pointing italics at a
            //   different typeface (the old Iosevka) is exactly the
            //   "inconsistent italics" divergence the operator sees.
            // * `font_size` — 13.0, exact match to ghostty's config.
            // * `line_height` — 1.65 = JetBrainsMono's native line
            //   height (1.32, measured live) × ghostty's adjust-cell-
            //   height(+25%). Reproduces ghostty's 21.45px cell at 13pt
            //   instead of the cramped 1.4×font tradition.
            font_family: "JetBrainsMono Nerd Font".into(),
            font_italic: "JetBrainsMono Nerd Font".into(),
            font_size: 13.0,
            line_height: 1.65,
            // Window — opinionated but minimal.
            padding: 0,
            decorations_macos: true,  // keep traffic-lights on mac
            decorations_linux: false, // tiling-WM friendly elsewhere
            // Cursor — visible + blinking but not jarring.
            cursor_style: "block".into(),
            cursor_blink: true,
            cursor_blink_rate_ms: 500,
            // Behavior — terminal-friendly fleet defaults.
            link_url_detect: true,
            mouse_reporting: true,
            mouse_hide_while_typing: true,
            scrollback_lines: 10_000,
            // Performance.
            vsync: true,
            // Keybinding — platform-conventional primary.
            primary_modifier: if cfg!(target_os = "macos") {
                "cmd".into()
            } else {
                "ctrl".into()
            },
            // Accessibility — defer to OS prefs at runtime.
            reduce_motion: false,
            font_scale: 1.0,
        }
    }
}

impl Default for FleetDefaults {
    fn default() -> Self {
        Self::prescribed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_has_zero_opinion_strings_and_numerics() {
        let b = FleetDefaults::bare();
        assert_eq!(b.theme, FleetTheme::Bare);
        assert_eq!(b.font_family, "");
        assert_eq!(b.font_italic, "");
        assert_eq!(b.font_size, 12.0);
        assert_eq!(b.padding, 0);
        assert!(!b.cursor_blink);
        assert!(!b.link_url_detect);
        assert!(!b.mouse_reporting);
        assert_eq!(b.scrollback_lines, 0);
        assert!(!b.vsync);
    }

    #[test]
    fn prescribed_carries_pleme_canonical_choices() {
        let p = FleetDefaults::prescribed();
        // Vellum is the prescribed fleet theme.
        assert_eq!(p.theme, FleetTheme::Vellum);
        assert_eq!(p.theme, FleetTheme::prescribed_default());
        // Font alignment to ghostty's known-good look: NON-Mono Nerd
        // family (icons keep designed width), italics on the SAME
        // family (synthesized slant, not a foreign typeface), size 13
        // (ghostty parity), 1.65 line-height (native 1.32 × +25% cell).
        assert_eq!(p.font_family, "JetBrainsMono Nerd Font");
        assert_eq!(p.font_italic, "JetBrainsMono Nerd Font");
        assert_eq!(p.font_size, 13.0);
        assert_eq!(p.line_height, 1.65);
        assert!(p.cursor_blink);
        assert!(p.link_url_detect);
        assert!(p.mouse_reporting);
        assert_eq!(p.scrollback_lines, 10_000);
        assert!(p.vsync);
    }

    #[test]
    fn default_returns_prescribed_not_bare() {
        // App authors who write `FleetDefaults::default()` get the
        // PRESCRIBED tier — most fleet code paths want this; bare
        // is an explicit opt-in.
        assert_eq!(FleetDefaults::default(), FleetDefaults::prescribed());
    }

    #[test]
    fn fleet_defaults_round_trip_via_serde() {
        let p = FleetDefaults::prescribed();
        let s = serde_yaml::to_string(&p).unwrap();
        let back: FleetDefaults = serde_yaml::from_str(&s).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn primary_modifier_is_platform_conventional() {
        #[cfg(target_os = "macos")]
        assert_eq!(FleetDefaults::prescribed().primary_modifier, "cmd");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(FleetDefaults::prescribed().primary_modifier, "ctrl");
    }
}
