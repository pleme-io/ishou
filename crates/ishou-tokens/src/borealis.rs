//! Borealis — the fleet theme, dark variant `borealis-night`.
//!
//! Frozen arctic substrate (the Nord-frost heritage, the ❄) + computation
//! as the aurora above it. Every hex here is gamut-fitted from authored
//! OKLCH targets or is an exact linear-space blend product. Colors are
//! BORN here; every downstream value is a token reference, never a
//! hand-authored hex.
//!
//! Canonical spec: BOREALIS fleet theme spec v2 (judge-approved).
//! Verified mechanically by `tests/borealis_matrix.rs` — the WCAG 2.1
//! contrast matrix over every default pairing, the OKLab ladder/tier
//! invariants, and the byte-exact blend-product assertions. A token
//! edit that breaks legibility fails the build.
//!
//! # Bands
//!
//! * **Night** — the arctic sky (backgrounds, ΔL=0.050 ladder).
//! * **Shadow + Snow** — moonlit snow text axis (ΔL=0.045 lattice).
//! * **Ice** — stratified frost accents (Frost lineage).
//! * **Aurora dual-duty** — accents legacy TUIs also use as backgrounds
//!   (one luminance row; text ≥4.5 on night0 AND snow3-over-slot ≥3.0).
//! * **Aurora glow** — the emissions (incl. `aurora_green`, THE signature,
//!   and `fable_violet`, the AGENT-RESERVED accent).
//! * **Bright** — the glow bloom (ANSI 9–14 + base24 12–17).
//! * **Derived** — exact linear-space blend products (selection, search,
//!   the equal-luminance GLASS band). The token IS the blend output,
//!   byte-exact, zero tolerance.

use serde::Serialize;

use crate::color::Rgb;

// ─── Linear-space blend (f64 — matches the spec's compute pipeline) ────────

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
/// This is THE derivation for every Borealis derived token (selection,
/// `search_others`, the glass band). The token equals this function's
/// output byte-exactly — `tests/borealis_matrix.rs` enforces zero
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

