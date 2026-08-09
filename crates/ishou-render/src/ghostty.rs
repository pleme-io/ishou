//! Ghostty renderer — emits a Ghostty `config` block binding palette indices
//! and cursor / selection / background colors.

use ishou_tokens::TokenSet;

pub fn render(t: &TokenSet) -> String {
    let c = &t.color;
    let hex = |rgb: ishou_tokens::Rgb| rgb.hex();
    let mut out = String::from("# ishou — pleme-io Ghostty config (generated)\n\n");

    // Nord palette as Ghostty 16-color ANSI mapping.
    out.push_str(&format!("background = {}\n", hex(c.polar_night_0)));
    out.push_str(&format!("foreground = {}\n", hex(c.snow_storm_2)));
    out.push_str(&format!("cursor-color = {}\n", hex(c.frost_1)));
    out.push_str(&format!(
        "selection-background = {}\n",
        hex(c.polar_night_2)
    ));
    out.push_str(&format!("selection-foreground = {}\n", hex(c.snow_storm_2)));

    // 0-15 ANSI
    let palette = [
        c.polar_night_0,
        c.aurora_red,
        c.aurora_green,
        c.aurora_yellow,
        c.frost_3,
        c.aurora_purple,
        c.frost_0,
        c.snow_storm_0,
        c.polar_night_3,
        c.aurora_red,
        c.aurora_green,
        c.aurora_yellow,
        c.frost_2,
        c.aurora_purple,
        c.frost_1,
        c.snow_storm_2,
    ];
    for (i, p) in palette.iter().enumerate() {
        out.push_str(&format!("palette = {i}={}\n", hex(*p)));
    }

    // Shader stack — reference the 13 blackmatter-ghostty shaders.
    let shaders = [
        "stardust",
        "sonic-boom",
        "prompt-saber",
        "cursor-glow",
        "cursor-trail",
        "bloom",
        "frost-haze",
        "spotlight",
        "background-pulse",
        "chromatic-aberration",
        "film-grain",
        "screen-curvature",
    ];
    for s in shaders {
        out.push_str(&format!(
            "custom-shader = ${{blackmatter-ghostty}}/shaders/{s}.glsl\n"
        ));
    }
    out
}
