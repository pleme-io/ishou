//! SCSS renderer — emits `$ishou-...` variables. Mirror of CSS.

use ishou_tokens::TokenSet;

pub fn render(t: &TokenSet) -> String {
    let mut out = String::from("// ishou — pleme-io design tokens (generated)\n\n");
    for (k, v) in t.color.entries() {
        out.push_str(&format!("$ishou-{}: {};\n", k.replace('_', "-"), v.hex()));
    }
    for (role, palette_key) in t.roles.pairs() {
        let rgb = t.color.get(palette_key).expect("role resolves");
        out.push_str(&format!("$ishou-color-{role}: {};\n", rgb.hex()));
    }
    for (k, v) in t.spacing.pairs() {
        out.push_str(&format!("$ishou-space-{k}: {v}px;\n"));
    }
    for (k, v) in t.radius.pairs() {
        out.push_str(&format!("$ishou-radius-{k}: {v}px;\n"));
    }
    out
}
