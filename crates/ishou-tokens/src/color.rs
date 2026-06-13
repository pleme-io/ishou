//! Color tokens. The Nord palette comes from `irodori`; ishou adds the
//! brand monochromes (pleme-io's bold-black + paper-white) and the semantic
//! role mapping used across web, TUI, and shader targets.

use irodori::{Color, NORD};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    #[must_use]
    pub fn hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
    #[must_use]
    pub fn from_color(c: Color) -> Self {
        Self::new(c.r, c.g, c.b)
    }
}

/// Every named color ishou recognises. Includes Nord + brand monochrome.
#[derive(Debug, Clone, Serialize)]
pub struct ColorPalette {
    // Nord polar night — dark backgrounds.
    pub polar_night_0: Rgb,
    pub polar_night_1: Rgb,
    pub polar_night_2: Rgb,
    pub polar_night_3: Rgb,
    // Nord snow storm — light foregrounds.
    pub snow_storm_0: Rgb,
    pub snow_storm_1: Rgb,
    pub snow_storm_2: Rgb,
    // Nord frost — cool accents (primary).
    pub frost_0: Rgb,
    pub frost_1: Rgb,
    pub frost_2: Rgb,
    pub frost_3: Rgb,
    // Nord aurora — semantic accents.
    pub aurora_red: Rgb,
    pub aurora_orange: Rgb,
    pub aurora_yellow: Rgb,
    pub aurora_green: Rgb,
    pub aurora_purple: Rgb,
    // pleme-io brand monochrome — used for logo, shadow, swerve stroke.
    pub ink: Rgb,        // bold-black (#0A0A0A) — deeper than polar_night_0
    pub paper: Rgb,      // paper-white (#F5F5F0) — warmer than snow_storm_2
    pub shadow_tone: Rgb, // tinted black used for drop shadows
}

impl ColorPalette {
    /// The canonical pleme-io palette: Nord + brand monochrome.
    #[must_use]
    pub fn pleme() -> Self {
        Self {
            polar_night_0: Rgb::from_color(NORD.polar_night[0]),
            polar_night_1: Rgb::from_color(NORD.polar_night[1]),
            polar_night_2: Rgb::from_color(NORD.polar_night[2]),
            polar_night_3: Rgb::from_color(NORD.polar_night[3]),
            snow_storm_0: Rgb::from_color(NORD.snow_storm[0]),
            snow_storm_1: Rgb::from_color(NORD.snow_storm[1]),
            snow_storm_2: Rgb::from_color(NORD.snow_storm[2]),
            frost_0: Rgb::from_color(NORD.frost[0]),
            frost_1: Rgb::from_color(NORD.frost[1]),
            frost_2: Rgb::from_color(NORD.frost[2]),
            frost_3: Rgb::from_color(NORD.frost[3]),
            aurora_red: Rgb::from_color(NORD.aurora[0]),
            aurora_orange: Rgb::from_color(NORD.aurora[1]),
            aurora_yellow: Rgb::from_color(NORD.aurora[2]),
            aurora_green: Rgb::from_color(NORD.aurora[3]),
            aurora_purple: Rgb::from_color(NORD.aurora[4]),
            ink: Rgb::new(0x0A, 0x0A, 0x0A),
            paper: Rgb::new(0xF5, 0xF5, 0xF0),
            shadow_tone: Rgb::new(0x14, 0x18, 0x22),
        }
    }
}

/// Semantic color roles consumers bind to — keeps tokens stable when a
/// palette value moves.
///
/// Role-keys resolve through `ColorPalette::get` (Nord) for the
/// `pleme_dark` binding and through `BorealisPalette::get` for the
/// `borealis_night` binding. The Borealis binding adds nine new roles
/// (`text_secondary` … `structural`) — spec §7 — that the Nord binding
/// fills with the nearest legacy palette key so every consumer
/// iterating `pairs()` sees a populated value on both themes.
#[derive(Debug, Clone, Serialize)]
pub struct SemanticRoles {
    pub background: &'static str,
    pub surface: &'static str,
    pub surface_elevated: &'static str,
    pub text: &'static str,
    pub text_muted: &'static str,
    pub text_dim: &'static str,
    pub primary: &'static str,
    pub primary_hover: &'static str,
    pub accent: &'static str,
    pub error: &'static str,
    pub warning: &'static str,
    pub success: &'static str,
    pub info: &'static str,
    pub border: &'static str,
    pub shadow: &'static str,
    pub ink: &'static str,
    pub paper: &'static str,
    // ── Borealis additions (spec §7 — the complete struct diff) ──────
    /// Secondary body text (Borealis `snow0`).
    pub text_secondary: &'static str,
    /// Brightest body text (Borealis `snow2`).
    pub text_bright: &'static str,
    /// AGENT-RESERVED accent — vigy / MCP / AI surfaces (Borealis
    /// `fable_violet`). Downstream references THIS role, never the hex.
    pub agent: &'static str,
    /// Agent attention state (Borealis `violet_bright`).
    pub agent_attention: &'static str,
    /// Links / function names (Borealis `ice_steel`).
    pub link: &'static str,
    /// Selection background (Borealis `selection` — the violet glass).
    pub selection: &'static str,
    /// Block cursor (Borealis `green_bright`).
    pub cursor: &'static str,
    /// Search-current background (Borealis `first_light`).
    pub search: &'static str,
    /// Structural-only accent — never body text (Borealis `ice_deep`).
    pub structural: &'static str,
}

