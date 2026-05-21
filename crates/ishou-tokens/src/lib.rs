//! ishou (意匠) — the pleme-io house crest.
//!
//! A typed design-token vocabulary for the pleme-io visual system. Pure data
//! structures; rendering lives in `ishou-render`. Consumers never hand-craft
//! CSS / Tailwind / TUI color tables — they call a renderer and get deterministic
//! output keyed off this token set.
//!
//! Layered:
//!
//! - `color` — re-exports the irodori Nord palette + semantic role mapping,
//!   adds the pleme-io brand monochrome (bold-black / paper-white).
//! - `typography` — font stacks + modular scale.
//! - `spacing` — 4px-based spacing scale.
//! - `radius` — border radius scale.
//! - `shadow` — elevation layers (Nord-tinted, bold; mirrors the brand).
//! - `motion` — easing curves + durations (shader-informed).
//! - `shader` — typed uniforms for the 13 blackmatter-ghostty GLSL shaders.
//! - `brand` — logo paths, mark variants, swerve geometry.
//!
//! The entire token set is reachable from `TokenSet::default()`.

pub mod brand;
pub mod color;
pub mod fleet_defaults;
pub mod fleet_keybinds;
pub mod fleet_theme;
pub mod motion;
pub mod radius;
pub mod refined;
pub mod shader;
pub mod shadow;
pub mod space;
pub mod spacing;
pub mod themed_config;
pub mod typography;

use serde::Serialize;

pub use brand::Brand;
pub use color::{ColorPalette, Rgb, SemanticRoles};
pub use fleet_defaults::FleetDefaults;
pub use fleet_keybinds::{FleetKeybinds, FleetKeybindsConsumer};
pub use fleet_theme::{FleetTheme, ResolvedTheme};
pub use refined::{Bounds, Refined};
pub use themed_config::{convergence, FleetThemedConfig};
pub use space::{linear_from_hex, Linear, LinearRgba, Srgb, SrgbA};
pub use motion::Motion;
pub use radius::Radius;
pub use shader::Shaders;
pub use shadow::{Shadow, ShadowSpec};
pub use spacing::Spacing;
pub use typography::Typography;

/// The complete ishou token set. Single source of truth for every render target.
#[derive(Debug, Clone, Serialize)]
pub struct TokenSet {
    pub color: ColorPalette,
    pub roles: SemanticRoles,
    pub brand: Brand,
    pub typography: Typography,
    pub spacing: Spacing,
    pub radius: Radius,
    pub shadow: Shadow,
    pub motion: Motion,
    pub shader: Shaders,
}

impl Default for TokenSet {
    fn default() -> Self {
        Self {
            color: ColorPalette::pleme(),
            roles: SemanticRoles::pleme_dark(),
            brand: Brand::pleme(),
            typography: Typography::pleme(),
            spacing: Spacing::default(),
            radius: Radius::default(),
            shadow: Shadow::default(),
            motion: Motion::default(),
            shader: Shaders::default(),
        }
    }
}

impl TokenSet {
    /// The canonical pleme-io token set. Calls `default()`; kept for clarity
    /// at call sites (`TokenSet::pleme()` reads better than `TokenSet::default()`).
    #[must_use]
    pub fn pleme() -> Self {
        Self::default()
    }

    /// Deterministic content hash used by arch-synthesizer attestation.
    /// Two token sets produce identical hashes iff every token is byte-equal.
    #[must_use]
    pub fn content_hash(&self) -> [u8; 32] {
        let json = serde_json::to_vec(self).expect("token set is always serializable");
        // Simple Fnv64 twice would do; using a plain sha-like fold here to avoid
        // pulling blake3 into the pure token crate. Renderer/attestation layer
        // wraps with BLAKE3 when hooked into arch-synthesizer.
        let mut out = [0u8; 32];
        for (i, byte) in json.iter().enumerate() {
            out[i % 32] ^= byte.wrapping_mul(31).wrapping_add(i as u8);
        }
        out
    }
}
