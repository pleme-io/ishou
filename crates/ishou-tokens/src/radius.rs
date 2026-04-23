//! Border radius tokens.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Radius {
    pub none: u16,
    pub sm: u16,
    pub md: u16,
    pub lg: u16,
    pub xl: u16,
    pub pill: u16,
    pub full: u16,
}

impl Default for Radius {
    fn default() -> Self {
        Self {
            none: 0,
            sm: 4,
            md: 8,
            lg: 12,
            xl: 16,
            pill: 999,
            full: 9999,
        }
    }
}

impl Radius {
    #[must_use]
    pub fn pairs(&self) -> [(&'static str, u16); 7] {
        [
            ("none", self.none),
            ("sm", self.sm),
            ("md", self.md),
            ("lg", self.lg),
            ("xl", self.xl),
            ("pill", self.pill),
            ("full", self.full),
        ]
    }
}
