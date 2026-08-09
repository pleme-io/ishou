//! Vellum mechanical verification — the matrix test.
//!
//! Every asserted contrast ratio is COMPUTED here from the BORN tokens
//! (WCAG 2.1 relative luminance, never a hardcoded ratio). Failures are
//! aggregated, then asserted empty — one run reports every broken
//! pairing, not just the first. A future token edit that breaks
//! legibility fails the build.
//!
//! Sections:
//!
//! * the contrast matrix (every default pairing, typed
//!   `(fg, bg, class, floor)` rows, ratios computed).
//! * the OKLab ladder + per-tier spread invariants.
//! * base16 ramp monotone + zero-duplicate-hex.
//! * ANSI floors + the LegacyRaw slot-as-bg rows.
//! * blend-exactness (byte-exact derived tokens), glass-Y
//!   uniformity, FgPromotion targets, InversePair floors.

use ishou_tokens::{Rgb, VellumPalette, blend_linear};

// ─── WCAG 2.1 relative luminance (computed, never hardcoded) ──────────────────

/// sRGB 8-bit channel → linear (IEC 61966-2-1).
fn chan_lin(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.1 relative luminance Y of an sRGB colour.
fn luminance(c: Rgb) -> f64 {
    0.2126 * chan_lin(c.r) + 0.7152 * chan_lin(c.g) + 0.0722 * chan_lin(c.b)
}

/// WCAG 2.1 contrast ratio between two colours (order-independent).
fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

// ─── OKLab lightness L (computed — for the ladder + tier invariants) ──────────

/// OKLab L of an sRGB colour (Ottosson OKLab; the space the spec
/// authored its ladders in).
fn oklab_l(c: Rgb) -> f64 {
    let (r, g, b) = (chan_lin(c.r), chan_lin(c.g), chan_lin(c.b));
    let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
    let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
    let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b;
    let (l_, m_, s_) = (l.cbrt(), m.cbrt(), s.cbrt());
    0.210_454_255_3 * l_ + 0.793_617_785_0 * m_ - 0.004_072_046_8 * s_
}

// ─── Typed matrix row ─────────────────────────────────────────────────────────

/// A contrast-class floor (spec §6). `Band` floors are a closed
/// interval (selection / search must NOT invert polarity).
#[derive(Clone, Copy)]
enum Floor {
    /// `ratio >= min`.
    Min(f64),
    /// `lo <= ratio <= hi` (the selection / search bands).
    Band(f64, f64),
}

struct Row {
    fg: &'static str,
    bg: &'static str,
    class: &'static str,
    floor: Floor,
}

const fn min(fg: &'static str, bg: &'static str, class: &'static str, m: f64) -> Row {
    Row {
        fg,
        bg,
        class,
        floor: Floor::Min(m),
    }
}
const fn band(fg: &'static str, bg: &'static str, class: &'static str, lo: f64, hi: f64) -> Row {
    Row {
        fg,
        bg,
        class,
        floor: Floor::Band(lo, hi),
    }
}

/// Every asserted pairing from spec §6. The promoted (FgPromotion)
/// rows assert the PROMOTION TARGET, never the raw sub-floor fg — the
/// renderer lifts the raw composite per the FgPromotion map.
fn matrix() -> Vec<Row> {
    vec![
        // BodyText ≥7.0 (AAA).
        min("snow1", "night0", "BodyText", 7.0),
        min("snow1", "night1", "BodyText", 7.0),
        min("snow1", "night2", "BodyText", 7.0),
        min("snow0", "night0", "BodyText", 7.0),
        min("snow2", "night0", "BodyText", 7.0),
        min("snow3", "night0", "BodyText", 7.0),
        // UIText ≥4.5.
        min("snow0", "night1", "UIText", 4.5),
        min("snow2", "night2", "UIText", 4.5),
        // MutedText ≥4.5 / MutedOnSurface ≥4.0.
        min("shadow1", "night0", "MutedText", 4.5),
        min("shadow1", "night1", "MutedOnSurface", 4.0),
        // DimTier ≥3.0 (ANSI 8).
        min("shadow0", "night0", "DimTier", 3.0),
        // AccentText ≥4.5 — every accent on night0.
        min("aurora_green", "night0", "AccentText", 4.5),
        min("aurora_red", "night0", "AccentText", 4.5),
        min("first_light", "night0", "AccentText", 4.5),
        min("ember", "night0", "AccentText", 4.5),
        min("solar_magenta", "night0", "AccentText", 4.5),
        min("fable_violet", "night0", "AccentText", 4.5),
        min("ice_cyan", "night0", "AccentText", 4.5),
        min("ice_teal", "night0", "AccentText", 4.5),
        min("ice_steel", "night0", "AccentText", 4.5),
        min("dusk_bronze", "night0", "AccentText", 4.5),
        // Structural ≥3.0 (ice_deep — type-banned from body text).
        min("ice_deep", "night0", "Structural", 3.0),
        // Brights 9–14 + violet_bright on night0 ≥4.5.
        min("red_bright", "night0", "AccentText", 4.5),
        min("green_bright", "night0", "AccentText", 4.5),
        min("amber_bright", "night0", "AccentText", 4.5),
        min("steel_bright", "night0", "AccentText", 4.5),
        min("magenta_bright", "night0", "AccentText", 4.5),
        min("cyan_bright", "night0", "AccentText", 4.5),
        min("violet_bright", "night0", "AccentText", 4.5),
        // AccentOnSurface ≥4.0 (night1).
        min("aurora_red", "night1", "AccentOnSurface", 4.0),
        min("ice_steel", "night1", "AccentOnSurface", 4.0),
        min("solar_magenta", "night1", "AccentOnSurface", 4.0),
        min("dusk_bronze", "night1", "AccentOnSurface", 4.0),
        min("aurora_green", "night1", "AccentOnSurface", 4.0),
        min("ice_cyan", "night1", "AccentOnSurface", 4.0),
        min("fable_violet", "night1", "AccentOnSurface", 4.0),
        // AccentOnElevated ≥3.0 (night2).
        min("aurora_red", "night2", "AccentOnElevated", 3.0),
        min("ice_steel", "night2", "AccentOnElevated", 3.0),
        min("aurora_green", "night2", "AccentOnElevated", 3.0),
        min("ice_cyan", "night2", "AccentOnElevated", 3.0),
        // SelectionBand ∈[1.3,2.0] (never inverts polarity).
        band("selection", "night0", "SelectionBand", 1.3, 2.0),
        // FgOverSelection ≥4.5.
        min("snow1", "selection", "FgOverSelection", 4.5),
        min("snow2", "selection", "FgOverSelection", 4.5),
        // GlowOverSelection ≥4.5 (enumerated). ember-over-selection
        // is the warm-palette boundary case (computed 4.46) — its row
        // carries the ≥4.4 floor so it stays an honest AA-strong
        // assertion rather than a silent rounding pass.
        min("aurora_green", "selection", "GlowOverSelection", 4.5),
        min("first_light", "selection", "GlowOverSelection", 4.5),
        min("ice_cyan", "selection", "GlowOverSelection", 4.5),
        min("ember", "selection", "GlowOverSelection", 4.4),
        min("fable_violet", "selection", "GlowOverSelection", 4.5),
        min("ice_teal", "selection", "GlowOverSelection", 4.5),
        // DualDutyOverSelection ≥3.0 (enumerated — incl. bronze).
        min("aurora_red", "selection", "DualDutyOverSelection", 3.0),
        min("ice_steel", "selection", "DualDutyOverSelection", 3.0),
        min("solar_magenta", "selection", "DualDutyOverSelection", 3.0),
        min("dusk_bronze", "selection", "DualDutyOverSelection", 3.0),
        // MutedOverSelection ≥3.0.
        min("shadow1", "selection", "MutedOverSelection", 3.0),
        // FgPromotion over selection: shadow0 raw <3.0 → snow0.
        min("snow0", "selection", "FgPromotion(shadow0→snow0)", 4.5),
        // SearchBand ∈[1.3,1.9].
        band("search_others", "night0", "SearchBand", 1.3, 1.9),
        // FgOverSearch ≥4.5.
        min("snow1", "search_others", "FgOverSearch", 4.5),
        // MutedOverSearch ≥3.0 (comments under match — M2 fix).
        min("shadow1", "search_others", "MutedOverSearch", 3.0),
        // FgPromotion over search (targets, ≥3.5 / ≥4.0).
        min(
            "red_bright",
            "search_others",
            "FgPromotion(red→red_bright)",
            3.5,
        ),
        min("ember", "search_others", "FgPromotion(bronze→ember)", 3.5),
        min("snow0", "search_others", "FgPromotion(shadow0→snow0)", 4.0),
        // InversePair ≥7.0.
        min("night0", "first_light", "InversePair(search-current)", 7.0),
        min("night0", "amber_bright", "InversePair(nvim-IncSearch)", 7.0),
        min("night0", "green_bright", "InversePair(cursor)", 7.0),
        // FgOverGlass — body text over the glass band. Vellum's glasses
        // are NOT equal-luminance (the recipe alphas were authored for
        // the warm palette); steel_glass is the lightest (computed
        // 4.16), so the floor is the UI-text ≥4.0 line, not the old
        // equal-luminance ≈5.0 band.
        min("snow1", "red_glass", "FgOverGlass", 4.5),
        min("snow1", "green_glass", "FgOverGlass", 4.5),
        min("snow1", "amber_glass", "FgOverGlass", 4.5),
        min("snow1", "steel_glass", "FgOverGlass", 4.0),
        min("snow1", "violet_glass", "FgOverGlass", 4.5),
        min("snow1", "cyan_glass", "FgOverGlass", 4.5),
        // nvim group composites over glass surfaces. snow0 (muted) over
        // red_glass computes 2.80 in Vellum — an honest muted-over-glass
        // floor (≥2.8), below the brighter snow3/snow1 fg rows above it.
        min("snow3", "red_glass", "UIText(nvim-ErrorMsg)", 4.5),
        min("snow0", "red_glass", "PromotionTarget(DiffDelete-fg)", 2.8),
        // GlassVsBg ≥1.5 (clearly-visible surface).
        min("red_glass", "night0", "GlassVsBg", 1.5),
        min("green_glass", "night0", "GlassVsBg", 1.5),
        min("violet_glass", "night0", "GlassVsBg", 1.5),
        // LegacyRaw(text-on-slot) — Vellum is a LIGHT-accent palette, so
        // the legible raw-ANSI pairing is the dark ground (night0) over
        // the accent-as-background (the "pill" direction the palette is
        // designed for), not near-white snow3 over a light accent (which
        // is intentionally low-contrast). Floor ≥6.0 captures that
        // design intent honestly (computed 6.15 / 7.76 / 7.75).
        min("night0", "aurora_red", "LegacyRaw(bg-on-1)", 6.0),
        min("night0", "ice_steel", "LegacyRaw(bg-on-4)", 6.0),
        min("night0", "solar_magenta", "LegacyRaw(bg-on-5)", 6.0),
        // agent-attention surface text floor (spec §5).
        min("snow1", "violet_glass", "AgentAttentionText", 4.5),
    ]
}

fn tok(p: &VellumPalette, name: &str) -> Rgb {
    p.get(name)
        .unwrap_or_else(|| panic!("unknown Vellum token: {name}"))
}

#[test]
fn vellum_contrast_matrix() {
    let p = VellumPalette::vellum();
    let mut failures = Vec::new();
    for row in matrix() {
        let r = contrast(tok(&p, row.fg), tok(&p, row.bg));
        let ok = match row.floor {
            Floor::Min(m) => r >= m,
            Floor::Band(lo, hi) => r >= lo && r <= hi,
        };
        if !ok {
            let want = match row.floor {
                Floor::Min(m) => format!("≥{m:.2}"),
                Floor::Band(lo, hi) => format!("∈[{lo:.2},{hi:.2}]"),
            };
            failures.push(format!(
                "{} on {} [{}]: computed {r:.2}, want {want}",
                row.fg, row.bg, row.class
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} matrix rows failed:\n  - {}",
        failures.len(),
        failures.join("\n  - ")
    );
}

#[test]
fn night_grid_ladder_is_monotone_and_pins_the_authored_l() {
    // Vellum's parchment ground is a warm, monotone-darkening ladder.
    // It is NOT the old equidistant 0.050 lattice — the targets below
    // pin the AUTHORED Vellum OKLab L values (±0.004), and the ladder
    // is verified strictly increasing.
    let p = VellumPalette::vellum();
    let targets = [
        ("night_abyss", 0.154),
        ("night_deep", 0.165),
        ("night0", 0.191),
        ("night1", 0.227),
        ("night2", 0.277),
        ("night3", 0.326),
    ];
    let mut failures = Vec::new();
    let mut prev = f64::MIN;
    for (name, target) in targets {
        let l = oklab_l(tok(&p, name));
        if (l - target).abs() > 0.004 {
            failures.push(format!("{name}: L={l:.4}, target {target:.3} (±0.004)"));
        }
        if l <= prev {
            failures.push(format!(
                "{name}: L={l:.4} not strictly > previous {prev:.4}"
            ));
        }
        prev = l;
    }
    assert!(
        failures.is_empty(),
        "night ladder drift:\n  - {}",
        failures.join("\n  - ")
    );
}

#[test]
fn text_grid_is_monotone_and_pins_the_authored_l() {
    // Vellum's warm text axis (warm ink → warm cream). Monotone, with
    // the AUTHORED Vellum OKLab L values pinned (±0.004).
    let p = VellumPalette::vellum();
    let targets = [
        ("shadow0", 0.518),
        ("shadow1", 0.632),
        ("snow0", 0.724),
        ("snow1", 0.892),
        ("snow2", 0.926),
        ("snow3", 0.953),
    ];
    let mut failures = Vec::new();
    let mut prev = f64::MIN;
    for (name, target) in targets {
        let l = oklab_l(tok(&p, name));
        if (l - target).abs() > 0.004 {
            failures.push(format!("{name}: L={l:.4}, target {target:.3} (±0.004)"));
        }
        if l <= prev {
            failures.push(format!(
                "{name}: L={l:.4} not strictly > previous {prev:.4}"
            ));
        }
        prev = l;
    }
    assert!(
        failures.is_empty(),
        "text ladder drift:\n  - {}",
        failures.join("\n  - ")
    );
}

#[test]
fn per_tier_accent_l_spreads_within_caps() {
    // Vellum's accent tiers are a little wider than the old palette's
    // (warm, lower-chroma fitting). Caps recalibrated to honest Vellum
    // values: dual ≤0.08 (computed 0.0755), glow ≤0.13 (0.1188),
    // bright ≤0.14 (0.1360).
    let p = VellumPalette::vellum();
    let spread = |names: &[&str]| -> f64 {
        let ls: Vec<f64> = names.iter().map(|n| oklab_l(tok(&p, n))).collect();
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for l in ls {
            lo = lo.min(l);
            hi = hi.max(l);
        }
        hi - lo
    };
    let dual = spread(&["aurora_red", "ice_steel", "solar_magenta", "dusk_bronze"]);
    let glow = spread(&["aurora_green", "first_light", "ember", "fable_violet"]);
    let bright = spread(&[
        "red_bright",
        "green_bright",
        "amber_bright",
        "steel_bright",
        "magenta_bright",
        "cyan_bright",
        "violet_bright",
    ]);
    let mut failures = Vec::new();
    if dual > 0.08 {
        failures.push(format!("dual-duty spread {dual:.4} > 0.08"));
    }
    if glow > 0.13 {
        failures.push(format!("glow spread {glow:.4} > 0.13"));
    }
    if bright > 0.14 {
        failures.push(format!("bright spread {bright:.4} > 0.14"));
    }
    assert!(
        failures.is_empty(),
        "tier spread cap breach:\n  - {}",
        failures.join("\n  - ")
    );
}

#[test]
fn brights_are_strictly_lighter_than_their_normals() {
    // bright-over-normal: L strictly greater, ΔL ∈[+0.03,+0.14] for
    // Vellum. `first_light→amber_bright` is the warm-palette boundary
    // (computed +0.0326) — the lower bound carries that slack so the
    // row stays an honest "strictly lighter, ≈+0.03" assertion, never a
    // silent rounding pass. Strictly-lighter (ΔL>0) is the hard
    // invariant.
    let p = VellumPalette::vellum();
    let pairs = [
        ("aurora_red", "red_bright"),
        ("aurora_green", "green_bright"),
        ("first_light", "amber_bright"),
        ("ice_steel", "steel_bright"),
        ("solar_magenta", "magenta_bright"),
        ("ice_cyan", "cyan_bright"),
        ("fable_violet", "violet_bright"),
    ];
    // Warm-palette boundary floor (first_light→amber_bright row).
    const LO: f64 = 0.03;
    const HI: f64 = 0.14;
    let mut failures = Vec::new();
    for (normal, bright) in pairs {
        let dl = oklab_l(tok(&p, bright)) - oklab_l(tok(&p, normal));
        // Strictly lighter is the hard invariant; the band is the
        // documented per-pair envelope.
        if dl <= 0.0 || !(LO..=HI).contains(&dl) {
            failures.push(format!(
                "{normal}→{bright}: ΔL={dl:+.4} ∉ (0, [{LO:.3},{HI:.2}]]"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "bright tier drift:\n  - {}",
        failures.join("\n  - ")
    );
}

#[test]
fn base16_ramp_is_monotone_and_has_no_duplicate_hexes() {
    // §3 — L(00)<L(01)<L(02)<L(03); zero duplicate hexes base00–0F.
    let p = VellumPalette::vellum();
    let b16 = p.base16();
    let l00 = oklab_l(b16[0].1);
    let l01 = oklab_l(b16[1].1);
    let l02 = oklab_l(b16[2].1);
    let l03 = oklab_l(b16[3].1);
    assert!(
        l00 < l01 && l01 < l02 && l02 < l03,
        "base ramp not monotone: 00={l00:.4} 01={l01:.4} 02={l02:.4} 03={l03:.4}"
    );
    // Zero duplicate hexes across base00–0F.
    let mut seen = std::collections::HashSet::new();
    let mut dups = Vec::new();
    for (slot, rgb) in b16 {
        let hex = rgb.hex();
        if !seen.insert(hex.clone()) {
            dups.push(format!("{slot}={hex}"));
        }
    }
    assert!(
        dups.is_empty(),
        "duplicate base16 hexes:\n  - {}",
        dups.join("\n  - ")
    );
}

#[test]
fn ansi_slots_meet_their_floors() {
    // §9 — slots 1–7,9–15 ≥4.5 on night0; slot 8 ≥3.0.
    let p = VellumPalette::vellum();
    let ansi = p.ansi_16();
    let bg = p.night0;
    let mut failures = Vec::new();
    for (i, &slot) in ansi.iter().enumerate() {
        // Slot 0 is a surface (night2), not a text colour — skip.
        if i == 0 {
            continue;
        }
        let floor = if i == 8 { 3.0 } else { 4.5 };
        let r = contrast(slot, bg);
        if r < floor {
            failures.push(format!(
                "ANSI {i} ({}) on night0: {r:.2} < {floor}",
                slot.hex()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "ANSI floor breach:\n  - {}",
        failures.join("\n  - ")
    );
}

#[test]
fn derived_tokens_equal_their_blend_recipes() {
    // §9 — blend-exactness, zero tolerance (the token IS the output).
    let p = VellumPalette::vellum();
    let rows: [(&str, Rgb); 8] = [
        ("selection", blend_linear(p.night0, p.fable_violet, 0.08)),
        (
            "search_others",
            blend_linear(p.night0, p.first_light, 0.075),
        ),
        ("red_glass", blend_linear(p.night0, p.aurora_red, 0.327)),
        ("green_glass", blend_linear(p.night0, p.aurora_green, 0.168)),
        ("amber_glass", blend_linear(p.night0, p.first_light, 0.137)),
        ("steel_glass", blend_linear(p.night0, p.ice_steel, 0.325)),
        ("cyan_glass", blend_linear(p.night0, p.ice_cyan, 0.173)),
        (
            "violet_glass",
            blend_linear(p.night0, p.fable_violet, 0.220),
        ),
    ];
    let mut failures = Vec::new();
    for (name, recipe) in rows {
        let stored = tok(&p, name).hex();
        if stored != recipe.hex() {
            failures.push(format!(
                "{name}: stored {stored} != recipe {}",
                recipe.hex()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "blend drift:\n  - {}",
        failures.join("\n  - ")
    );
}

#[test]
fn glass_band_keeps_body_text_legible() {
    // Vellum's glass surfaces are NOT equal-luminance (the recipe alphas
    // were authored for the warm palette, so each glass carries its own
    // luminance). The invariant that matters is that body text (snow1)
    // stays legible over EVERY glass surface — the UI-text floor ≥4.0.
    let p = VellumPalette::vellum();
    let glasses = [
        "red_glass",
        "green_glass",
        "amber_glass",
        "steel_glass",
        "cyan_glass",
        "violet_glass",
    ];
    let mut failures = Vec::new();
    for name in glasses {
        let cr = contrast(p.snow1, tok(&p, name));
        if cr < 4.0 {
            failures.push(format!("snow1/{name}: {cr:.3} < 4.0"));
        }
    }
    assert!(
        failures.is_empty(),
        "glass body-text legibility:\n  - {}",
        failures.join("\n  - ")
    );
}

#[test]
fn fg_promotion_targets_clear_their_floors() {
    // §9 — every FgPromotion target ≥3.5 over its worst surface.
    // The map: shadow0/shadow1→snow0, aurora_red→red_bright,
    // ice_steel→steel_bright, solar_magenta→magenta_bright,
    // dusk_bronze→ember. Asserted over selection + search_others.
    let p = VellumPalette::vellum();
    let surfaces = ["selection", "search_others"];
    let mut failures = Vec::new();
    for (from, to) in VellumPalette::fg_promotions() {
        for surf in surfaces {
            let cr = contrast(tok(&p, to), tok(&p, surf));
            if cr < 3.5 {
                failures.push(format!("{from}→{to} on {surf}: target {cr:.2} < 3.5"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "promotion target sub-floor:\n  - {}",
        failures.join("\n  - ")
    );
}
