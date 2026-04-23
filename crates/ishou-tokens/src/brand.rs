//! Brand identity tokens — logo paths, mark variants, geometry.
//!
//! The pleme-io brand vocabulary: **bold black, white paper, swerves**. The
//! swerve is the signature geometric primitive — a curve-and-counter-curve
//! stroke that forms the `pleme` mark. These tokens describe the mark
//! geometrically so renderers can stamp it into SVG, GLSL paths, or keep it
//! as a reference to an external asset.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Brand {
    pub name: &'static str,
    pub tagline: &'static str,
    pub logo_paths: LogoAssets,
    pub swerve: Swerve,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogoAssets {
    /// Full lockup (mark + wordmark) — primary.
    pub lockup: &'static str,
    /// Mark-only variant for favicons, avatars, 1-bit contexts.
    pub mark: &'static str,
    /// Monochrome-black lockup — for light backgrounds.
    pub lockup_mono_black: &'static str,
    /// Monochrome-white lockup — for dark backgrounds.
    pub lockup_mono_white: &'static str,
    /// Favicon (SVG, multi-size embedded).
    pub favicon: &'static str,
}

/// Geometric parameterization of the pleme-io swerve mark.
///
/// The swerve is drawn as two mirrored cubic Bézier segments inside a 1:1
/// viewBox. Renderers can expand this into an SVG path, a GLSL signed-
/// distance field, or a Rust `lyon` path for custom rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Swerve {
    pub viewbox: u16,          // e.g. 64 → "0 0 64 64"
    pub stroke_width: u16,     // in viewbox units
    pub stroke_cap: StrokeCap, // round / square / butt
    /// SVG path `d` attribute for the swerve stroke. Pre-computed so all
    /// renderers agree byte-for-byte.
    pub path_d: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

impl Brand {
    #[must_use]
    pub const fn pleme() -> Self {
        Self {
            name: "pleme-io",
            tagline: "Typed infrastructure for the next decade",
            logo_paths: LogoAssets {
                lockup: "assets/logo.svg",
                mark: "assets/mark.svg",
                lockup_mono_black: "assets/logo-mono-black.svg",
                lockup_mono_white: "assets/logo-mono-white.svg",
                favicon: "assets/favicon.svg",
            },
            // The swerve: starts top-left, curves down to mid-bottom, counter-
            // curves up to top-right. Drawn at 64×64 so scales cleanly.
            swerve: Swerve {
                viewbox: 64,
                stroke_width: 10,
                stroke_cap: StrokeCap::Round,
                path_d: "M 8 10 C 14 46, 30 52, 32 32 C 34 12, 50 18, 56 54",
            },
        }
    }
}