/// Every named Borealis token. Authored constants for the four bands +
/// bright tier; derived tokens computed via [`blend_linear`] at
/// construction so they can never drift from their recipes.
#[derive(Debug, Clone, Serialize)]
pub struct BorealisPalette {
    // Night band — the arctic sky (ΔL=0.050 ladder, violet undertone).
    pub night_abyss: Rgb,
    pub night_deep: Rgb,
    pub night0: Rgb,
    pub night1: Rgb,
    pub night2: Rgb,
    pub night3: Rgb,
    // Shadow + Snow band — moonlit snow text axis (ΔL=0.045 lattice).
    pub shadow0: Rgb,
    pub shadow1: Rgb,
    pub snow0: Rgb,
    pub snow1: Rgb,
    pub snow2: Rgb,
    pub snow3: Rgb,
    // Ice band — stratified frost.
    pub ice_teal: Rgb,
    pub ice_cyan: Rgb,
    pub ice_steel: Rgb,
    pub ice_deep: Rgb,
    // Aurora band, dual-duty tier (one luminance row, L 0.644–0.661).
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
    // Bright tier — the glow bloom (ANSI 9–14 + base24 base12–17).
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

impl BorealisPalette {
    /// The dark variant — `borealis-night`. Derived tokens are computed
    /// from their blend recipes here, never authored.
    #[must_use]
    pub fn night() -> Self {
        // Authored band constants (gamut-fitted from OKLCH targets).
        let night_abyss = Rgb::new(0x0A, 0x0C, 0x14);
        let night_deep = Rgb::new(0x14, 0x16, 0x21);
        let night0 = Rgb::new(0x1F, 0x22, 0x2F);
        let night1 = Rgb::new(0x2B, 0x2E, 0x3D);
        let night2 = Rgb::new(0x38, 0x3B, 0x4C);
        let night3 = Rgb::new(0x45, 0x48, 0x59);
        let shadow0 = Rgb::new(0x73, 0x76, 0x89);
        let shadow1 = Rgb::new(0x8C, 0x92, 0xA4);
        let snow0 = Rgb::new(0xB5, 0xBC, 0xC9);
        let snow1 = Rgb::new(0xD4, 0xD9, 0xE3);
        let snow2 = Rgb::new(0xE4, 0xE8, 0xEF);
        let snow3 = Rgb::new(0xF5, 0xF7, 0xFA);
        let ice_teal = Rgb::new(0x6E, 0xB4, 0xA4);
        let ice_cyan = Rgb::new(0x73, 0xC6, 0xD9);
        let ice_steel = Rgb::new(0x6A, 0x90, 0xC0);
        let ice_deep = Rgb::new(0x57, 0x7C, 0xAF);
        let aurora_red = Rgb::new(0xD8, 0x6E, 0x67);
        let solar_magenta = Rgb::new(0xC6, 0x73, 0xA3);
        let dusk_bronze = Rgb::new(0xB3, 0x83, 0x63);
        let aurora_green = Rgb::new(0x67, 0xD1, 0x91);
        let first_light = Rgb::new(0xED, 0xC9, 0x80);
        let ember = Rgb::new(0xE8, 0x97, 0x72);
        let fable_violet = Rgb::new(0xB6, 0x9A, 0xE9);
        let red_bright = Rgb::new(0xFF, 0x84, 0x82);
        let green_bright = Rgb::new(0x74, 0xE2, 0x9F);
        let amber_bright = Rgb::new(0xFA, 0xD6, 0x8D);
        let steel_bright = Rgb::new(0x89, 0xB1, 0xE3);
        let magenta_bright = Rgb::new(0xF8, 0xA1, 0xD3);
        let cyan_bright = Rgb::new(0x7E, 0xD7, 0xEC);
        let violet_bright = Rgb::new(0xC6, 0xA9, 0xFC);

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

    /// Look up a Borealis token by snake_case name. Brand monochromes
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

    /// Every Borealis token as (name, Rgb) pairs, in band order.
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

    /// ONE canonical ANSI-16 mapping fleet-wide (spec §4). Slots 1/4/5
    /// are the dual-duty tier (text ≥4.5 on night0 AND snow3-over-slot
    /// ≥3.0); slot 15 is `snow3` (= base07) BECAUSE the dual-duty
    /// window is empty against snow2 — proven, not chosen.
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

    /// base16 slots (spec §3 — canonical slot semantics, all 7 port
    /// mistakes avoided). base02 is the violet-carrying selection
    /// blend BY DESIGN.
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

    /// The terminal surface map (spec §5) — mado QUEUED spec + the
    /// ghostty render target both consume this.
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
            bell_flash: AlphaPaint { color: self.snow3, alpha: 0.10 },
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

/// The §5 terminal surface map as one typed value. `minimum_contrast`
/// and `bold_is_bright` are Guard-pinned fleet decisions (M1 layer c +
/// M6 declared decision: bold renders as weight, brights via SGR 90–97).
#[derive(Debug, Clone, Serialize)]
pub struct TerminalSurfaces {
    pub background: Rgb,
    pub foreground: Rgb,
    /// Block cursor — `green_bright` (inverse pair ≥7.0, computed 9.88).
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
    /// Grid-cell safety net for raw-ANSI legacy pairs (M1 layer c).
    pub minimum_contrast: f32,
    /// Declared decision (M6): false fleet-wide; bold = weight.
    pub bold_is_bright: bool,
    pub workspace_tint_pleme: Rgb,
    pub workspace_tint_akeyless: Rgb,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_tokens_match_spec_hexes() {
        let p = BorealisPalette::night();
        let expect = [
            ("night_abyss", "#0A0C14"),
            ("night_deep", "#141621"),
            ("night0", "#1F222F"),
            ("night1", "#2B2E3D"),
            ("night2", "#383B4C"),
            ("night3", "#454859"),
            ("shadow0", "#737689"),
            ("shadow1", "#8C92A4"),
            ("snow0", "#B5BCC9"),
            ("snow1", "#D4D9E3"),
            ("snow2", "#E4E8EF"),
            ("snow3", "#F5F7FA"),
            ("ice_teal", "#6EB4A4"),
            ("ice_cyan", "#73C6D9"),
            ("ice_steel", "#6A90C0"),
            ("ice_deep", "#577CAF"),
            ("aurora_red", "#D86E67"),
            ("solar_magenta", "#C673A3"),
            ("dusk_bronze", "#B38363"),
            ("aurora_green", "#67D191"),
            ("first_light", "#EDC980"),
            ("ember", "#E89772"),
            ("fable_violet", "#B69AE9"),
            ("red_bright", "#FF8482"),
            ("green_bright", "#74E29F"),
            ("amber_bright", "#FAD68D"),
            ("steel_bright", "#89B1E3"),
            ("magenta_bright", "#F8A1D3"),
            ("cyan_bright", "#7ED7EC"),
            ("violet_bright", "#C6A9FC"),
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
    fn derived_tokens_are_byte_exact_blend_products() {
        // M5+M7: the token IS the blend output, byte-exact, zero
        // tolerance. These also pin the spec's published hexes so the
        // recipe and the published value can never diverge.
        let p = BorealisPalette::night();
        let rows: [(&str, Rgb, &str); 8] = [
            ("selection", blend_linear(p.night0, p.fable_violet, 0.08), "#3F3955"),
            ("search_others", blend_linear(p.night0, p.first_light, 0.075), "#4E443A"),
            ("red_glass", blend_linear(p.night0, p.aurora_red, 0.327), "#854647"),
            ("green_glass", blend_linear(p.night0, p.aurora_green, 0.168), "#34614B"),
            ("amber_glass", blend_linear(p.night0, p.first_light, 0.137), "#645642"),
            ("steel_glass", blend_linear(p.night0, p.ice_steel, 0.325), "#435A79"),
            ("cyan_glass", blend_linear(p.night0, p.ice_cyan, 0.173), "#395E6A"),
            ("violet_glass", blend_linear(p.night0, p.fable_violet, 0.220), "#5F527C"),
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
        let p = BorealisPalette::night();
        let ansi = p.ansi_16();
        assert_eq!(ansi[0].hex(), "#383B4C"); // night2, NEVER base00
        assert_eq!(ansi[1].hex(), "#D86E67");
        assert_eq!(ansi[2].hex(), "#67D191");
        assert_eq!(ansi[3].hex(), "#EDC980");
        assert_eq!(ansi[4].hex(), "#6A90C0");
        assert_eq!(ansi[5].hex(), "#C673A3");
        assert_eq!(ansi[6].hex(), "#73C6D9");
        assert_eq!(ansi[7].hex(), "#B5BCC9");
        assert_eq!(ansi[8].hex(), "#737689");
        assert_eq!(ansi[9].hex(), "#FF8482");
        assert_eq!(ansi[10].hex(), "#74E29F");
        assert_eq!(ansi[11].hex(), "#FAD68D");
        assert_eq!(ansi[12].hex(), "#89B1E3");
        assert_eq!(ansi[13].hex(), "#F8A1D3");
        assert_eq!(ansi[14].hex(), "#7ED7EC");
        assert_eq!(ansi[15].hex(), "#F5F7FA"); // snow3 = base07
    }

    #[test]
    fn surfaces_pin_fleet_decisions() {
        let s = BorealisPalette::night().surfaces();
        assert!((s.minimum_contrast - 3.0).abs() < f32::EPSILON);
        assert!(!s.bold_is_bright);
        assert_eq!(s.cursor.hex(), "#74E29F"); // green_bright
        assert_eq!(s.cursor_text.hex(), "#1F222F");
        assert_eq!(s.selection_background.hex(), "#3F3955");
        assert_eq!(s.search_current_background.hex(), "#EDC980");
        assert_eq!(s.titlebar_band.hex(), "#2B2E3D");
    }

    #[test]
    fn fg_promotion_map_is_total_over_the_six_sub_floor_tokens() {
        let names: Vec<&str> = BorealisPalette::fg_promotions().iter().map(|(f, _)| *f).collect();
        for required in ["shadow0", "shadow1", "aurora_red", "ice_steel", "solar_magenta", "dusk_bronze"] {
            assert!(names.contains(&required), "missing promotion for {required}");
        }
        // Every target resolves.
        let p = BorealisPalette::night();
        for (from, to) in BorealisPalette::fg_promotions() {
            assert!(p.get(from).is_some() && p.get(to).is_some(), "{from}→{to} unresolvable");
        }
    }
}
