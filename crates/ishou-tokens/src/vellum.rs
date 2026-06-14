//! Vellum — the fleet theme, warm aged-paper Nord-matte.
//!
//! Aged-paper substrate (a warm, low-chroma parchment ground) + computation
//! as muted matte ink above it. Every hex here is gamut-fitted from authored
//! targets or is an exact linear-space blend product. Colors are BORN here;
//! every downstream value is a token reference, never a hand-authored hex.
//!
//! The fleet theme is "Vellum". Verified mechanically by
//! `tests/vellum_matrix.rs` — the WCAG 2.1 contrast matrix over every
//! default pairing plus the byte-exact blend-product assertions. A token
//! edit that breaks legibility fails the build.
//!
//! # Bands
//!
//! The band FIELD names are functional labels carried over from the
//! engine; only the authored values change to the warm aged-paper palette.
//!
//! * **Night** — the parchment ground (backgrounds, darkest-to-lightest).
//! * **Shadow + Snow** — the text axis (warm ink → warm cream).
//! * **Ice** — the cool/steel matte accents.
//! * **Aurora dual-duty** — accents legacy TUIs also use as backgrounds.
//! * **Aurora glow** — the emissions (incl. `aurora_green`, THE signature,
//!   and `fable_violet`, the AGENT-RESERVED accent).
//! * **Bright** — the brighter accent tier (ANSI 9–14 + base24 12–17).
//! * **Derived** — exact linear-space blend products (selection, search,
//!   the GLASS band). The token IS the blend output, byte-exact.

use serde::Serialize;

use crate::color::Rgb;

// ─── Linear-space blend (f64 — matches the compute pipeline) ────────────────

fn channel_to_linear(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_channel(c: f64) -> u8 {
    let v = if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    let scaled = (v * 255.0).round();
    if scaled.is_nan() {
        return 0;
    }
    scaled.clamp(0.0, 255.0) as u8
}

/// Linear-space alpha blend: `(1-alpha)·bg + alpha·accent`, computed in
/// linear RGB (IEC 61966-2-1), quantised back to 8-bit sRGB.
///
/// This is THE derivation for every Vellum derived token (selection,
/// `search_others`, the glass band). The token equals this function's
/// output byte-exactly — `tests/vellum_matrix.rs` enforces zero
/// tolerance. sRGB-space compositors (CSS) consume the RESOLVED hex,
/// never an alpha recipe.
#[must_use]
pub fn blend_linear(bg: Rgb, accent: Rgb, alpha: f64) -> Rgb {
    let blend = |b: u8, a: u8| {
        linear_to_channel((1.0 - alpha) * channel_to_linear(b) + alpha * channel_to_linear(a))
    };
    Rgb::new(
        blend(bg.r, accent.r),
        blend(bg.g, accent.g),
        blend(bg.b, accent.b),
    )
}

/// A token paired with the alpha it is painted at (overlay surfaces the
/// renderer composites live: bell flash, URL underline, scrollbar, …).
/// Where a token must be consumed OPAQUE, the resolved blend product is
/// a first-class palette member instead (selection, glass band).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct AlphaPaint {
    pub color: Rgb,
    pub alpha: f32,
}

// ─── The palette ────────────────────────────────────────────────────────────

