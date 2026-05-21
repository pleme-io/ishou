//! Fleet keybindings — the canonical cross-app chord atlas.
//!
//! # Why
//!
//! Peer of [`FleetDefaults`](crate::FleetDefaults) for the input
//! surface. Every operator-facing pleme-io app (mado, tear, frost,
//! frostmourne, ayatsuri, namimado, escriba, …) makes the same set
//! of "what chord opens the history picker / clears the buffer /
//! switches panes" choices. Without a shared source these answers
//! drift — frost's `(defbind :key "C-r" ...)` diverges from a
//! separate `(defpicker :key "C-r" ...)` (incident 2026-05-21,
//! Ctrl-R unresponsive); mado's input policy hard-codes Cmd-= while
//! tear hard-codes C-b; the fleet stops feeling like one product
//! and operators lose muscle memory at every boundary.
//!
//! `FleetKeybinds` is the **single typed source** every fleet app's
//! `shikumi::TieredConfig::prescribed_default()` reads from for its
//! keybinding fields. Apps that bind "history picker" pick
//! `FleetKeybinds::prescribed().history_picker`. Changing the fleet
//! chord = one line touch here = every app converges next launch.
//!
//! # Chord notation
//!
//! `&'static str` carrying the canonical short-form notation shared
//! by frost-lisp's `defbind :key`, tear-types' `KeyChord` `from_tmux`,
//! and awase's `Hotkey::parse`:
//!
//! - `"C-r"` = Ctrl + R
//! - `"M-c"` = Alt/Meta + C
//! - `"C-x e"` = Ctrl + X, then E (multi-chord)
//! - `"cmd+space"` = Cmd + Space (long form for desktop hotkeys)
//!
//! All three downstream parsers normalize to the same internal
//! `Hotkey`. Keeping the field type `&'static str` lets the atlas
//! be a `const` value at the cost of no compile-time parse — that
//! cost is paid once per consumer via `awase::Hotkey::parse(…)`
//! or equivalent.
//!
//! # The bare/prescribed contract
//!
//! Mirrors [`FleetDefaults`](crate::FleetDefaults). Apps with bare
//! semantics use `FleetKeybinds::bare()` (every chord empty); the
//! 90% case uses `FleetKeybinds::prescribed()` (the pleme-io
//! developer-prescribed muscle-memory map).
//!
//! # Migration template for fleet apps
//!
//! ```rust,ignore
//! use ishou_tokens::{FleetKeybinds, FleetKeybindsConsumer};
//!
//! pub struct MyAppKeyConfig {
//!     pub history_picker: String,
//!     pub clear_buffer:   String,
//!     // ...app-specific bindings...
//! }
//!
//! impl FleetKeybindsConsumer for MyAppKeyConfig {
//!     fn from_keybinds(kb: &FleetKeybinds) -> Self {
//!         Self {
//!             history_picker: kb.history_picker.into(),
//!             clear_buffer:   kb.clear_buffer.into(),
//!         }
//!     }
//! }
//! ```

use serde::Serialize;

/// The cross-app keybinding atlas every fleet app references.
///
/// Each field is a named operator-intent → canonical chord (short-
/// form `&'static str`). Adding a new fleet-wide chord = one field
/// here + one `expect_*` method on
/// [`convergence::Guard`](crate::convergence::Guard) + one mapping
/// in each consumer's `from_keybinds`.
///
/// # Two constructors
///
/// * [`FleetKeybinds::bare`] — zero-opinion floor; every chord
///   `""`. Apps with bare semantics inherit this.
/// * [`FleetKeybinds::prescribed`] — the canonical pleme-io muscle-
///   memory map (skim-pickers on `C-r`/`C-t`/`M-c`/`C-f`, reedline-
///   style emacs widgets, tear multiplexer prefix `C-b`). Touching
///   this propagates fleet-wide on next launch.
///
/// # Intents covered
///
/// **Shell pickers** (frost / frostmourne, blzsh-parity):
/// `history_picker`, `files_picker`, `dir_picker`, `content_picker`.
///
/// **Shell line-editing widgets** (frost / frostmourne):
/// `clear_buffer`, `kill_line`, `edit_in_editor`, `help`,
/// `clipboard_copy`, `clipboard_paste`, `toggle_sudo`,
/// `insert_last_arg`.
///
/// **Terminal multiplexer** (tear): `multiplexer_prefix`.
///
/// New intents land here as they emerge fleet-wide. Single-app
/// chords stay in the app's own config; only chords with ≥2
/// consumers (or ones that operators expect to be fleet-canonical
/// like Ctrl-R) belong here.
/// `Serialize` (not `Deserialize`) is intentional. The atlas is
/// **code**, not config — its value comes from being the single
/// hand-edited source every fleet app shares. Letting operators
/// override it via YAML would defeat the point. Serialize is
/// kept so snapshots/diagnostics (e.g. `frost-mcp frost_status`)
/// can emit it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FleetKeybinds {
    // ── Shell pickers (skim-backed) ──────────────────────────────
    /// History search picker (★ canonical Ctrl-R muscle memory).
    pub history_picker: &'static str,
    /// File picker — insert path at cursor.
    pub files_picker: &'static str,
    /// Directory picker — emit `cd <dir>` and auto-submit.
    pub dir_picker: &'static str,
    /// Content (rg-grep) picker — open editor at line.
    pub content_picker: &'static str,

    // ── Shell line-editing widgets (reedline / blzsh-parity) ─────
    /// Clear the buffer / repaint the prompt.
    pub clear_buffer: &'static str,
    /// Kill (cut) the entire buffer.
    pub kill_line: &'static str,
    /// Edit the current command in `$EDITOR`, then submit.
    pub edit_in_editor: &'static str,
    /// Show help / cheatsheet.
    pub help: &'static str,
    /// Copy buffer to OS clipboard.
    pub clipboard_copy: &'static str,
    /// Paste OS clipboard into buffer.
    pub clipboard_paste: &'static str,
    /// Toggle leading `sudo ` on the current buffer.
    pub toggle_sudo: &'static str,
    /// Insert the last argument of the previous command (M-. emacs).
    pub insert_last_arg: &'static str,

    // ── Terminal multiplexer (tear) ──────────────────────────────
    /// Prefix-key for tear multiplexer commands (`prefix + c` =
    /// new window, etc.).
    pub multiplexer_prefix: &'static str,
}

