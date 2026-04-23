//! GLSL renderer — emits a `#define` / `const` header consumable by the 13
//! blackmatter-ghostty shaders. Drop-in replacement for their hand-written
//! `nord-common.glsl`.

use ishou_tokens::TokenSet;

pub fn render(t: &TokenSet) -> String {
    let mut out = String::from("// ishou — pleme-io design tokens (GLSL, generated)\n\n");
    for (k, v) in t.color.entries() {
        out.push_str(&format!(
            "#define ISHOU_{} vec3({:.4}, {:.4}, {:.4})\n",
            k.to_uppercase(),
            f32::from(v.r) / 255.0,
            f32::from(v.g) / 255.0,
            f32::from(v.b) / 255.0,
        ));
    }
    // shader uniform defaults — renderers / hosts can override per-shader.
    let sh = &t.shader;
    out.push_str("\n// Default shader uniforms\n");
    out.push_str(&format!("#define ISHOU_BLOOM_THRESHOLD {:.3}\n", sh.bloom.threshold));
    out.push_str(&format!("#define ISHOU_BLOOM_INTENSITY {:.3}\n", sh.bloom.intensity));
    out.push_str(&format!("#define ISHOU_CA_MAGNITUDE {:.4}\n", sh.chromatic_aberration.magnitude));
    out.push_str(&format!("#define ISHOU_CURSOR_GLOW_RADIUS {:.2}\n", sh.cursor_glow.radius_px));
    out.push_str(&format!("#define ISHOU_FILM_GRAIN {:.4}\n", sh.film_grain.intensity));
    out.push_str(&format!("#define ISHOU_FROST_HAZE {:.3}\n", sh.frost_haze.intensity));
    out.push_str(&format!("#define ISHOU_SCREEN_CURV {:.4}\n", sh.screen_curvature.curvature));
    out.push_str(&format!("#define ISHOU_SCREEN_VIGN {:.3}\n", sh.screen_curvature.vignette));
    out.push_str(&format!("#define ISHOU_SONIC_INTENSITY {:.3}\n", sh.sonic_boom.intensity));
    out.push_str(&format!("#define ISHOU_SPOT_RADIUS {:.1}\n", sh.spotlight.radius_px));
    out.push_str(&format!("#define ISHOU_STARDUST_COUNT {}\n", sh.stardust.particle_count));
    out
}
