//! Material Design 3 system-color renderer — emits the 34 `--md-sys-color-*`
//! CSS custom properties Material consumers (e.g. `pleme-mui`) expect, every one
//! MAPPED from the ishou [`TokenSet`]. This is the render target that lets the
//! Material component library consume ishou instead of forking its own
//! hard-coded MD3 palette — the move that collapses the design-system fork
//! (`ishou` ↔ `pleme-mui`/`irodori`) the fleet audit named.
//!
//! ishou's token sets are dark-first (Nord / Vellum), so the mapping is a dark
//! MD3 scheme: MD3's tonal roles are bound to ishou's semantic roles + the
//! 4-step `polar_night` surface ladder. Render this target from a different
//! `TokenSet` (a light or brand theme) to get that theme's MD3 surface — the
//! *mapping* is one place, the *theme* is the input. The `on-*` (contrast) roles
//! bind to the opposing end of the scale (dark `background` on the light
//! accents; light `text` on the dark surfaces).

use ishou_tokens::{Rgb, TokenSet};

/// Resolve a semantic role name (kebab, as in [`SemanticRoles::pairs`]) to hex
/// via the TokenSet's role→palette binding. Every pleme/vellum role is always
/// bound, so the `background` fallback is unreachable in practice.
fn role(t: &TokenSet, name: &str) -> String {
    let key = t
        .roles
        .pairs()
        .into_iter()
        .find(|(r, _)| *r == name)
        .map(|(_, k)| k)
        .unwrap_or("background");
    palette(t, key)
}

/// Resolve a palette key (snake_case, e.g. `polar_night_0`, `ink`) to hex.
fn palette(t: &TokenSet, key: &str) -> String {
    t.color
        .get(key)
        .map(|c: Rgb| c.hex())
        .unwrap_or_else(|| "#000000".to_string())
}

/// Render the MD3 system-color sheet from a token set.
#[must_use]
pub fn render(t: &TokenSet) -> String {
    // (md-sys-color suffix, resolved hex from ishou). The mapping is the only
    // new design decision; the values all come from the TokenSet.
    let pairs: [(&str, String); 34] = [
        ("primary", role(t, "primary")),
        ("on-primary", role(t, "background")),
        ("primary-container", role(t, "structural")),
        ("on-primary-container", role(t, "text")),
        ("secondary", role(t, "structural")),
        ("on-secondary", role(t, "background")),
        ("secondary-container", role(t, "surface-elevated")),
        ("on-secondary-container", role(t, "text")),
        ("tertiary", role(t, "accent")),
        ("on-tertiary", role(t, "background")),
        ("tertiary-container", role(t, "surface-elevated")),
        ("on-tertiary-container", role(t, "text")),
        ("error", role(t, "error")),
        ("on-error", role(t, "background")),
        ("error-container", role(t, "surface-elevated")),
        ("on-error-container", role(t, "text")),
        ("background", role(t, "background")),
        ("on-background", role(t, "text")),
        ("surface", role(t, "surface")),
        ("on-surface", role(t, "text")),
        ("surface-variant", role(t, "surface-elevated")),
        ("on-surface-variant", role(t, "text-muted")),
        ("outline", role(t, "text-dim")),
        ("outline-variant", role(t, "surface-elevated")),
        ("inverse-surface", role(t, "text")),
        ("inverse-on-surface", role(t, "background")),
        ("inverse-primary", role(t, "structural")),
        ("surface-dim", role(t, "background")),
        ("surface-bright", role(t, "text-dim")),
        ("surface-container-lowest", palette(t, "ink")),
        ("surface-container-low", role(t, "background")),
        ("surface-container", role(t, "surface")),
        ("surface-container-high", role(t, "surface-elevated")),
        ("surface-container-highest", role(t, "text-dim")),
    ];

    let mut out =
        String::from("/* ishou — Material Design 3 system colors (generated; do not edit) */\n\n");
    out.push_str(":root {\n");
    for (suffix, hex) in &pairs {
        out.push_str(&format!("  --md-sys-color-{suffix}: {hex};\n"));
    }
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact `--md-sys-color-*` set `pleme-mui::theme::Md3Tokens` consumes.
    const MD3_SYSTEM_COLORS: [&str; 34] = [
        "primary", "on-primary", "primary-container", "on-primary-container",
        "secondary", "on-secondary", "secondary-container", "on-secondary-container",
        "tertiary", "on-tertiary", "tertiary-container", "on-tertiary-container",
        "error", "on-error", "error-container", "on-error-container",
        "background", "on-background", "surface", "on-surface",
        "surface-variant", "on-surface-variant", "outline", "outline-variant",
        "inverse-surface", "inverse-on-surface", "inverse-primary",
        "surface-dim", "surface-bright", "surface-container-lowest",
        "surface-container-low", "surface-container", "surface-container-high",
        "surface-container-highest",
    ];

    #[test]
    fn emits_every_md3_system_color_pleme_mui_consumes() {
        let out = render(&TokenSet::pleme());
        for prop in MD3_SYSTEM_COLORS {
            let needle = format!("--md-sys-color-{prop}:");
            assert!(out.contains(&needle), "missing MD3 color: {prop}");
        }
    }

    #[test]
    fn every_role_resolves_to_a_real_token_hex() {
        let out = render(&TokenSet::pleme());
        // The "#000000" sentinel only appears if a role failed to resolve.
        assert!(
            !out.contains("#000000"),
            "an MD3 role failed to resolve from ishou tokens"
        );
        // Exactly 34 resolved hex values (one per system color), no more.
        assert_eq!(out.matches('#').count(), 34, "expected 34 resolved hex values");
    }

    #[test]
    fn is_deterministic() {
        assert_eq!(render(&TokenSet::pleme()), render(&TokenSet::pleme()));
    }

    #[test]
    fn steel_theme_renders_the_metallic_surface() {
        let out = render(&TokenSet::steel());
        // primary ← frost_1 = the blued-steel #5E8CC4
        assert!(
            out.contains("--md-sys-color-primary: #5E8CC4"),
            "steel primary should be blued-steel"
        );
        // background ← polar_night_0 = the machined near-black
        assert!(out.contains("--md-sys-color-background: #0B0E12"));
        // still fully resolved (no sentinel)
        assert!(!out.contains("#000000"), "steel md3 should fully resolve");
    }
}
