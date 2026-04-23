//! Typography tokens — font stacks + modular scale.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Typography {
    pub families: FontFamilies,
    pub scale: TypeScale,
    pub weight: FontWeights,
    pub line_height: LineHeights,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FontFamilies {
    pub serif: &'static str,
    pub sans: &'static str,
    pub mono: &'static str,
    pub display: &'static str,
}

/// 1.25 modular scale from 0.75rem base-ish, in em-style sizes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypeScale {
    pub xs: f32,
    pub sm: f32,
    pub base: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub x2: f32,
    pub x3: f32,
    pub x4: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FontWeights {
    pub light: u16,
    pub regular: u16,
    pub medium: u16,
    pub semibold: u16,
    pub bold: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineHeights {
    pub tight: f32,
    pub base: f32,
    pub relaxed: f32,
    pub prose: f32,
}

impl Typography {
    #[must_use]
    pub const fn pleme() -> Self {
        Self {
            families: FontFamilies {
                serif: "'Iowan Old Style', 'Charter', 'Georgia', 'Cambria', Times, serif",
                sans: "'Inter', system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif",
                mono: "'JetBrains Mono', 'SF Mono', 'Menlo', 'Consolas', monospace",
                // Display = used for brand titles / hero headings; swerve-forward.
                display: "'Inter', system-ui, sans-serif",
            },
            scale: TypeScale {
                xs: 0.75,
                sm: 0.875,
                base: 1.0,
                md: 1.125,
                lg: 1.25,
                xl: 1.5,
                x2: 1.875,
                x3: 2.25,
                x4: 3.0,
            },
            weight: FontWeights {
                light: 300,
                regular: 400,
                medium: 500,
                semibold: 600,
                bold: 700,
            },
            line_height: LineHeights {
                tight: 1.15,
                base: 1.5,
                relaxed: 1.65,
                prose: 1.7,
            },
        }
    }
}