/// A band-structured authored palette. Theme-agnostic engine: the band
/// FIELD names are functional labels; each constructor authors a concrete
/// theme's values onto them. Derived tokens computed via [`blend_linear`]
/// at construction so they can never drift from their recipes.
///
/// Two themes ship today: [`Palette::vellum`] (warm aged-paper, the fleet
/// default) and [`Palette::polar_veil`] (cool neutral deep-polar-night).
#[derive(Debug, Clone, Serialize)]
pub struct Palette {
    // Night band — the parchment ground (darkest-to-lightest).
    pub night_abyss: Rgb,
    pub night_deep: Rgb,
    pub night0: Rgb,
    pub night1: Rgb,
    pub night2: Rgb,
    pub night3: Rgb,
    // Shadow + Snow band — the warm text axis.
    pub shadow0: Rgb,
    pub shadow1: Rgb,
    pub snow0: Rgb,
    pub snow1: Rgb,
    pub snow2: Rgb,
    pub snow3: Rgb,
    // Ice band — cool/steel matte accents.
    pub ice_teal: Rgb,
    pub ice_cyan: Rgb,
    pub ice_steel: Rgb,
    pub ice_deep: Rgb,
    // Aurora band, dual-duty tier.
    pub aurora_red: Rgb,
    pub solar_magenta: Rgb,
    pub dusk_bronze: Rgb,
    // Aurora band, glow tier.
    pub aurora_green: Rgb,
    pub first_light: Rgb,
    pub ember: Rgb,
    /// AGENT-RESERVED: vigy, MCP activity, AI surfaces, selection tint.
    /// NOT in base16/ANSI — downstream references the SEMANTIC
    /// (`agent` role), never the hex.
    pub fable_violet: Rgb,
    // Bright tier — the brighter accent tier (ANSI 9–14 + base24 base12–17).
    pub red_bright: Rgb,
    pub green_bright: Rgb,
    pub amber_bright: Rgb,
    pub steel_bright: Rgb,
    pub magenta_bright: Rgb,
    pub cyan_bright: Rgb,
    pub violet_bright: Rgb,
    // Derived tokens — exact blend products (byte-exact by construction).
    pub selection: Rgb,
    pub search_others: Rgb,
    pub red_glass: Rgb,
    pub green_glass: Rgb,
    pub amber_glass: Rgb,
    pub steel_glass: Rgb,
    pub cyan_glass: Rgb,
    pub violet_glass: Rgb,
}

/// Back-compat alias. Consumers wrote `VellumPalette` before the engine was
/// generalized to hold more than one theme; the name still resolves to the
/// theme-agnostic [`Palette`] so `VellumPalette::vellum()` keeps compiling.
pub type VellumPalette = Palette;

impl Palette {
    /// The fleet theme — `vellum` (warm aged-paper Nord-matte). Derived
    /// tokens are computed from their blend recipes here, never authored.
    #[must_use]
    pub fn vellum() -> Self {
        // Authored band constants (warm aged-paper Nord-matte).
        let night_abyss = Rgb::new(0x0D, 0x0C, 0x08);
        let night_deep = Rgb::new(0x10, 0x0E, 0x0A);
        let night0 = Rgb::new(0x16, 0x14, 0x0E);
        let night1 = Rgb::new(0x1F, 0x1C, 0x15);
        let night2 = Rgb::new(0x2B, 0x28, 0x20);
        let night3 = Rgb::new(0x38, 0x34, 0x2A);
        let shadow0 = Rgb::new(0x6E, 0x68, 0x57);
        let shadow1 = Rgb::new(0x90, 0x89, 0x7B);
        let snow0 = Rgb::new(0xAD, 0xA5, 0x93);
        let snow1 = Rgb::new(0xE2, 0xDB, 0xC8);
        let snow2 = Rgb::new(0xED, 0xE6, 0xD6);
        let snow3 = Rgb::new(0xF4, 0xEF, 0xE2);
        let ice_teal = Rgb::new(0x8F, 0xB9, 0xAB);
        let ice_cyan = Rgb::new(0x94, 0xBB, 0xB8);
        let ice_steel = Rgb::new(0x99, 0xAA, 0xBE);
        let ice_deep = Rgb::new(0x7E, 0x92, 0xAC);
        let aurora_red = Rgb::new(0xC9, 0x83, 0x7B);
        let solar_magenta = Rgb::new(0xB8, 0xA1, 0xB9);
        let dusk_bronze = Rgb::new(0xB3, 0x88, 0x6C);
        let aurora_green = Rgb::new(0xA9, 0xBB, 0x8C);
        let first_light = Rgb::new(0xD7, 0xC4, 0x89);
        let ember = Rgb::new(0xCB, 0x90, 0x70);
        let fable_violet = Rgb::new(0xB2, 0x9E, 0xC4);
        let red_bright = Rgb::new(0xD4, 0x90, 0x88);
        let green_bright = Rgb::new(0xAD, 0xD7, 0xA3);
        let amber_bright = Rgb::new(0xE0, 0xCF, 0x96);
        let steel_bright = Rgb::new(0xA8, 0xB7, 0xCB);
        let magenta_bright = Rgb::new(0xC6, 0xB0, 0xC7);
        let cyan_bright = Rgb::new(0xA6, 0xCB, 0xC8);
        let violet_bright = Rgb::new(0xC2, 0xB0, 0xD2);

        Self {
            night_abyss,
            night_deep,
            night0,
            night1,
            night2,
            night3,
            shadow0,
            shadow1,
            snow0,
            snow1,
            snow2,
            snow3,
            ice_teal,
            ice_cyan,
            ice_steel,
            ice_deep,
            aurora_red,
            solar_magenta,
            dusk_bronze,
            aurora_green,
            first_light,
            ember,
            fable_violet,
            red_bright,
            green_bright,
            amber_bright,
            steel_bright,
            magenta_bright,
            cyan_bright,
            violet_bright,
            // Derived — the token IS the blend output, byte-exact.
            selection: blend_linear(night0, fable_violet, 0.08),
            search_others: blend_linear(night0, first_light, 0.075),
            red_glass: blend_linear(night0, aurora_red, 0.327),
            green_glass: blend_linear(night0, aurora_green, 0.168),
            amber_glass: blend_linear(night0, first_light, 0.137),
            steel_glass: blend_linear(night0, ice_steel, 0.325),
            cyan_glass: blend_linear(night0, ice_cyan, 0.173),
            violet_glass: blend_linear(night0, fable_violet, 0.220),
        }
    }

