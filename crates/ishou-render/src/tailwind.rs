//! Tailwind renderer — emits a complete `tailwind.config.js` with colors,
//! fonts, spacing, radius, and shadow extensions keyed off ishou tokens.

use ishou_tokens::{ShadowSpec, TokenSet};

pub fn render(t: &TokenSet) -> String {
    let mut out = String::from(
        "/** @type {import('tailwindcss').Config} */\n\
         /* ishou — pleme-io Tailwind config (generated; do not edit) */\n\
         module.exports = {\n  \
         content: [\"./src/**/*.{rs,html,js,ts,tsx,jsx}\", \"./index.html\"],\n  \
         darkMode: \"class\",\n  \
         theme: {\n    extend: {\n",
    );

    // colors
    out.push_str("      colors: {\n");
    out.push_str("        ishou: {\n");
    for (k, v) in t.color.entries() {
        out.push_str(&format!(
            "          \"{}\": \"{}\",\n",
            k.replace('_', "-"),
            v.hex()
        ));
    }
    out.push_str("        },\n");
    // role aliases
    for (role, palette_key) in t.roles.pairs() {
        let rgb = t.color.get(palette_key).expect("role resolves");
        out.push_str(&format!("        \"{role}\": \"{}\",\n", rgb.hex()));
    }
    out.push_str("      },\n");

    // fontFamily
    out.push_str("      fontFamily: {\n");
    let f = &t.typography.families;
    out.push_str(&format!("        serif: [{}],\n", js_stack(f.serif)));
    out.push_str(&format!("        sans: [{}],\n", js_stack(f.sans)));
    out.push_str(&format!("        mono: [{}],\n", js_stack(f.mono)));
    out.push_str(&format!("        display: [{}],\n", js_stack(f.display)));
    out.push_str("      },\n");

    // fontSize
    out.push_str("      fontSize: {\n");
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
        out.push_str(&format!("        \"{k}\": \"{v}rem\",\n"));
    }
    out.push_str("      },\n");

    // spacing
    out.push_str("      spacing: {\n");
    for (k, v) in t.spacing.pairs() {
        out.push_str(&format!("        \"{k}\": \"{v}px\",\n"));
    }
    out.push_str("      },\n");

    // borderRadius
    out.push_str("      borderRadius: {\n");
    for (k, v) in t.radius.pairs() {
        out.push_str(&format!("        \"{k}\": \"{v}px\",\n"));
    }
    out.push_str("      },\n");

    // boxShadow
    out.push_str("      boxShadow: {\n");
    let tone = t.color.shadow_tone;
    for (k, spec) in t.shadow.pairs() {
        out.push_str(&format!(
            "        \"{k}\": \"{}\",\n",
            tailwind_shadow(spec, tone)
        ));
    }
    out.push_str("      },\n");

    // transitionTimingFunction
    out.push_str("      transitionTimingFunction: {\n");
    let e = &t.motion.easing;
    for (k, c) in [
        ("standard", e.standard),
        ("decelerate", e.decelerate),
        ("accelerate", e.accelerate),
        ("sonic-boom", e.sonic_boom),
        ("saber", e.saber),
    ] {
        out.push_str(&format!(
            "        \"{k}\": \"cubic-bezier({}, {}, {}, {})\",\n",
            c.0, c.1, c.2, c.3
        ));
    }
    out.push_str("      },\n");

    // transitionDuration
    out.push_str("      transitionDuration: {\n");
    let d = &t.motion.duration;
    for (k, v) in [
        ("instant", d.instant_ms),
        ("fast", d.fast_ms),
        ("base", d.base_ms),
        ("slow", d.slow_ms),
        ("hero", d.hero_ms),
    ] {
        out.push_str(&format!("        \"{k}\": \"{v}ms\",\n"));
    }
    out.push_str("      },\n");

    out.push_str("    },\n  },\n  plugins: [],\n};\n");
    out
}

fn js_stack(stack: &str) -> String {
    // stack is "'Foo', 'Bar', sans-serif" → `"Foo", "Bar", "sans-serif"`
    stack
        .split(',')
        .map(|p| {
            let p = p.trim().trim_matches('\'').trim_matches('"');
            format!("\"{p}\"")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn tailwind_shadow(spec: &ShadowSpec, tone: ishou_tokens::Rgb) -> String {
    if spec.alpha_pct == 0 {
        return "none".into();
    }
    let alpha = f32::from(spec.alpha_pct) / 100.0;
    format!(
        "{}px {}px {}px {}px rgba({}, {}, {}, {:.2})",
        spec.offset_x, spec.offset_y, spec.blur, spec.spread, tone.r, tone.g, tone.b, alpha
    )
}
