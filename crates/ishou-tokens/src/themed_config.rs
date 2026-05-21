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

/// One-line fleet-convergence guard for themed apps. Drop a single
/// `#[test]` calling this into your app's tests:
///
/// ```rust,ignore
/// #[test]
/// fn fleet_convergence() {
///     ishou_tokens::convergence::assert_themed(MyAppConfig::default());
/// }
/// ```
///
/// Asserts that the supplied config — typically the app's
/// `prescribed_default()` or `Default::default()` — has visual
/// fields matching `FleetDefaults::prescribed()`. The closure-based
/// API lets each app expose exactly which fields participate (apps
/// vary in coverage; some have font + theme, others have full
/// padding/cursor/scrollback).
pub mod convergence {
    use crate::fleet_defaults::FleetDefaults;
    use crate::fleet_keybinds::{FleetKeybinds, normalize_chord};
    use crate::fleet_theme::FleetTheme;

    /// Builder-style guard. Each `expect_*` call adds one assertion;
    /// `run()` panics on the first divergence with a clear message
    /// naming the field + the drift.
    ///
    /// Asserts against the union of `FleetDefaults::prescribed()`
    /// (visual: font / theme / padding / cursor / scrollback) and
    /// `FleetKeybinds::prescribed()` (input: history_picker /
    /// files_picker / clear_buffer / …).
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// #[test]
    /// fn mado_converges_with_fleet() {
    ///     ishou_tokens::convergence::Guard::for_app("mado")
    ///         .expect_font_family(MadoConfig::default().font_family.as_str())
    ///         .expect_font_size(MadoConfig::default().font_size)
    ///         .expect_padding(MadoConfig::default().padding)
    ///         .expect_history_picker(MadoConfig::default().keys.history_picker.as_str())
    ///         .run();
    /// }
    /// ```
    #[must_use]
    pub struct Guard {
        app: &'static str,
        fd: FleetDefaults,
        kb: FleetKeybinds,
        failures: Vec<String>,
    }

    impl Guard {
        /// Start a guard for `app_name` (used in panic messages).
        #[must_use]
        pub fn for_app(app_name: &'static str) -> Self {
            Self {
                app: app_name,
                fd: FleetDefaults::prescribed(),
                kb: FleetKeybinds::prescribed(),
                failures: Vec::new(),
            }
        }

        /// Assert `actual` matches `FleetDefaults::prescribed().font_family`.
        #[must_use]
        pub fn expect_font_family(mut self, actual: &str) -> Self {
            if actual != self.fd.font_family {
                self.failures.push(format!(
                    "{}: font_family drift — actual {:?} != fleet {:?}",
                    self.app, actual, self.fd.font_family
                ));
            }
            self
        }

        /// Assert `actual` matches `FleetDefaults::prescribed().font_size`
        /// (within 0.001 epsilon — f32 round-trips through serde).
        #[must_use]
        pub fn expect_font_size(mut self, actual: f32) -> Self {
            if (actual - self.fd.font_size).abs() >= 0.001 {
                self.failures.push(format!(
                    "{}: font_size drift — actual {} != fleet {}",
                    self.app, actual, self.fd.font_size
                ));
            }
            self
        }

        /// Assert `actual` matches `FleetDefaults::prescribed().padding`.
        #[must_use]
        pub fn expect_padding(mut self, actual: u32) -> Self {
            if actual != self.fd.padding {
                self.failures.push(format!(
                    "{}: padding drift — actual {} != fleet {}",
                    self.app, actual, self.fd.padding
                ));
            }
            self
        }

        /// Assert `actual` matches `FleetDefaults::prescribed().cursor_style`.
        #[must_use]
        pub fn expect_cursor_style(mut self, actual: &str) -> Self {
            if actual != self.fd.cursor_style {
                self.failures.push(format!(
                    "{}: cursor_style drift — actual {:?} != fleet {:?}",
                    self.app, actual, self.fd.cursor_style
                ));
            }
            self
        }

        /// Assert `actual` matches `FleetDefaults::prescribed().scrollback_lines`.
        #[must_use]
        pub fn expect_scrollback_lines(mut self, actual: usize) -> Self {
            if actual != self.fd.scrollback_lines {
                self.failures.push(format!(
                    "{}: scrollback_lines drift — actual {} != fleet {}",
                    self.app, actual, self.fd.scrollback_lines
                ));
            }
            self
        }

