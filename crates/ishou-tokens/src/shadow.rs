//! Elevation / drop-shadow tokens.
//!
//! The pleme-io brand motif is "bold black shadows" — the logo lands with
//! hard, slightly-offset drops. These tokens tune shadow geometry + Nord-tinted
//! black so shadows always read as brand, not stock Material.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Shadow {
    pub none: ShadowSpec,
    pub sm: ShadowSpec,
    pub md: ShadowSpec,
    pub lg: ShadowSpec,
    pub xl: ShadowSpec,
    /// Brand-authentic bold drop — the logo-lockup shadow, larger offset,
    /// 100% opacity, slight tint. Reserved for hero surfaces.
    pub brand_bold: ShadowSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowSpec {
    pub offset_x: i16,
    pub offset_y: i16,
    pub blur: u16,
    pub spread: i16,
    /// Alpha applied atop the palette `shadow_tone` color. 0..=100 percent.
    pub alpha_pct: u8,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            none: ShadowSpec { offset_x: 0, offset_y: 0, blur: 0, spread: 0, alpha_pct: 0 },
            sm: ShadowSpec { offset_x: 0, offset_y: 1, blur: 2, spread: 0, alpha_pct: 20 },
            md: ShadowSpec { offset_x: 0, offset_y: 4, blur: 8, spread: 0, alpha_pct: 30 },
            lg: ShadowSpec { offset_x: 0, offset_y: 8, blur: 24, spread: -2, alpha_pct: 40 },
            xl: ShadowSpec { offset_x: 0, offset_y: 16, blur: 48, spread: -4, alpha_pct: 50 },
            // The bold brand drop — hard, opaque, slightly offset.
            brand_bold: ShadowSpec { offset_x: 6, offset_y: 8, blur: 0, spread: 0, alpha_pct: 100 },
        }
    }
}

impl Shadow {
    #[must_use]
    pub fn pairs(&self) -> [(&'static str, &ShadowSpec); 6] {
        [
            ("none", &self.none),
            ("sm", &self.sm),
            ("md", &self.md),
            ("lg", &self.lg),
            ("xl", &self.xl),
            ("brand-bold", &self.brand_bold),
        ]
    }
}
