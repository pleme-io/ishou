//! TUI renderer — emits Rust code binding ishou colors to ratatui /
//! crossterm `Color::Rgb`. Designed to be pasted into a TUI app's `theme.rs`
//! or written to a file and included via `include!`.

use ishou_tokens::TokenSet;

pub fn render(t: &TokenSet) -> String {
    let mut out = String::from(
        "// ishou — pleme-io TUI color bindings (generated)\n\
         // Compatible with ratatui::style::Color and crossterm::style::Color.\n\n\
         pub mod ishou_tui {\n",
    );
    for (k, v) in t.color.entries() {
        out.push_str(&format!(
            "    /// {name}\n    pub const {ident}: (u8, u8, u8) = ({}, {}, {});\n",
            v.r,
            v.g,
            v.b,
            name = k.replace('_', "-"),
            ident = k.to_uppercase(),
        ));
    }
    out.push_str("\n    /// ratatui integration: `ratatui::style::Color::Rgb(r, g, b)` from a tuple.\n");
    out.push_str(
        "    #[cfg(feature = \"ratatui\")]\n    \
         pub fn r(rgb: (u8, u8, u8)) -> ratatui::style::Color {\n        \
         ratatui::style::Color::Rgb(rgb.0, rgb.1, rgb.2)\n    }\n",
    );
    out.push_str("\n    /// crossterm integration.\n");
    out.push_str(
        "    #[cfg(feature = \"crossterm\")]\n    \
         pub fn c(rgb: (u8, u8, u8)) -> crossterm::style::Color {\n        \
         crossterm::style::Color::Rgb { r: rgb.0, g: rgb.1, b: rgb.2 }\n    }\n",
    );
    out.push_str("}\n");
    out
}