        /// Assert `actual` matches `FleetDefaults::prescribed().theme`.
        #[must_use]
        pub fn expect_theme(mut self, actual: FleetTheme) -> Self {
            if actual != self.fd.theme {
                self.failures.push(format!(
                    "{}: theme drift — actual {:?} != fleet {:?}",
                    self.app, actual, self.fd.theme
                ));
            }
            self
        }

        // ── Keybind intent expectations ──────────────────────────
        //
        // One method per `FleetKeybinds` field. Asserts the app's
        // bound chord for that intent matches the fleet canonical
        // chord. Drift is the 2026-05-21 Ctrl-R class of bug.

        /// Assert `actual` matches `FleetKeybinds::prescribed().history_picker`.
        #[must_use]
        pub fn expect_history_picker(mut self, actual: &str) -> Self {
            self.check_chord("history_picker", actual, self.kb.history_picker);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().files_picker`.
        #[must_use]
        pub fn expect_files_picker(mut self, actual: &str) -> Self {
            self.check_chord("files_picker", actual, self.kb.files_picker);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().dir_picker`.
        #[must_use]
        pub fn expect_dir_picker(mut self, actual: &str) -> Self {
            self.check_chord("dir_picker", actual, self.kb.dir_picker);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().content_picker`.
        #[must_use]
        pub fn expect_content_picker(mut self, actual: &str) -> Self {
            self.check_chord("content_picker", actual, self.kb.content_picker);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().clear_buffer`.
        #[must_use]
        pub fn expect_clear_buffer(mut self, actual: &str) -> Self {
            self.check_chord("clear_buffer", actual, self.kb.clear_buffer);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().kill_line`.
        #[must_use]
        pub fn expect_kill_line(mut self, actual: &str) -> Self {
            self.check_chord("kill_line", actual, self.kb.kill_line);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().edit_in_editor`.
        #[must_use]
        pub fn expect_edit_in_editor(mut self, actual: &str) -> Self {
            self.check_chord("edit_in_editor", actual, self.kb.edit_in_editor);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().help`.
        #[must_use]
        pub fn expect_help(mut self, actual: &str) -> Self {
            self.check_chord("help", actual, self.kb.help);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().clipboard_copy`.
        #[must_use]
        pub fn expect_clipboard_copy(mut self, actual: &str) -> Self {
            self.check_chord("clipboard_copy", actual, self.kb.clipboard_copy);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().clipboard_paste`.
        #[must_use]
        pub fn expect_clipboard_paste(mut self, actual: &str) -> Self {
            self.check_chord("clipboard_paste", actual, self.kb.clipboard_paste);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().toggle_sudo`.
        #[must_use]
        pub fn expect_toggle_sudo(mut self, actual: &str) -> Self {
            self.check_chord("toggle_sudo", actual, self.kb.toggle_sudo);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().insert_last_arg`.
        #[must_use]
        pub fn expect_insert_last_arg(mut self, actual: &str) -> Self {
            self.check_chord("insert_last_arg", actual, self.kb.insert_last_arg);
            self
        }

        /// Assert `actual` matches `FleetKeybinds::prescribed().multiplexer_prefix`.
        #[must_use]
        pub fn expect_multiplexer_prefix(mut self, actual: &str) -> Self {
            self.check_chord("multiplexer_prefix", actual, self.kb.multiplexer_prefix);
            self
        }

        fn check_chord(&mut self, intent: &str, actual: &str, expected: &str) {
            // Canonicalize both sides through `normalize_chord` so apps
            // that store the long form ("ctrl+b") match the atlas's
            // short form ("C-b") and vice versa. The normalizer is
            // idempotent so already-long-form input passes through; the
            // chord identity (modifier+key) is what we're asserting,
            // not the syntactic notation.
            let actual_norm = normalize_chord(actual);
            let expected_norm = normalize_chord(expected);
            if actual_norm != expected_norm {
                self.failures.push(format!(
                    "{}: {} chord drift — actual {:?} (normalized {:?}) != fleet {:?} (normalized {:?})",
                    self.app, intent, actual, actual_norm, expected, expected_norm,
                ));
            }
        }

        /// Run all collected assertions. Panics on the first failure
        /// with a multi-line message naming every drift; use in tests.
        ///
        /// # Panics
        ///
        /// Panics if any `expect_*` call recorded a drift, with a
        /// message listing every field that drifted from the fleet
        /// baseline.
        pub fn run(self) {
            assert!(
                self.failures.is_empty(),
                "{} drifted from FleetDefaults::prescribed():\n  - {}",
                self.app,
                self.failures.join("\n  - ")
            );
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn empty_guard_passes() {
            Guard::for_app("fixture").run();
        }

        #[test]
        fn matching_font_family_passes() {
            let fd = FleetDefaults::prescribed();
            Guard::for_app("fixture")
                .expect_font_family(&fd.font_family)
                .run();
        }

        #[test]
        #[should_panic(expected = "font_family drift")]
        fn divergent_font_family_panics_with_clear_message() {
            Guard::for_app("fixture")
                .expect_font_family("NotTheFleetFont")
                .run();
        }

        #[test]
        fn full_guard_chain_on_prescribed_defaults_passes() {
            let fd = FleetDefaults::prescribed();
            Guard::for_app("fixture")
                .expect_font_family(&fd.font_family)
                .expect_font_size(fd.font_size)
                .expect_padding(fd.padding)
                .expect_cursor_style(&fd.cursor_style)
                .expect_scrollback_lines(fd.scrollback_lines)
                .expect_theme(fd.theme)
                .run();
        }

        #[test]
        #[should_panic(expected = "scrollback_lines drift")]
        fn divergence_message_names_the_drifting_field() {
            Guard::for_app("fixture")
                .expect_scrollback_lines(42)
                .run();
        }

        #[test]
        fn full_keybind_chain_on_prescribed_passes() {
            let kb = FleetKeybinds::prescribed();
            Guard::for_app("fixture")
                .expect_history_picker(kb.history_picker)
                .expect_files_picker(kb.files_picker)
                .expect_dir_picker(kb.dir_picker)
                .expect_content_picker(kb.content_picker)
                .expect_clear_buffer(kb.clear_buffer)
                .expect_kill_line(kb.kill_line)
                .expect_edit_in_editor(kb.edit_in_editor)
                .expect_help(kb.help)
                .expect_clipboard_copy(kb.clipboard_copy)
                .expect_clipboard_paste(kb.clipboard_paste)
                .expect_toggle_sudo(kb.toggle_sudo)
                .expect_insert_last_arg(kb.insert_last_arg)
                .expect_multiplexer_prefix(kb.multiplexer_prefix)
                .run();
        }

        #[test]
        #[should_panic(expected = "history_picker chord drift")]
        fn divergent_history_picker_panics_with_clear_message() {
            Guard::for_app("fixture").expect_history_picker("C-z").run();
        }

        #[test]
        #[should_panic(expected = "multiplexer_prefix chord drift")]
        fn divergent_multiplexer_prefix_panics() {
            Guard::for_app("fixture").expect_multiplexer_prefix("C-a").run();
        }

        #[test]
        fn guard_matches_long_form_chord_against_short_form_atlas() {
            // Atlas declares `multiplexer_prefix = "C-b"`; tear (and any
            // other consumer that stores the long form) supplies
            // "ctrl+b". The Guard normalizes both sides so the chord
            // identity matches, not the syntactic surface. Closes the
            // 2026-05-21 tear-tear-config Guard-normalization gap.
            Guard::for_app("tear-fixture")
                .expect_multiplexer_prefix("ctrl+b")
                .run();
        }

        #[test]
        fn guard_matches_short_form_chord_against_short_form_atlas() {
            // Sanity — supplying the atlas's own form also matches.
            Guard::for_app("frostmourne-fixture")
                .expect_history_picker("C-r")
                .run();
        }

        #[test]
        fn guard_matches_long_form_history_picker_against_atlas() {
            // The full short ↔ long match for non-multiplexer chords too.
            Guard::for_app("future-consumer")
                .expect_history_picker("ctrl+r")
                .run();
        }

        #[test]
        #[should_panic(expected = "history_picker chord drift")]
        fn guard_rejects_genuinely_different_chord_even_with_normalization() {
            // C-q is a real, different chord — must still fail even
            // though the notation game succeeds.
            Guard::for_app("fixture")
                .expect_history_picker("ctrl+q")
                .run();
        }
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