impl FleetKeybinds {
    /// **Tier 0 — bare**: every chord empty. Apps with bare
    /// semantics inherit this. Matches the `bare()` contract of
    /// `shikumi::TieredConfig`.
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            history_picker: "",
            files_picker: "",
            dir_picker: "",
            content_picker: "",
            clear_buffer: "",
            kill_line: "",
            edit_in_editor: "",
            help: "",
            clipboard_copy: "",
            clipboard_paste: "",
            toggle_sudo: "",
            insert_last_arg: "",
            multiplexer_prefix: "",
        }
    }

    /// **Tier 2 — prescribed**: the canonical pleme-io muscle-
    /// memory map. Every fleet app's `prescribed_default()`
    /// references this so input + output consistency is enforced
    /// by construction.
    ///
    /// Touching this function propagates fleet-wide on next
    /// compile + rebuild.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            // Shell pickers — match blackmatter-shell's skim bindings
            // so blzsh → frostmourne operators lose no muscle memory.
            history_picker: "C-r",
            files_picker: "C-t",
            dir_picker: "M-c",
            content_picker: "C-f",
            // Reedline emacs-style widgets — match blzsh widget set.
            clear_buffer: "C-l",
            kill_line: "C-u",
            edit_in_editor: "C-x e",
            help: "M-?",
            clipboard_copy: "M-y",
            clipboard_paste: "M-Y",
            toggle_sudo: "M-s",
            insert_last_arg: "M-.",
            // tear multiplexer — tmux-conventional C-b. Operators
            // moving from tmux find their prefix in the same place.
            multiplexer_prefix: "C-b",
        }
    }
}

impl Default for FleetKeybinds {
    fn default() -> Self {
        Self::prescribed()
    }
}

/// Marker + factory trait. Any pleme-io Rust app config with
/// keybinding fields impls this so its prescribed tier is
/// mechanically derived from `FleetKeybinds` instead of hand-rolled.
///
/// Peer of [`FleetThemedConfig`](crate::FleetThemedConfig) for the
/// input surface. Both pair with `shikumi::TieredConfig`; apps with
/// both visual + keybinding config implement both traits.
pub trait FleetKeybindsConsumer: Sized {
    /// Compose a config instance from a fleet keybinds atlas. App-
    /// specific bindings (those NOT in `FleetKeybinds`) should
    /// typically come from `<Self as shikumi::TieredConfig>::bare()`
    /// via struct update syntax (`..Self::bare()`).
    ///
    /// Touching `FleetKeybinds` changes propagate here on next
    /// compile — the compiler enforces.
    fn from_keybinds(kb: &FleetKeybinds) -> Self;
}