    /// **Polar Veil** — the cool/neutral deep-polar-night sibling theme.
    /// The SAME band structure as [`Palette::vellum`], authored with a
    /// cooler, lower-warmth palette. Derived tokens use the identical
    /// [`blend_linear`] recipes, so the glass band comes out cool
    /// automatically. base16-faithful (the cool deep-polar matte ground).
    #[must_use]
    pub fn polar_veil() -> Self {
        // Authored band constants (cool/neutral deep-polar-night).
        let night_abyss = Rgb::new(0x0E, 0x10, 0x16);
        let night_deep = Rgb::new(0x12, 0x14, 0x1B);
        let night0 = Rgb::new(0x17, 0x1A, 0x22); // base00
        let night1 = Rgb::new(0x21, 0x24, 0x2F); // base01
        let night2 = Rgb::new(0x2C, 0x31, 0x40); // base02 — ANSI slot 0
        let night3 = Rgb::new(0x3A, 0x41, 0x50); // dividers, a step above night2
        let shadow0 = Rgb::new(0x96, 0x9E, 0xB1); // ANSI-8 grey
        let shadow1 = Rgb::new(0x8A, 0x93, 0xA8); // base03 comment
        let snow0 = Rgb::new(0xA7, 0xAE, 0xBC); // base04
        let snow1 = Rgb::new(0xD3, 0xD9, 0xE3); // base05
        let snow2 = Rgb::new(0xE2, 0xE6, 0xEE); // base06
        let snow3 = Rgb::new(0xF0, 0xF3, 0xF8); // base07 — ANSI 15
        let ice_teal = Rgb::new(0x8F, 0xBE, 0xB6);
        let ice_cyan = Rgb::new(0x8C, 0xC0, 0xC6); // base0C
        let ice_steel = Rgb::new(0x88, 0xA8, 0xCC); // base0D
        let ice_deep = Rgb::new(0x6E, 0x86, 0xAA);
        let aurora_red = Rgb::new(0xCC, 0x70, 0x7A); // base08
        let solar_magenta = Rgb::new(0xB6, 0x9C, 0xC2); // base0E
        let dusk_bronze = Rgb::new(0xAE, 0x82, 0x70); // base0F
        let aurora_green = Rgb::new(0xA1, 0xBC, 0x8B); // base0B
        let first_light = Rgb::new(0xDC, 0xC2, 0x87); // base0A
        let ember = Rgb::new(0xD0, 0x8C, 0x6E); // base09
        let fable_violet = Rgb::new(0xB0, 0x9C, 0xD2); // AGENT-RESERVED
        let red_bright = Rgb::new(0xD8, 0x7F, 0x88);
        // ANSI-10 br-green. This field is dual-duty cursor + ANSI-10 in the
        // shared engine (as in vellum); the spec's `ansi_16()` array fixes
        // it to #AEC79A, so that is the authoritative value.
        let green_bright = Rgb::new(0xAE, 0xC7, 0x9A);
        let amber_bright = Rgb::new(0xE6, 0xCE, 0x96);
        let steel_bright = Rgb::new(0x9A, 0xB6, 0xD6);
        let magenta_bright = Rgb::new(0xC4, 0xAC, 0xCE);
        let cyan_bright = Rgb::new(0x9C, 0xCC, 0xD2);
        let violet_bright = Rgb::new(0xC7, 0xB6, 0xDE);

        Self {
            night_abyss,
            night_deep,
            night0,
            night1,
            night2,
            night3,
            shadow0,
            shadow1,
            snow0,
            snow1,
            snow2,
            snow3,
            ice_teal,
            ice_cyan,
            ice_steel,
            ice_deep,
            aurora_red,
            solar_magenta,
            dusk_bronze,
            aurora_green,
            first_light,
            ember,
            fable_violet,
            red_bright,
            green_bright,
            amber_bright,
            steel_bright,
            magenta_bright,
            cyan_bright,
            violet_bright,
            // Derived — the token IS the blend output, byte-exact. The SAME
            // recipes as vellum; cool inputs yield cool outputs.
            selection: blend_linear(night0, fable_violet, 0.08),
            search_others: blend_linear(night0, first_light, 0.075),
            red_glass: blend_linear(night0, aurora_red, 0.327),
            green_glass: blend_linear(night0, aurora_green, 0.168),
            amber_glass: blend_linear(night0, first_light, 0.137),
            steel_glass: blend_linear(night0, ice_steel, 0.325),
            cyan_glass: blend_linear(night0, ice_cyan, 0.173),
            violet_glass: blend_linear(night0, fable_violet, 0.220),
        }
    }

