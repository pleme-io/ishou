//! Rust renderer — emits a `pub const` module for consumers that want
//! compile-time colors (e.g., TUI apps not wanting to deserialize JSON).

use ishou_tokens::TokenSet;

pub fn render(t: &TokenSet) -> String {
    let mut out = String::from(
        "// ishou — pleme-io design tokens (generated)\n\
         // Use ishou-tokens::TokenSet for runtime; this file is for\n\
         // consumers that want const-propagated values.\n\n\
         #[allow(dead_code)]\n\
         pub mod color {\n    #[derive(Clone, Copy, Debug)]\n    pub struct Rgb(pub u8, pub u8, pub u8);\n\n",
    );
    for (k, v) in t.color.entries() {
        out.push_str(&format!(
            "    pub const {}: Rgb = Rgb({}, {}, {});\n",
            k.to_uppercase(),
            v.r,
            v.g,
            v.b
        ));
    }
    out.push_str("}\n\npub mod spacing {\n");
    for (k, v) in t.spacing.pairs() {
        out.push_str(&format!("    pub const SPACE_{}: u16 = {v};\n", k));
    }
    out.push_str("}\n\npub mod radius {\n");
    for (k, v) in t.radius.pairs() {
        out.push_str(&format!(
            "    pub const RADIUS_{}: u16 = {v};\n",
            k.to_uppercase()
        ));
    }
    out.push_str("}\n");
    out
}
