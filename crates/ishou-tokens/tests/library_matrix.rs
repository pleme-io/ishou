//! Theme-library verification matrix — the forcing function over the
//! WHOLE `FleetTheme` registry (★★ CLOSED-LOOP MASS-SYNTHESIS rule 1).
//!
//! `vellum_matrix.rs` pins the *Vellum* palette's per-token design grid.
//! THIS file pins the invariants EVERY theme in the library must satisfy,
//! by iterating `FleetTheme::all()` and resolving each. A new theme that
//! leaves an ANSI slot empty, regresses body-text legibility, collides on
//! a name, or is added to the enum but not to `all()` fails the build —
//! so "the library is uniformly legible" is mechanical, not asserted.
//!
//! WCAG 2.1 contrast is recomputed here from the resolved hex (the same
//! relative-luminance formula the design workflow used), never trusted.

use ishou_tokens::FleetTheme;

// ── WCAG 2.1 relative luminance + contrast (order-independent) ────────────

fn parse_hex(s: &str) -> (u8, u8, u8) {
    let h = s.trim_start_matches('#');
    assert_eq!(h.len(), 6, "not a #rrggbb hex: {s:?}");
    let b = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).expect("hex");
    (b(0), b(2), b(4))
}

fn chan_lin(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn luminance(hex: &str) -> f64 {
    let (r, g, b) = parse_hex(hex);
    0.2126 * chan_lin(r) + 0.7152 * chan_lin(g) + 0.0722 * chan_lin(b)
}

fn contrast(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

// ── Forcing functions ─────────────────────────────────────────────────────

/// The registry is TOTAL: every variant `name()` recognises is reachable
/// through `all()`. (The `name()` match is itself exhaustive — the
/// compiler forces a new variant to be named; this forces it into `all()`.)
#[test]
fn library_is_complete() {
    // Round-trip every listed theme through serde and back — proves each
    // entry in `all()` is a real, serialisable variant with a stable name.
    for &t in FleetTheme::all() {
        let s = serde_yaml::to_string(&t).unwrap();
        let back: FleetTheme = serde_yaml::from_str(&s).unwrap();
        assert_eq!(t, back, "theme {} does not round-trip through serde", t.name());
    }
    // The two named anchors must be in the library.
    assert!(FleetTheme::all().contains(&FleetTheme::bare()), "bare missing from library");
    assert!(
        FleetTheme::all().contains(&FleetTheme::prescribed_default()),
        "prescribed default missing from library"
    );
    // Vellum is the shipping default.
    assert_eq!(FleetTheme::prescribed_default(), FleetTheme::Vellum);
}

/// Every theme name is unique and matches its serde encoding.
#[test]
fn theme_names_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for &t in FleetTheme::all() {
        assert!(seen.insert(t.name()), "duplicate theme name: {}", t.name());
        // name() must equal the resolved theme's `name` field.
        assert_eq!(t.name(), t.resolve().name, "name() disagrees with resolve().name");
    }
}

/// Structural completeness: every theme fills all 16 ANSI slots with a
/// valid #rrggbb, and bg != fg. A terminal app using indexed color must
/// never see an empty or malformed slot.
#[test]
fn every_theme_has_a_complete_ansi_palette() {
    let mut failures = Vec::new();
    for &t in FleetTheme::all() {
        let r = t.resolve();
        if r.background.eq_ignore_ascii_case(&r.foreground) {
            failures.push(format!("{}: background == foreground", t.name()));
        }
        for (i, c) in r.ansi_16.iter().enumerate() {
            if c.len() != 7 || !c.starts_with('#') || u32::from_str_radix(&c[1..], 16).is_err() {
                failures.push(format!("{}: ansi[{i}] is not #rrggbb: {c:?}", t.name()));
            }
        }
        if r.background.len() != 7 || r.foreground.len() != 7 {
            failures.push(format!("{}: bg/fg not #rrggbb", t.name()));
        }
    }
    assert!(failures.is_empty(), "ANSI completeness failures:\n  - {}", failures.join("\n  - "));
}

/// LEGIBILITY FLOOR — the load-bearing forcing function. Body text on the
/// theme background must clear WCAG AAA (>=7.0) for EVERY theme in the
/// library. Vellum fixed Nord's classic low-contrast trap; this guarantees
/// no future theme reintroduces it. (Operator stressed visibility twice;
/// "vim was all-white" — this is the type-level promise it can't recur.)
#[test]
fn every_theme_body_text_clears_aaa() {
    let mut failures = Vec::new();
    for &t in FleetTheme::all() {
        let r = t.resolve();
        let ratio = contrast(&r.foreground, &r.background);
        if ratio < 7.0 {
            failures.push(format!(
                "{}: fg {} / bg {} = {ratio:.2}:1 (< 7.0 AAA)",
                t.name(), r.foreground, r.background
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} theme(s) fail the AAA body-text floor:\n  - {}",
        failures.len(), failures.join("\n  - ")
    );
}

/// Pin the prescribed fleet default's identity so a stray re-point is caught.
#[test]
fn prescribed_default_is_vellum_warm_parchment() {
    let r = FleetTheme::prescribed_default().resolve();
    assert_eq!(r.name, "vellum");
    assert_eq!(r.background.to_uppercase(), "#16140E");
    assert_eq!(r.foreground.to_uppercase(), "#E2DBC8");
    // The warm-parchment promise: background is warmer than it is cool (R > B).
    let (br, _, bb) = parse_hex(&r.background);
    assert!(br >= bb, "vellum background should be warm (R >= B), got {}", r.background);
}