    /// Look up a Vellum token by snake_case name. Brand monochromes
    /// (`ink` / `paper` / `shadow_tone`) resolve through the shared
    /// pleme palette so the role map can reference them unchanged.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Rgb> {
        Some(match name {
            "night_abyss" => self.night_abyss,
            "night_deep" => self.night_deep,
            "night0" => self.night0,
            "night1" => self.night1,
            "night2" => self.night2,
            "night3" => self.night3,
            "shadow0" => self.shadow0,
            "shadow1" => self.shadow1,
            "snow0" => self.snow0,
            "snow1" => self.snow1,
            "snow2" => self.snow2,
            "snow3" => self.snow3,
            "ice_teal" => self.ice_teal,
            "ice_cyan" => self.ice_cyan,
            "ice_steel" => self.ice_steel,
            "ice_deep" => self.ice_deep,
            "aurora_red" => self.aurora_red,
            "solar_magenta" => self.solar_magenta,
            "dusk_bronze" => self.dusk_bronze,
            "aurora_green" => self.aurora_green,
            "first_light" => self.first_light,
            "ember" => self.ember,
            "fable_violet" => self.fable_violet,
            "red_bright" => self.red_bright,
            "green_bright" => self.green_bright,
            "amber_bright" => self.amber_bright,
            "steel_bright" => self.steel_bright,
            "magenta_bright" => self.magenta_bright,
            "cyan_bright" => self.cyan_bright,
            "violet_bright" => self.violet_bright,
            "selection" => self.selection,
            "search_others" => self.search_others,
            "red_glass" => self.red_glass,
            "green_glass" => self.green_glass,
            "amber_glass" => self.amber_glass,
            "steel_glass" => self.steel_glass,
            "cyan_glass" => self.cyan_glass,
            "violet_glass" => self.violet_glass,
            _ => return crate::color::ColorPalette::pleme().get(name),
        })
    }

    /// Every Vellum token as (name, Rgb) pairs, in band order.
    #[must_use]
    pub fn entries(&self) -> [(&'static str, Rgb); 38] {
        [
            ("night_abyss", self.night_abyss),
            ("night_deep", self.night_deep),
            ("night0", self.night0),
            ("night1", self.night1),
            ("night2", self.night2),
            ("night3", self.night3),
            ("shadow0", self.shadow0),
            ("shadow1", self.shadow1),
            ("snow0", self.snow0),
            ("snow1", self.snow1),
            ("snow2", self.snow2),
            ("snow3", self.snow3),
            ("ice_teal", self.ice_teal),
            ("ice_cyan", self.ice_cyan),
            ("ice_steel", self.ice_steel),
            ("ice_deep", self.ice_deep),
            ("aurora_red", self.aurora_red),
            ("solar_magenta", self.solar_magenta),
            ("dusk_bronze", self.dusk_bronze),
            ("aurora_green", self.aurora_green),
            ("first_light", self.first_light),
            ("ember", self.ember),
            ("fable_violet", self.fable_violet),
            ("red_bright", self.red_bright),
            ("green_bright", self.green_bright),
            ("amber_bright", self.amber_bright),
            ("steel_bright", self.steel_bright),
            ("magenta_bright", self.magenta_bright),
            ("cyan_bright", self.cyan_bright),
            ("violet_bright", self.violet_bright),
            ("selection", self.selection),
            ("search_others", self.search_others),
            ("red_glass", self.red_glass),
            ("green_glass", self.green_glass),
            ("amber_glass", self.amber_glass),
            ("steel_glass", self.steel_glass),
            ("cyan_glass", self.cyan_glass),
            ("violet_glass", self.violet_glass),
        ]
    }

    /// ONE canonical ANSI-16 mapping fleet-wide. Slots 1/4/5 are the
    /// dual-duty tier; slot 15 is `snow3` (= base07).
    #[must_use]
    pub fn ansi_16(&self) -> [Rgb; 16] {
        [
            self.night2,         // 0  black — surface, NEVER base00
            self.aurora_red,     // 1  red — dual-duty
            self.aurora_green,   // 2  green — the signature
            self.first_light,    // 3  yellow
            self.ice_steel,      // 4  blue — dual-duty
            self.solar_magenta,  // 5  magenta — dual-duty
            self.ice_cyan,       // 6  cyan
            self.snow0,          // 7  white
            self.shadow0,        // 8  br-black — dim tier (typed floor 3.0)
            self.red_bright,     // 9  br-red
            self.green_bright,   // 10 br-green
            self.amber_bright,   // 11 br-yellow
            self.steel_bright,   // 12 br-blue
            self.magenta_bright, // 13 br-magenta
            self.cyan_bright,    // 14 br-cyan
            self.snow3,          // 15 br-white — = base07, never #FFFFFF
        ]
    }

    /// base16 slots — canonical slot semantics. base02 is the violet-
    /// carrying selection blend BY DESIGN.
    #[must_use]
    pub fn base16(&self) -> [(&'static str, Rgb); 16] {
        [
            ("base00", self.night0),
            ("base01", self.night1),
            ("base02", self.selection),
            ("base03", self.shadow1),
            ("base04", self.snow0),
            ("base05", self.snow1),
            ("base06", self.snow2),
            ("base07", self.snow3),
            ("base08", self.aurora_red),
            ("base09", self.ember),
            ("base0A", self.first_light),
            ("base0B", self.aurora_green),
            ("base0C", self.ice_cyan),
            ("base0D", self.ice_steel),
            ("base0E", self.solar_magenta),
            ("base0F", self.dusk_bronze),
        ]
    }

    /// base24 sibling — base16 + real two-tier brights (base10–17).
    #[must_use]
    pub fn base24(&self) -> [(&'static str, Rgb); 24] {
        let b16 = self.base16();
        [
            b16[0], b16[1], b16[2], b16[3], b16[4], b16[5], b16[6], b16[7],
            b16[8], b16[9], b16[10], b16[11], b16[12], b16[13], b16[14],
            b16[15],
            ("base10", self.night_deep),
            ("base11", self.night_abyss),
            ("base12", self.red_bright),
            ("base13", self.amber_bright),
            ("base14", self.green_bright),
            ("base15", self.cyan_bright),
            ("base16", self.steel_bright),
            ("base17", self.magenta_bright),
        ]
    }

    /// Typed FgPromotion map — fires in fleet renderers when a fg's
    /// composite over a highlight surface computes <3.0. Promotion
    /// preserves hue family; targets verified by the contrast matrix.
    /// Boundary: applies to mado/ghostty grid+overlay composites and
    /// nvim groups we author; cannot reach foreign compositors.
    #[must_use]
    pub const fn fg_promotions() -> [(&'static str, &'static str); 6] {
        [
            ("shadow0", "snow0"),
            ("shadow1", "snow0"),
            ("aurora_red", "red_bright"),
            ("ice_steel", "steel_bright"),
            ("solar_magenta", "magenta_bright"),
            ("dusk_bronze", "ember"),
        ]
    }

    /// The terminal surface map — mado QUEUED spec + the ghostty render
    /// target both consume this.
    #[must_use]
    pub fn surfaces(&self) -> TerminalSurfaces {
        TerminalSurfaces {
            background: self.night0,
            foreground: self.snow1,
            cursor: self.green_bright,
            cursor_text: self.night0,
            selection_background: self.selection,
            selection_foreground: self.snow2,
            search_others_background: self.search_others,
            search_current_background: self.first_light,
            search_current_foreground: self.night0,
            search_status_text: self.ice_cyan,
            bell_flash: AlphaPaint { color: self.snow3, alpha: 0.06 },
            url_underline: AlphaPaint { color: self.ice_cyan, alpha: 0.60 },
            scrollbar_thumb: AlphaPaint { color: self.ice_cyan, alpha: 0.35 },
            osc133_separator: AlphaPaint { color: self.ice_deep, alpha: 0.30 },
            split_divider: self.night3,
            unfocused_split_fill: self.night_deep,
            titlebar_band: self.night1,
            agent_attention_surface: self.violet_glass,
            minimum_contrast: 3.0,
            bold_is_bright: false,
            workspace_tint_pleme: self.night0,
            workspace_tint_akeyless: self.night_deep,
        }
    }
}

/// The terminal surface map as one typed value. `minimum_contrast`
/// and `bold_is_bright` are Guard-pinned fleet decisions (bold renders
/// as weight, brights via SGR 90–97).
#[derive(Debug, Clone, Serialize)]
pub struct TerminalSurfaces {
    pub background: Rgb,
    pub foreground: Rgb,
    /// Block cursor — `green_bright` (inverse pair ≥7.0).
    pub cursor: Rgb,
    pub cursor_text: Rgb,
    /// Opaque consumers use this token; the GPU path composites
    /// `fable_violet` @ α0.08 LINEAR — identical hex by construction.
    pub selection_background: Rgb,
    pub selection_foreground: Rgb,
    pub search_others_background: Rgb,
    /// INVERSE VIDEO — the only inverted element on screen.
    pub search_current_background: Rgb,
    pub search_current_foreground: Rgb,
    pub search_status_text: Rgb,
    pub bell_flash: AlphaPaint,
    pub url_underline: AlphaPaint,
    pub scrollbar_thumb: AlphaPaint,
    pub osc133_separator: AlphaPaint,
    pub split_divider: Rgb,
    pub unfocused_split_fill: Rgb,
    pub titlebar_band: Rgb,
    pub agent_attention_surface: Rgb,
    /// Grid-cell safety net for raw-ANSI legacy pairs.
    pub minimum_contrast: f32,
    /// Declared decision: false fleet-wide; bold = weight.
    pub bold_is_bright: bool,
    pub workspace_tint_pleme: Rgb,
    pub workspace_tint_akeyless: Rgb,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_tokens_match_spec_hexes() {
        let p = VellumPalette::vellum();
        let expect = [
            ("night_abyss", "#0D0C08"),
            ("night_deep", "#100E0A"),
            ("night0", "#16140E"),
            ("night1", "#1F1C15"),
            ("night2", "#2B2820"),
            ("night3", "#38342A"),
            ("shadow0", "#6E6857"),
            ("shadow1", "#90897B"),
            ("snow0", "#ADA593"),
            ("snow1", "#E2DBC8"),
            ("snow2", "#EDE6D6"),
            ("snow3", "#F4EFE2"),
            ("ice_teal", "#8FB9AB"),
            ("ice_cyan", "#94BBB8"),
            ("ice_steel", "#99AABE"),
            ("ice_deep", "#7E92AC"),
            ("aurora_red", "#C9837B"),
            ("solar_magenta", "#B8A1B9"),
            ("dusk_bronze", "#B3886C"),
            ("aurora_green", "#A9BB8C"),
            ("first_light", "#D7C489"),
            ("ember", "#CB9070"),
            ("fable_violet", "#B29EC4"),
            ("red_bright", "#D49088"),
            ("green_bright", "#ADD7A3"),
            ("amber_bright", "#E0CF96"),
            ("steel_bright", "#A8B7CB"),
            ("magenta_bright", "#C6B0C7"),
            ("cyan_bright", "#A6CBC8"),
            ("violet_bright", "#C2B0D2"),
        ];
        let mut failures = Vec::new();
        for (name, hex) in expect {
            let got = p.get(name).unwrap().hex();
            if got != hex {
                failures.push(format!("{name}: got {got}, spec {hex}"));
            }
        }
        assert!(failures.is_empty(), "token drift:\n  - {}", failures.join("\n  - "));
    }

    #[test]
    fn polar_veil_authored_tokens_match_spec_hexes() {
        let p = Palette::polar_veil();
        let expect = [
            ("night_abyss", "#0E1016"),
            ("night_deep", "#12141B"),
            ("night0", "#171A22"),
            ("night1", "#21242F"),
            ("night2", "#2C3140"),
            ("night3", "#3A4150"),
            ("shadow0", "#969EB1"),
            ("shadow1", "#8A93A8"),
            ("snow0", "#A7AEBC"),
            ("snow1", "#D3D9E3"),
            ("snow2", "#E2E6EE"),
            ("snow3", "#F0F3F8"),
            ("ice_teal", "#8FBEB6"),
            ("ice_cyan", "#8CC0C6"),
            ("ice_steel", "#88A8CC"),
            ("ice_deep", "#6E86AA"),
            ("aurora_red", "#CC707A"),
            ("solar_magenta", "#B69CC2"),
            ("dusk_bronze", "#AE8270"),
            ("aurora_green", "#A1BC8B"),
            ("first_light", "#DCC287"),
            ("ember", "#D08C6E"),
            ("fable_violet", "#B09CD2"),
            ("red_bright", "#D87F88"),
            ("green_bright", "#AEC79A"),
            ("amber_bright", "#E6CE96"),
            ("steel_bright", "#9AB6D6"),
            ("magenta_bright", "#C4ACCE"),
            ("cyan_bright", "#9CCCD2"),
            ("violet_bright", "#C7B6DE"),
        ];
        let mut failures = Vec::new();
        for (name, hex) in expect {
            let got = p.get(name).unwrap().hex();
            if got != hex {
                failures.push(format!("{name}: got {got}, spec {hex}"));
            }
        }
        assert!(failures.is_empty(), "polar_veil token drift:\n  - {}", failures.join("\n  - "));
    }

    #[test]
    fn polar_veil_ansi_16_matches_spec_table() {
        // The base16 ANSI-16 surface, cool deep-polar-night.
        let p = Palette::polar_veil();
        let ansi: Vec<String> = p.ansi_16().iter().map(super::Rgb::hex).collect();
        let expect = [
            "#2C3140", "#CC707A", "#A1BC8B", "#DCC287", "#88A8CC", "#B69CC2",
            "#8CC0C6", "#A7AEBC", "#969EB1", "#D87F88", "#AEC79A", "#E6CE96",
            "#9AB6D6", "#C4ACCE", "#9CCCD2", "#F0F3F8",
        ];
        assert_eq!(ansi, expect, "polar_veil ANSI-16 drift");
    }

    #[test]
    fn derived_tokens_are_byte_exact_blend_products() {
        // The token IS the blend output, byte-exact, zero tolerance.
        // These also pin the published hexes so the recipe and the
        // published value can never diverge.
        let p = VellumPalette::vellum();
        let rows: [(&str, Rgb, &str); 8] = [
            ("selection", blend_linear(p.night0, p.fable_violet, 0.08), "#3A343E"),
            ("search_others", blend_linear(p.night0, p.first_light, 0.075), "#443E2A"),
            ("red_glass", blend_linear(p.night0, p.aurora_red, 0.327), "#7B4F4A"),
            ("green_glass", blend_linear(p.night0, p.aurora_green, 0.168), "#4D543E"),
            ("amber_glass", blend_linear(p.night0, p.first_light, 0.137), "#595137"),
            ("steel_glass", blend_linear(p.night0, p.ice_steel, 0.325), "#5D6773"),
            ("cyan_glass", blend_linear(p.night0, p.ice_cyan, 0.173), "#445553"),
            ("violet_glass", blend_linear(p.night0, p.fable_violet, 0.220), "#5B5063"),
        ];
        let mut failures = Vec::new();
        for (name, blended, spec_hex) in rows {
            let stored = p.get(name).unwrap().hex();
            if stored != blended.hex() {
                failures.push(format!("{name}: stored {stored} != blend {}", blended.hex()));
            }
            if stored != spec_hex {
                failures.push(format!("{name}: stored {stored} != spec {spec_hex}"));
            }
        }
        assert!(failures.is_empty(), "blend drift:\n  - {}", failures.join("\n  - "));
    }

    #[test]
    fn ansi_16_mapping_matches_spec_table() {
        let p = VellumPalette::vellum();
        let ansi = p.ansi_16();
        assert_eq!(ansi[0].hex(), "#2B2820"); // night2, NEVER base00
        assert_eq!(ansi[1].hex(), "#C9837B");
        assert_eq!(ansi[2].hex(), "#A9BB8C");
        assert_eq!(ansi[3].hex(), "#D7C489");
        assert_eq!(ansi[4].hex(), "#99AABE");
        assert_eq!(ansi[5].hex(), "#B8A1B9");
        assert_eq!(ansi[6].hex(), "#94BBB8");
        assert_eq!(ansi[7].hex(), "#ADA593");
        assert_eq!(ansi[8].hex(), "#6E6857");
        assert_eq!(ansi[9].hex(), "#D49088");
        assert_eq!(ansi[10].hex(), "#ADD7A3");
        assert_eq!(ansi[11].hex(), "#E0CF96");
        assert_eq!(ansi[12].hex(), "#A8B7CB");
        assert_eq!(ansi[13].hex(), "#C6B0C7");
        assert_eq!(ansi[14].hex(), "#A6CBC8");
        assert_eq!(ansi[15].hex(), "#F4EFE2"); // snow3 = base07
    }

    #[test]
    fn surfaces_pin_fleet_decisions() {
        let s = VellumPalette::vellum().surfaces();
        assert!((s.minimum_contrast - 3.0).abs() < f32::EPSILON);
        assert!(!s.bold_is_bright);
        assert_eq!(s.cursor.hex(), "#ADD7A3"); // green_bright
        assert_eq!(s.cursor_text.hex(), "#16140E");
        assert_eq!(s.selection_background.hex(), "#3A343E");
        assert_eq!(s.search_current_background.hex(), "#D7C489");
        assert_eq!(s.titlebar_band.hex(), "#1F1C15");
    }

    #[test]
    fn fg_promotion_map_is_total_over_the_six_sub_floor_tokens() {
        let names: Vec<&str> = VellumPalette::fg_promotions().iter().map(|(f, _)| *f).collect();
        for required in ["shadow0", "shadow1", "aurora_red", "ice_steel", "solar_magenta", "dusk_bronze"] {
            assert!(names.contains(&required), "missing promotion for {required}");
        }
        // Every target resolves.
        let p = VellumPalette::vellum();
        for (from, to) in VellumPalette::fg_promotions() {
            assert!(p.get(from).is_some() && p.get(to).is_some(), "{from}→{to} unresolvable");
        }
    }
}
