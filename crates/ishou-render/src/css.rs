//! CSS renderer — emits `:root { ... }` custom properties + a small set of
//! opinionated utility classes (brand typography, shadow modifiers, swerve
//! backdrop). The utility layer is narrow on purpose — consumers use Tailwind
//! for general utility, ishou owns the brand-critical pieces.

use ishou_tokens::{ColorPalette, Rgb, Shadow, ShadowSpec, TokenSet};

pub fn render(t: &TokenSet) -> String {
    let mut out = String::from("/* ishou — pleme-io design tokens (generated; do not edit) */\n\n");
    out.push_str(":root {\n");
    palette_vars(&mut out, &t.color);
    role_vars(&mut out, t);
    typography_vars(&mut out, t);
    spacing_vars(&mut out, t);
    radius_vars(&mut out, t);
    shadow_vars(&mut out, &t.color, &t.shadow);
    motion_vars(&mut out, t);
    out.push_str("}\n\n");
    utility_classes(&mut out);
    out
}

fn palette_vars(out: &mut String, c: &ColorPalette) {
    out.push_str("  /* palette */\n");
    for (k, v) in c.entries() {
        out.push_str(&format!(
            "  --ishou-{}: {};\n",
            k.replace('_', "-"),
            v.hex()
        ));
    }
}

fn role_vars(out: &mut String, t: &TokenSet) {
    out.push_str("  /* semantic roles */\n");
    for (role, palette_key) in t.roles.pairs() {
        out.push_str(&format!(
            "  --ishou-color-{role}: var(--ishou-{});\n",
            palette_key.replace('_', "-")
        ));
    }
}

fn typography_vars(out: &mut String, t: &TokenSet) {
    let f = &t.typography.families;
    out.push_str("  /* typography */\n");
    out.push_str(&format!("  --ishou-font-serif: {};\n", f.serif));
    out.push_str(&format!("  --ishou-font-sans: {};\n", f.sans));
    out.push_str(&format!("  --ishou-font-mono: {};\n", f.mono));
    out.push_str(&format!("  --ishou-font-display: {};\n", f.display));
    let s = &t.typography.scale;
    for (k, v) in [
        ("xs", s.xs),
        ("sm", s.sm),
        ("base", s.base),
        ("md", s.md),
        ("lg", s.lg),
        ("xl", s.xl),
        ("2xl", s.x2),
        ("3xl", s.x3),
        ("4xl", s.x4),
    ] {
        out.push_str(&format!("  --ishou-text-{k}: {v}rem;\n"));
    }
    let w = &t.typography.weight;
    for (k, v) in [
        ("light", w.light),
        ("regular", w.regular),
        ("medium", w.medium),
        ("semibold", w.semibold),
        ("bold", w.bold),
    ] {
        out.push_str(&format!("  --ishou-weight-{k}: {v};\n"));
    }
    let lh = &t.typography.line_height;
    for (k, v) in [
        ("tight", lh.tight),
        ("base", lh.base),
        ("relaxed", lh.relaxed),
        ("prose", lh.prose),
    ] {
        out.push_str(&format!("  --ishou-leading-{k}: {v};\n"));
    }
}

fn spacing_vars(out: &mut String, t: &TokenSet) {
    out.push_str("  /* spacing (px) */\n");
    for (k, v) in t.spacing.pairs() {
        out.push_str(&format!("  --ishou-space-{k}: {v}px;\n"));
    }
}

fn radius_vars(out: &mut String, t: &TokenSet) {
    out.push_str("  /* radius */\n");
    for (k, v) in t.radius.pairs() {
        let suffix = if v >= 9999 {
            "9999px".to_string()
        } else {
            format!("{v}px")
        };
        out.push_str(&format!("  --ishou-radius-{k}: {suffix};\n"));
    }
}

fn shadow_vars(out: &mut String, palette: &ColorPalette, shadow: &Shadow) {
    out.push_str("  /* shadow (elevation) */\n");
    for (k, spec) in shadow.pairs() {
        out.push_str(&format!(
            "  --ishou-shadow-{k}: {};\n",
            shadow_css(spec, palette.shadow_tone)
        ));
    }
}

fn shadow_css(spec: &ShadowSpec, tone: Rgb) -> String {
    if spec.alpha_pct == 0 {
        return "none".into();
    }
    let alpha = f32::from(spec.alpha_pct) / 100.0;
    format!(
        "{}px {}px {}px {}px rgba({}, {}, {}, {:.2})",
        spec.offset_x, spec.offset_y, spec.blur, spec.spread, tone.r, tone.g, tone.b, alpha
    )
}

fn motion_vars(out: &mut String, t: &TokenSet) {
    out.push_str("  /* motion */\n");
    let d = &t.motion.duration;
    for (k, v) in [
        ("instant", d.instant_ms),
        ("fast", d.fast_ms),
        ("base", d.base_ms),
        ("slow", d.slow_ms),
        ("hero", d.hero_ms),
    ] {
        out.push_str(&format!("  --ishou-duration-{k}: {v}ms;\n"));
    }
    let e = &t.motion.easing;
    for (k, c) in [
        ("standard", e.standard),
        ("decelerate", e.decelerate),
        ("accelerate", e.accelerate),
        ("sonic-boom", e.sonic_boom),
        ("saber", e.saber),
    ] {
        out.push_str(&format!(
            "  --ishou-ease-{k}: cubic-bezier({:.3}, {:.3}, {:.3}, {:.3});\n",
            c.0, c.1, c.2, c.3
        ));
    }
}

fn utility_classes(out: &mut String) {
    out.push_str(
        r#"/* Minimal utility layer — brand-critical only. Use Tailwind for everything else. */

.ishou-shadow-brand {
  box-shadow: var(--ishou-shadow-brand-bold);
}

.ishou-serif { font-family: var(--ishou-font-serif); }
.ishou-sans { font-family: var(--ishou-font-sans); }
.ishou-mono { font-family: var(--ishou-font-mono); }

.ishou-surface {
  background: var(--ishou-color-surface);
  color: var(--ishou-color-text);
  border-radius: var(--ishou-radius-md);
}

.ishou-ink { color: var(--ishou-color-ink); background: var(--ishou-color-paper); }
.ishou-paper { color: var(--ishou-color-paper); background: var(--ishou-color-ink); }
"#,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ishou_tokens::TokenSet;

    #[test]
    fn render_is_deterministic() {
        let a = render(&TokenSet::pleme());
        let b = render(&TokenSet::pleme());
        assert_eq!(a, b);
    }

    #[test]
    fn render_contains_every_role() {
        let t = TokenSet::pleme();
        let out = render(&t);
        for (role, _) in t.roles.pairs() {
            let prop = format!("--ishou-color-{role}:");
            assert!(out.contains(&prop), "missing role: {role}");
        }
    }
}