impl SemanticRoles {
    /// Dark-first binding used across every pleme-io product.
    /// The nine Borealis-only roles map to the nearest Nord palette key
    /// so a `pleme_dark` consumer iterating `pairs()` never sees an
    /// unresolvable role.
    #[must_use]
    pub const fn pleme_dark() -> Self {
        Self {
            background: "polar_night_0",
            surface: "polar_night_1",
            surface_elevated: "polar_night_2",
            text: "snow_storm_2",
            text_muted: "snow_storm_0",
            text_dim: "polar_night_3",
            primary: "frost_1",
            primary_hover: "frost_2",
            accent: "aurora_purple",
            error: "aurora_red",
            warning: "aurora_orange",
            success: "aurora_green",
            info: "frost_0",
            border: "polar_night_2",
            shadow: "shadow_tone",
            ink: "ink",
            paper: "paper",
            // Borealis additions — nearest Nord key for legacy theme.
            text_secondary: "snow_storm_0",
            text_bright: "snow_storm_2",
            agent: "aurora_purple",
            agent_attention: "aurora_purple",
            link: "frost_2",
            selection: "polar_night_2",
            cursor: "frost_1",
            search: "aurora_yellow",
            structural: "frost_3",
        }
    }

    /// Borealis dark binding — `borealis-night` (spec §7). Every role
    /// resolves through `BorealisPalette::get`. M3 resolution: `info`
    /// is `ice_teal` (one token, one duty). The agent accent
    /// (`fable_violet`) is named via the `agent` role so downstream
    /// references the SEMANTIC, never the hex.
    #[must_use]
    pub const fn borealis_night() -> Self {
        Self {
            background: "night0",
            surface: "night1",
            surface_elevated: "night2",
            text: "snow1",
            text_muted: "shadow1",
            text_dim: "shadow0",
            primary: "ice_cyan",
            primary_hover: "cyan_bright",
            accent: "fable_violet",
            error: "aurora_red",
            warning: "first_light",
            success: "aurora_green",
            info: "ice_teal", // M3 — info IS ice_teal, everywhere.
            border: "night3",
            shadow: "shadow_tone",
            ink: "ink",
            paper: "paper",
            // Borealis-native roles.
            text_secondary: "snow0",
            text_bright: "snow2",
            agent: "fable_violet",
            agent_attention: "violet_bright",
            link: "ice_steel",
            selection: "selection",
            cursor: "green_bright",
            search: "first_light",
            structural: "ice_deep",
        }
    }

    /// Lookup table of every role → palette-key pair, in stable order.
    /// Used by renderers to iterate.
    #[must_use]
    pub fn pairs(&self) -> [(&'static str, &'static str); 26] {
        [
            ("background", self.background),
            ("surface", self.surface),
            ("surface-elevated", self.surface_elevated),
            ("text", self.text),
            ("text-muted", self.text_muted),
            ("text-dim", self.text_dim),
            ("primary", self.primary),
            ("primary-hover", self.primary_hover),
            ("accent", self.accent),
            ("error", self.error),
            ("warning", self.warning),
            ("success", self.success),
            ("info", self.info),
            ("border", self.border),
            ("shadow", self.shadow),
            ("ink", self.ink),
            ("paper", self.paper),
            ("text-secondary", self.text_secondary),
            ("text-bright", self.text_bright),
            ("agent", self.agent),
            ("agent-attention", self.agent_attention),
            ("link", self.link),
            ("selection", self.selection),
            ("cursor", self.cursor),
            ("search", self.search),
            ("structural", self.structural),
        ]
    }
}

impl ColorPalette {
    /// Look up a palette entry by snake_case name (e.g. "frost_1", "ink").
    /// Returns `None` for unknown names.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Rgb> {
        Some(match name {
            "polar_night_0" => self.polar_night_0,
            "polar_night_1" => self.polar_night_1,
            "polar_night_2" => self.polar_night_2,
            "polar_night_3" => self.polar_night_3,
            "snow_storm_0" => self.snow_storm_0,
            "snow_storm_1" => self.snow_storm_1,
            "snow_storm_2" => self.snow_storm_2,
            "frost_0" => self.frost_0,
            "frost_1" => self.frost_1,
            "frost_2" => self.frost_2,
            "frost_3" => self.frost_3,
            "aurora_red" => self.aurora_red,
            "aurora_orange" => self.aurora_orange,
            "aurora_yellow" => self.aurora_yellow,
            "aurora_green" => self.aurora_green,
            "aurora_purple" => self.aurora_purple,
            "ink" => self.ink,
            "paper" => self.paper,
            "shadow_tone" => self.shadow_tone,
            _ => return None,
        })
    }

    /// Every palette entry as (name, Rgb) pairs, in stable order.
    #[must_use]
    pub fn entries(&self) -> [(&'static str, Rgb); 19] {
        [
            ("polar_night_0", self.polar_night_0),
            ("polar_night_1", self.polar_night_1),
            ("polar_night_2", self.polar_night_2),
            ("polar_night_3", self.polar_night_3),
            ("snow_storm_0", self.snow_storm_0),
            ("snow_storm_1", self.snow_storm_1),
            ("snow_storm_2", self.snow_storm_2),
            ("frost_0", self.frost_0),
            ("frost_1", self.frost_1),
            ("frost_2", self.frost_2),
            ("frost_3", self.frost_3),
            ("aurora_red", self.aurora_red),
            ("aurora_orange", self.aurora_orange),
            ("aurora_yellow", self.aurora_yellow),
            ("aurora_green", self.aurora_green),
            ("aurora_purple", self.aurora_purple),
            ("ink", self.ink),
            ("paper", self.paper),
            ("shadow_tone", self.shadow_tone),
        ]
    }
}
