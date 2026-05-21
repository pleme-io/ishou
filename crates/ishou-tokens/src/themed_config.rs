//! The `FleetThemedConfig` trait — convergence by construction.
//!
//! # Why
//!
//! Every fleet app with visual config (mado, namimado, escriba,
//! hibiki, hikki, hikyaku, fumi, kekkai, taimen, …) hand-rolled
//! `shikumi::TieredConfig::prescribed_default()` reading hardcoded
//! constants — `"JetBrainsMono Nerd Font Mono"`, `14.0`, `FleetTheme::PlemeDark`.
//! Duplication that drifts.
//!
//! `FleetThemedConfig` makes the convergence a **type-level
//! guarantee**: a themed config implements `from_fleet(&FleetDefaults)`,
//! and its `TieredConfig::prescribed_default()` delegates to that
//! method passing `FleetDefaults::prescribed()`. Changing one
//! field of `FleetDefaults` propagates to every themed config on
//! next compile — the compiler enforces consistency.
//!
//! # Pattern
//!
//! ```rust,ignore
//! use ishou_tokens::{FleetDefaults, FleetThemedConfig, FleetTheme};
//! use shikumi::TieredConfig;
//!
//! pub struct MyAppConfig {
//!     pub theme: FleetTheme,
//!     pub font_family: String,
//!     pub font_size: f32,
//!     pub padding: u32,
//!     // ...app-specific fields...
//! }
//!
//! impl FleetThemedConfig for MyAppConfig {
//!     fn from_fleet(fd: &FleetDefaults) -> Self {
//!         Self {
//!             theme: fd.theme,
//!             font_family: fd.font_family.clone(),
//!             font_size: fd.font_size,
//!             padding: fd.padding,
//!             ..<Self as TieredConfig>::bare()
//!         }
//!     }
//! }
//!
//! impl TieredConfig for MyAppConfig {
//!     fn bare() -> Self { /* explicit zero-opinion */ }
//!     fn prescribed_default() -> Self {
//!         <Self as FleetThemedConfig>::from_fleet(&FleetDefaults::prescribed())
//!     }
//! }
//! ```
//!
//! Now the app's `prescribed_default()` is **derived** from the
//! fleet baseline — not duplicated.
//!
//! # Anti-pattern
//!
//! Manually copying FleetDefaults values into a `prescribed_default()`
//! body. The next FleetDefaults change won't propagate — and the
//! compiler won't tell you. Use `FleetThemedConfig` whenever the
//! app has theme/font/padding/cursor fields.

use crate::fleet_defaults::FleetDefaults;

/// Marker + factory trait. Any pleme-io Rust app config with visual
/// fields (theme/font/padding/cursor) impls this so its prescribed
/// tier is mechanically derived from `FleetDefaults` instead of
/// hand-rolled.
///
/// The trait pairs with `shikumi::TieredConfig`. Both must be
/// implemented; `prescribed_default()` typically delegates to
/// `<Self as FleetThemedConfig>::from_fleet(&FleetDefaults::prescribed())`.
pub trait FleetThemedConfig: Sized {
    /// Compose a config instance from a fleet baseline. App-specific
    /// fields (those NOT in `FleetDefaults`) should typically come
    /// from `<Self as shikumi::TieredConfig>::bare()` via struct
    /// update syntax (`..Self::bare()`).
    ///
    /// Touching `FleetDefaults` changes propagate here on next
    /// compile — the compiler enforces.
    fn from_fleet(fd: &FleetDefaults) -> Self;
}

impl FleetDefaults {
    /// Materialize any `FleetThemedConfig` from this `FleetDefaults`
    /// instance. Reads at call-site:
    ///
    /// ```rust,ignore
    /// let cfg: MyAppConfig = FleetDefaults::prescribed().apply_to();
    /// ```
    #[must_use]
    pub fn apply_to<C: FleetThemedConfig>(&self) -> C {
        C::from_fleet(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fleet_theme::FleetTheme;

    /// Fixture: an app config with visual fields wired to FleetDefaults.
    #[derive(Debug, Clone, PartialEq)]
    struct TestAppConfig {
        theme: FleetTheme,
        font_family: String,
        font_size: f32,
        padding: u32,
        cursor_style: String,
        app_specific_flag: bool,
    }

    impl TestAppConfig {
        fn bare() -> Self {
            Self {
                theme: FleetTheme::Bare,
                font_family: String::new(),
                font_size: 0.0,
                padding: 0,
                cursor_style: String::new(),
                app_specific_flag: false,
            }
        }
    }

    impl FleetThemedConfig for TestAppConfig {
        fn from_fleet(fd: &FleetDefaults) -> Self {
            Self {
                theme: fd.theme,
                font_family: fd.font_family.clone(),
                font_size: fd.font_size,
                padding: fd.padding,
                cursor_style: fd.cursor_style.clone(),
                ..Self::bare()
            }
        }
    }

    #[test]
    fn from_fleet_pulls_canonical_pleme_values() {
        let fd = FleetDefaults::prescribed();
        let cfg = TestAppConfig::from_fleet(&fd);
        assert_eq!(cfg.theme, FleetTheme::PlemeDark);
        assert_eq!(cfg.font_family, "JetBrainsMono Nerd Font Mono");
        assert_eq!(cfg.font_size, 14.0);
        assert_eq!(cfg.padding, 0);
        assert_eq!(cfg.cursor_style, "block");
    }

    #[test]
    fn from_fleet_bare_yields_bare_visuals() {
        let fd = FleetDefaults::bare();
        let cfg = TestAppConfig::from_fleet(&fd);
        assert_eq!(cfg.theme, FleetTheme::Bare);
        assert_eq!(cfg.font_family, "");
        assert_eq!(cfg.font_size, 12.0);
        assert_eq!(cfg.padding, 0);
    }

    #[test]
    fn app_specific_fields_inherit_from_bare() {
        let fd = FleetDefaults::prescribed();
        let cfg = TestAppConfig::from_fleet(&fd);
        // app_specific_flag isn't in FleetDefaults, must come from Self::bare().
        assert!(!cfg.app_specific_flag);
    }

    #[test]
    fn apply_to_helper_round_trips_to_from_fleet() {
        let fd = FleetDefaults::prescribed();
        let direct = TestAppConfig::from_fleet(&fd);
        let via_helper: TestAppConfig = fd.apply_to();
        assert_eq!(direct, via_helper);
    }

    #[test]
    fn cross_app_theme_converges_when_fleet_defaults_change() {
        // Two different fixture configs both pull from the same
        // FleetDefaults — proves the convergence guarantee at the
        // type level. (FleetDefaults isn't mutable in this test,
        // but if it were, both would track in lockstep.)
        #[derive(Debug, Clone, PartialEq)]
        struct OtherApp {
            theme: FleetTheme,
            font_family: String,
            scrollback: usize,
        }
        impl OtherApp {
            fn bare() -> Self {
                Self { theme: FleetTheme::Bare, font_family: String::new(), scrollback: 0 }
            }
        }
        impl FleetThemedConfig for OtherApp {
            fn from_fleet(fd: &FleetDefaults) -> Self {
                Self {
                    theme: fd.theme,
                    font_family: fd.font_family.clone(),
                    ..Self::bare()
                }
            }
        }

        let fd = FleetDefaults::prescribed();
        let a = TestAppConfig::from_fleet(&fd);
        let b = OtherApp::from_fleet(&fd);
        // Both see the same theme + font, by construction.
        assert_eq!(a.theme, b.theme);
        assert_eq!(a.font_family, b.font_family);
    }
}
