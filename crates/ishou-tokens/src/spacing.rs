//! Spacing scale — 4px base unit, modular multiples.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Spacing {
    pub px_0: u16,
    pub px_1: u16,
    pub px_2: u16,
    pub px_3: u16,
    pub px_4: u16,
    pub px_6: u16,
    pub px_8: u16,
    pub px_10: u16,
    pub px_12: u16,
    pub px_16: u16,
    pub px_20: u16,
    pub px_24: u16,
    pub px_32: u16,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            px_0: 0,
            px_1: 4,
            px_2: 8,
            px_3: 12,
            px_4: 16,
            px_6: 24,
            px_8: 32,
            px_10: 40,
            px_12: 48,
            px_16: 64,
            px_20: 80,
            px_24: 96,
            px_32: 128,
        }
    }
}

impl Spacing {
    /// (step-name, px) pairs — stable order, consumed by renderers.
    #[must_use]
    pub fn pairs(&self) -> [(&'static str, u16); 13] {
        [
            ("0", self.px_0),
            ("1", self.px_1),
            ("2", self.px_2),
            ("3", self.px_3),
            ("4", self.px_4),
            ("6", self.px_6),
            ("8", self.px_8),
            ("10", self.px_10),
            ("12", self.px_12),
            ("16", self.px_16),
            ("20", self.px_20),
            ("24", self.px_24),
            ("32", self.px_32),
        ]
    }
}