impl FleetKeybinds {
    /// Materialize any `FleetKeybindsConsumer` from this
    /// `FleetKeybinds` instance. Reads at call-site:
    ///
    /// ```rust,ignore
    /// let cfg: MyAppKeyConfig = FleetKeybinds::prescribed().apply_to();
    /// ```
    #[must_use]
    pub fn apply_to<C: FleetKeybindsConsumer>(&self) -> C {
        C::from_keybinds(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_has_every_chord_empty() {
        let b = FleetKeybinds::bare();
        assert_eq!(b.history_picker, "");
        assert_eq!(b.files_picker, "");
        assert_eq!(b.dir_picker, "");
        assert_eq!(b.content_picker, "");
        assert_eq!(b.clear_buffer, "");
        assert_eq!(b.kill_line, "");
        assert_eq!(b.edit_in_editor, "");
        assert_eq!(b.help, "");
        assert_eq!(b.clipboard_copy, "");
        assert_eq!(b.clipboard_paste, "");
        assert_eq!(b.toggle_sudo, "");
        assert_eq!(b.insert_last_arg, "");
        assert_eq!(b.multiplexer_prefix, "");
    }

    #[test]
    fn prescribed_carries_canonical_muscle_memory_chords() {
        let p = FleetKeybinds::prescribed();
        // The ★ binding — every fleet operator expects C-r.
        assert_eq!(p.history_picker, "C-r");
        assert_eq!(p.files_picker, "C-t");
        assert_eq!(p.dir_picker, "M-c");
        assert_eq!(p.content_picker, "C-f");
        assert_eq!(p.clear_buffer, "C-l");
        assert_eq!(p.kill_line, "C-u");
        assert_eq!(p.edit_in_editor, "C-x e");
        assert_eq!(p.help, "M-?");
        assert_eq!(p.clipboard_copy, "M-y");
        assert_eq!(p.clipboard_paste, "M-Y");
        assert_eq!(p.toggle_sudo, "M-s");
        assert_eq!(p.insert_last_arg, "M-.");
        assert_eq!(p.multiplexer_prefix, "C-b");
    }

    #[test]
    fn default_returns_prescribed_not_bare() {
        assert_eq!(FleetKeybinds::default(), FleetKeybinds::prescribed());
    }

    #[test]
    fn serializes_to_yaml_for_diagnostics() {
        // Serialize-only (no round-trip) — the atlas is code, not
        // config. Snapshots (frost-mcp, mado config_get) emit it.
        let p = FleetKeybinds::prescribed();
        let s = serde_yaml::to_string(&p).unwrap();
        assert!(s.contains("history_picker: C-r"), "yaml: {s}");
        assert!(s.contains("multiplexer_prefix: C-b"), "yaml: {s}");
    }

    #[test]
    fn no_two_intents_collide_on_the_same_chord() {
        // Compile-time-ish guard: in prescribed(), every intent gets
        // its own chord. A drift that double-binds (like the 2026-05-21
        // Ctrl-R incident) fails this test the moment the atlas
        // ships a duplicate.
        let p = FleetKeybinds::prescribed();
        let chords = [
            p.history_picker,
            p.files_picker,
            p.dir_picker,
            p.content_picker,
            p.clear_buffer,
            p.kill_line,
            p.edit_in_editor,
            p.help,
            p.clipboard_copy,
            p.clipboard_paste,
            p.toggle_sudo,
            p.insert_last_arg,
            p.multiplexer_prefix,
        ];
        let mut sorted: Vec<&str> = chords.to_vec();
        sorted.sort_unstable();
        let len_before = sorted.len();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            len_before,
            "FleetKeybinds::prescribed() has duplicate chord — would re-create the 2026-05-21 Ctrl-R double-bind incident"
        );
    }

    /// Fixture: an app config consuming the atlas.
    #[derive(Debug, Clone, PartialEq)]
    struct TestAppKeyConfig {
        history_picker: String,
        clear_buffer: String,
        app_specific_chord: String,
    }

    impl TestAppKeyConfig {
        fn bare() -> Self {
            Self {
                history_picker: String::new(),
                clear_buffer: String::new(),
                app_specific_chord: "C-z".into(),
            }
        }
    }

    impl FleetKeybindsConsumer for TestAppKeyConfig {
        fn from_keybinds(kb: &FleetKeybinds) -> Self {
            Self {
                history_picker: kb.history_picker.into(),
                clear_buffer: kb.clear_buffer.into(),
                ..Self::bare()
            }
        }
    }

    #[test]
    fn from_keybinds_pulls_canonical_chords() {
        let kb = FleetKeybinds::prescribed();
        let cfg = TestAppKeyConfig::from_keybinds(&kb);
        assert_eq!(cfg.history_picker, "C-r");
        assert_eq!(cfg.clear_buffer, "C-l");
    }

    #[test]
    fn from_keybinds_bare_yields_empty_chords() {
        let kb = FleetKeybinds::bare();
        let cfg = TestAppKeyConfig::from_keybinds(&kb);
        assert_eq!(cfg.history_picker, "");
        assert_eq!(cfg.clear_buffer, "");
    }

    #[test]
    fn app_specific_chords_inherit_from_bare() {
        let kb = FleetKeybinds::prescribed();
        let cfg = TestAppKeyConfig::from_keybinds(&kb);
        // app_specific_chord isn't in FleetKeybinds — comes from bare().
        assert_eq!(cfg.app_specific_chord, "C-z");
    }

    #[test]
    fn apply_to_helper_round_trips_to_from_keybinds() {
        let kb = FleetKeybinds::prescribed();
        let direct = TestAppKeyConfig::from_keybinds(&kb);
        let via_helper: TestAppKeyConfig = kb.apply_to();
        assert_eq!(direct, via_helper);
    }
}
