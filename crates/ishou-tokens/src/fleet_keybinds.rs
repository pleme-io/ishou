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
/// **GUI terminals** (mado / namimado / hibiki / …): `copy`,
/// `paste`, `select_all`, `search_open`, `search_close`,
/// `search_next`, `search_prev`, `font_increase`, `font_decrease`,
/// `font_reset`, `toggle_fullscreen`, `clear_screen`.
///
/// **TUIs** (pauta / banken / acervo / tanken / …): `select_next`,
/// `select_prev`, `confirm`, `back`, `quit`, `cycle_focus`,
/// `filter`.
///
/// The four surfaces are typed as [`KeybindSurface`], and chord
/// uniqueness is checked **per surface** rather than globally —
/// `escape` legitimately means "dismiss" on two of them.
///
/// New intents land here as they emerge fleet-wide. Single-app
/// chords stay in the app's own config; only chords with ≥2
/// consumers (or ones that operators expect to be fleet-canonical
/// like Ctrl-R) belong here. That rule is why a TUI `?`-for-help
/// intent is deliberately ABSENT: only kura binds it today, so it
/// is one app's chord, not a fleet convention.
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

    // ── GUI terminals (mado / namimado / escriba / …) ────────────
    //
    // Cmd-modifier chords using the atlas `D-` short-form (= cmd
    // on macOS, super elsewhere). Every cross-GUI-terminal intent
    // lands here so operator muscle memory carries across mado,
    // namimado, escriba, hibiki, hikki, fumi, kekkai, taimen. App-
    // specific chords (e.g. mado's tear-attach split nav) stay in
    // the app's own config and don't belong on the atlas.
    /// Copy selection to system clipboard. macOS Cmd-C / Linux Ctrl-C.
    pub copy: &'static str,
    /// Paste from system clipboard. Cmd-V / Ctrl-V.
    pub paste: &'static str,
    /// Select everything on the current screen.
    pub select_all: &'static str,
    /// Open the in-app search overlay.
    pub search_open: &'static str,
    /// Close the in-app search overlay (ESC, no modifier — same
    /// across every fleet GUI).
    pub search_close: &'static str,
    /// Jump to next search match.
    pub search_next: &'static str,
    /// Jump to previous search match.
    pub search_prev: &'static str,
    /// Increase font size one step.
    pub font_increase: &'static str,
    /// Decrease font size one step.
    pub font_decrease: &'static str,
    /// Reset font size to the prescribed default.
    pub font_reset: &'static str,
    /// Toggle fullscreen window state.
    pub toggle_fullscreen: &'static str,
    /// Clear the visible screen (alternate-screen-respecting).
    pub clear_screen: &'static str,

    // ── TUI surfaces (pauta / banken / acervo / tanken / kura /
    //    arnes / hikyaku / alicerce / mirante / hibiki) ────────────
    //
    // Bare, unmodified keys — a full-screen TUI owns the whole
    // keyboard, so it does not need (and cannot rely on) a modifier
    // to disambiguate from a shell or the host terminal's chrome.
    //
    // ★ These were NOT invented here. Every value below is what 3+
    // of the ten fleet TUIs already bind today; the atlas is
    // recording an existing consensus so the stragglers can be
    // brought to it, not imposing a new one. `confirm` is unanimous
    // across all ten. What convergence actually changes is narrow
    // and known: mirante binds no vi-keys at all (arrows only),
    // arnes overloads `escape` as quit because it has no navigation
    // stack, and alicerce collapses quit and back onto one action.
    //
    // Arrow keys are deliberately ABSENT. Eight of ten TUIs bind
    // `down`/`up` alongside `j`/`k`, but nobody disagrees about what
    // an arrow key does — it needs no fleet coordination, so it is
    // an app-level alias, not an atlas intent. The atlas exists for
    // chords operators could otherwise find in different places.
    /// Move the selection one row toward the end of the list.
    pub select_next: &'static str,
    /// Move the selection one row toward the start of the list.
    pub select_prev: &'static str,
    /// Act on the selected row (open / accept / drill in).
    pub confirm: &'static str,
    /// Dismiss the current overlay, or pop one level of navigation.
    /// NOT "quit" — an app whose `back` exits the process is the
    /// arnes deviation, and it costs the operator the ability to
    /// leave a modal without leaving the tool.
    pub back: &'static str,
    /// Leave the application.
    pub quit: &'static str,
    /// Cycle keyboard focus between panes / lanes / fields.
    pub cycle_focus: &'static str,
    /// Open incremental search / filter over the current list.
    /// Distinct from [`Self::search_open`], which is the GUI
    /// terminal's Cmd-F over rendered scrollback — different
    /// surface, different corpus, same word.
    pub filter: &'static str,
}

/// Which operator-facing surface an intent belongs to.
///
/// The atlas spans three surfaces that are never focused at once, and
/// that is load-bearing for conflict checking: a chord must be unique
/// *within* a surface, not globally. `escape` correctly means "close
/// the search overlay" in a GUI terminal and "go back" in a TUI —
/// forcing those apart would be modelling the keyboard wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum KeybindSurface {
    /// Interactive shell line editor (frost / frostmourne).
    Shell,
    /// Terminal multiplexer prefix space (tear).
    Multiplexer,
    /// GUI terminal chrome (mado / namimado / hibiki / …).
    GuiTerminal,
    /// Full-screen terminal UI (pauta / banken / acervo / …).
    Tui,
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
            // GUI terminal intents — bare = empty.
            copy: "",
            paste: "",
            select_all: "",
            search_open: "",
            search_close: "",
            search_next: "",
            search_prev: "",
            font_increase: "",
            font_decrease: "",
            font_reset: "",
            toggle_fullscreen: "",
            clear_screen: "",
            // TUI intents — bare = empty.
            select_next: "",
            select_prev: "",
            confirm: "",
            back: "",
            quit: "",
            cycle_focus: "",
            filter: "",
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
            // GUI terminals — Cmd-modifier (atlas D- short-form =
            // cmd on macOS, super elsewhere). Matches mado's
            // long-standing curated defaults + every other macOS
            // terminal convention (iTerm2, Terminal.app, ghostty).
            copy: "D-c",
            paste: "D-v",
            select_all: "D-a",
            search_open: "D-f",
            // search_close uses ESC alone (no modifier) — universal
            // muscle memory across every modal UI.
            search_close: "escape",
            search_next: "D-g",
            // search_prev: Cmd+Shift+G — atlas short-form puts shift
            // as a chained modifier prefix.
            search_prev: "D-S-g",
            font_increase: "D-=",
            font_decrease: "D--",
            font_reset: "D-0",
            // toggle_fullscreen: Cmd+Ctrl+F (macOS-native fullscreen
            // gesture).
            toggle_fullscreen: "D-C-f",
            // clear_screen: Cmd+K (iTerm2-canonical; ghostty-parity).
            clear_screen: "D-k",

            // TUI surfaces — bare keys, recording the consensus ten
            // fleet TUIs already share (see the struct comment for
            // the three apps that deviate and how).
            //
            // vi-keys for motion: bound by 8/10 today. mirante is the
            // one holdout and its absence is what breaks j/k muscle
            // memory across the fleet.
            select_next: "j",
            select_prev: "k",
            // The single unanimous intent — all ten TUIs, zero drift.
            // Spelled awase-canonical: `Hotkey::parse` accepts both
            // "return" and "enter", and `Key::Return.display()` emits
            // "return" (awase/src/hotkey.rs:242,346).
            confirm: "return",
            // Same awase-canonical spelling as `search_close` above,
            // and deliberately the SAME chord — see `KeybindSurface`:
            // dismiss-an-overlay is one gesture, and the surfaces are
            // never focused together.
            //
            // ★ TRAP for egaku consumers: `egaku_term` spells these
            // two keys "esc" and "enter", NOT "escape"/"return", so a
            // chord taken from this atlas must be translated before it
            // reaches `KeyCombo` — and `KeyCombo::key(s)` stores `s`
            // as a LITERAL key name with no parsing, so an untranslated
            // "escape" silently never matches. banken carries the
            // measured 2-row table (banken/src/keys.rs); it belongs in
            // one shared bridge, not per app.
            back: "escape",
            quit: "q",
            cycle_focus: "tab",
            // `/` — the list-filter convention, bound by tanken,
            // hibiki and acervo-ui today. pauta and banken have no
            // search intent at all, which is a coverage gap rather
            // than a conflict: nothing to reconcile, only to add.
            filter: "/",
        }
    }

    /// Every intent as `(surface, name, chord)`, in declaration order.
    ///
    /// The reflection surface conflict checks and diagnostics read,
    /// so neither has to re-list the struct by hand. The previous
    /// collision test *did* hand-list all 24 chords, which meant a
    /// newly-added field was silently unchecked until somebody
    /// remembered to add a row — the ★★ CATALOG REFLECTION failure
    /// this replaces.
    ///
    /// Adding a field to [`FleetKeybinds`] without adding its row
    /// here fails `every_field_appears_in_the_intent_catalog`.
    #[must_use]
    pub fn all_intents(&self) -> [(KeybindSurface, &'static str, &'static str); 32] {
        use KeybindSurface::{GuiTerminal, Multiplexer, Shell, Tui};
        [
            (Shell, "history_picker", self.history_picker),
            (Shell, "files_picker", self.files_picker),
            (Shell, "dir_picker", self.dir_picker),
            (Shell, "content_picker", self.content_picker),
            (Shell, "clear_buffer", self.clear_buffer),
            (Shell, "kill_line", self.kill_line),
            (Shell, "edit_in_editor", self.edit_in_editor),
            (Shell, "help", self.help),
            (Shell, "clipboard_copy", self.clipboard_copy),
            (Shell, "clipboard_paste", self.clipboard_paste),
            (Shell, "toggle_sudo", self.toggle_sudo),
            (Shell, "insert_last_arg", self.insert_last_arg),
            (Multiplexer, "multiplexer_prefix", self.multiplexer_prefix),
            (GuiTerminal, "copy", self.copy),
            (GuiTerminal, "paste", self.paste),
            (GuiTerminal, "select_all", self.select_all),
            (GuiTerminal, "search_open", self.search_open),
            (GuiTerminal, "search_close", self.search_close),
            (GuiTerminal, "search_next", self.search_next),
            (GuiTerminal, "search_prev", self.search_prev),
            (GuiTerminal, "font_increase", self.font_increase),
            (GuiTerminal, "font_decrease", self.font_decrease),
            (GuiTerminal, "font_reset", self.font_reset),
            (GuiTerminal, "toggle_fullscreen", self.toggle_fullscreen),
            (GuiTerminal, "clear_screen", self.clear_screen),
            (Tui, "select_next", self.select_next),
            (Tui, "select_prev", self.select_prev),
            (Tui, "confirm", self.confirm),
            (Tui, "back", self.back),
            (Tui, "quit", self.quit),
            (Tui, "cycle_focus", self.cycle_focus),
            (Tui, "filter", self.filter),
        ]
    }

    /// The `(name, chord)` pairs for one surface.
    ///
    /// A TUI wiring its keymap wants the [`KeybindSurface::Tui`] rows
    /// and nothing else — handing it `copy: "D-c"` would be noise it
    /// has no way to honour.
    #[must_use]
    pub fn intents_for(&self, surface: KeybindSurface) -> Vec<(&'static str, &'static str)> {
        self.all_intents()
            .into_iter()
            .filter(|(s, _, _)| *s == surface)
            .map(|(_, name, chord)| (name, chord))
            .collect()
    }
}

impl Default for FleetKeybinds {
    fn default() -> Self {
        Self::prescribed()
    }
}

/// Normalize an atlas chord string into the canonical long form.
///
/// The atlas declares chords in the concise emacs/tmux short-form
/// (`"C-r"`, `"M-c"`, `"C-x e"`) — this matches frost-lisp,
/// blackmatter-shell, blzsh, and every operator's muscle memory.
/// Consumers that need the explicit long form (`"ctrl+r"`, `"alt+c"`)
/// — tear's prefix-key matcher, awase's hotkey parser when it lands
/// — pass the atlas value through this helper to get the verbose
/// notation. The two forms are interchangeable at the consumer
/// layer; lifting the normalizer here means no consumer has to
/// reinvent it and every consumer agrees on the canonical mapping.
///
/// Mapping (idempotent — long-form input passes through unchanged):
///
/// - `C-` / `c-` → `ctrl+`
/// - `M-` / `m-` → `alt+`
/// - `S-` / `s-` → `shift+`
/// - `D-` / `d-` → `super+`
///
/// Multi-key chords (`"C-x e"`) keep their internal whitespace —
/// the normalization runs per space-separated chord segment, so
/// `"C-x e"` becomes `"ctrl+x e"`.
#[must_use]
pub fn normalize_chord(input: &str) -> String {
    input
        .split_whitespace()
        .map(normalize_single_chord)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_single_chord(chord: &str) -> String {
    // If the chord already contains '+' it's long-form; pass through
    // lowercased so case-only drift normalizes too.
    if chord.contains('+') {
        return chord.to_ascii_lowercase();
    }
    let segments: Vec<&str> = chord.split('-').collect();
    // Single-segment input is a bare key (`"tab"`, `"f10"`, `"r"`).
    if segments.len() == 1 {
        return segments[0].to_ascii_lowercase();
    }
    // Multi-segment input: the LAST segment is the key (regardless of
    // case — "C-c" means "ctrl+c", not "ctrl+ctrl"); every earlier
    // segment is a modifier letter.
    let key = segments.last().unwrap().to_ascii_lowercase();
    let modifiers = &segments[..segments.len() - 1];
    let mut parts: Vec<String> = modifiers
        .iter()
        .filter_map(|seg| match *seg {
            "C" | "c" => Some("ctrl"),
            "M" | "m" => Some("alt"),
            "S" | "s" => Some("shift"),
            "D" | "d" => Some("super"),
            _ => None,
        })
        .map(String::from)
        .collect();
    parts.push(key);
    parts.join("+")
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
        // GUI terminal intents.
        assert_eq!(b.copy, "");
        assert_eq!(b.paste, "");
        assert_eq!(b.select_all, "");
        assert_eq!(b.search_open, "");
        assert_eq!(b.search_close, "");
        assert_eq!(b.search_next, "");
        assert_eq!(b.search_prev, "");
        assert_eq!(b.font_increase, "");
        assert_eq!(b.font_decrease, "");
        assert_eq!(b.font_reset, "");
        assert_eq!(b.toggle_fullscreen, "");
        assert_eq!(b.clear_screen, "");
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
        // GUI terminals — Cmd-modifier chord set.
        assert_eq!(p.copy, "D-c");
        assert_eq!(p.paste, "D-v");
        assert_eq!(p.select_all, "D-a");
        assert_eq!(p.search_open, "D-f");
        assert_eq!(p.search_close, "escape");
        assert_eq!(p.search_next, "D-g");
        assert_eq!(p.search_prev, "D-S-g");
        assert_eq!(p.font_increase, "D-=");
        assert_eq!(p.font_decrease, "D--");
        assert_eq!(p.font_reset, "D-0");
        assert_eq!(p.toggle_fullscreen, "D-C-f");
        assert_eq!(p.clear_screen, "D-k");
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
    fn normalize_chord_short_form_expands_to_long_form() {
        assert_eq!(normalize_chord("C-r"), "ctrl+r");
        assert_eq!(normalize_chord("M-c"), "alt+c");
        assert_eq!(normalize_chord("S-tab"), "shift+tab");
        assert_eq!(normalize_chord("D-space"), "super+space");
    }

    #[test]
    fn normalize_chord_long_form_passes_through() {
        // Idempotent — feeding the long form back in keeps it stable
        // (with lowercase normalization).
        assert_eq!(normalize_chord("ctrl+r"), "ctrl+r");
        assert_eq!(normalize_chord("Ctrl+R"), "ctrl+r");
        assert_eq!(normalize_chord("alt+left"), "alt+left");
    }

    #[test]
    fn normalize_chord_multi_key_normalizes_each_segment() {
        // Two-key chords like `C-x e` keep their internal whitespace
        // but each segment normalizes independently.
        assert_eq!(normalize_chord("C-x e"), "ctrl+x e");
        assert_eq!(normalize_chord("C-x C-c"), "ctrl+x ctrl+c");
    }

    #[test]
    fn normalize_chord_empty_input_yields_empty() {
        assert_eq!(normalize_chord(""), "");
        assert_eq!(normalize_chord("   "), "");
    }

    #[test]
    fn normalize_chord_lone_key_no_modifier() {
        // Plain keys like `tab`, `f10` carry no modifier — the
        // normalizer should keep them intact.
        assert_eq!(normalize_chord("tab"), "tab");
        assert_eq!(normalize_chord("f10"), "f10");
    }

    #[test]
    fn normalize_chord_atlas_round_trip_matches_tear_form() {
        // The whole reason this lives in ishou-tokens: tear's
        // `KeyChord::from_tmux` produces exactly this. Pinning the
        // contract here means tear (and every future consumer) can
        // stop reinventing the wheel and route through atlas helpers.
        let atlas = FleetKeybinds::prescribed();
        assert_eq!(normalize_chord(atlas.multiplexer_prefix), "ctrl+b");
        assert_eq!(normalize_chord(atlas.history_picker), "ctrl+r");
        assert_eq!(normalize_chord(atlas.dir_picker), "alt+c");
        assert_eq!(normalize_chord(atlas.edit_in_editor), "ctrl+x e");
    }

    #[test]
    fn no_two_intents_collide_on_the_same_chord_within_a_surface() {
        // In prescribed(), every intent gets its own chord ON ITS OWN
        // SURFACE. A drift that double-binds (like the 2026-05-21
        // Ctrl-R incident) fails this the moment the atlas ships a
        // duplicate.
        //
        // Per-surface, not global: the surfaces are never focused
        // together, and collapsing them would forbid `escape` from
        // meaning "dismiss" in both a GUI terminal and a TUI — which
        // is the correct binding in each. See `KeybindSurface`.
        //
        // Reads the catalog rather than re-listing the struct, so a
        // field added without a catalog row is caught by
        // `every_field_appears_in_the_intent_catalog` instead of
        // silently escaping this check.
        let p = FleetKeybinds::prescribed();
        for surface in [
            KeybindSurface::Shell,
            KeybindSurface::Multiplexer,
            KeybindSurface::GuiTerminal,
            KeybindSurface::Tui,
        ] {
            let rows = p.intents_for(surface);
            let mut seen: Vec<&str> = rows.iter().map(|(_, chord)| *chord).collect();
            let before = seen.len();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(
                seen.len(),
                before,
                "FleetKeybinds::prescribed() double-binds a chord on {surface:?}: {rows:?}"
            );
        }
    }

    #[test]
    fn one_chord_may_mean_the_same_gesture_on_two_surfaces() {
        // Pinned deliberately, because it is the case that broke the
        // old global uniqueness check and the reason the model is
        // per-surface. `escape` dismisses the GUI terminal's search
        // overlay AND pops a TUI's navigation stack; those are the
        // same gesture on surfaces that are never focused at once.
        // If a future change forces these apart, that is a REGRESSION
        // in the model, not a conflict fixed.
        let p = FleetKeybinds::prescribed();
        assert_eq!(p.search_close, "escape");
        assert_eq!(p.back, "escape");
        assert_eq!(
            p.search_close, p.back,
            "the cross-surface dismiss gesture must stay one chord"
        );
    }

    #[test]
    fn every_field_appears_in_the_intent_catalog() {
        // ★★ CATALOG REFLECTION: adding a field without adding its
        // catalog row must fail here rather than silently escaping
        // the conflict check.
        //
        // `bare()` sets every field to "", so a field missing from
        // the catalog is invisible to a value-based scan. Instead the
        // catalog is checked against the serialized struct, which IS
        // derived from the field list by serde — the one view that
        // cannot drift from the real fields.
        let p = FleetKeybinds::prescribed();
        let json = serde_json::to_value(&p).expect("atlas serializes");
        let obj = json.as_object().expect("atlas is a struct");

        let catalog: Vec<&str> = p.all_intents().iter().map(|(_, name, _)| *name).collect();
        assert_eq!(
            obj.len(),
            catalog.len(),
            "FleetKeybinds has {} fields but all_intents() lists {} — add the missing catalog row",
            obj.len(),
            catalog.len()
        );
        for field in obj.keys() {
            assert!(
                catalog.contains(&field.as_str()),
                "field `{field}` is missing from all_intents()"
            );
        }
    }

    #[test]
    fn the_tui_surface_carries_the_intents_a_list_ui_needs() {
        let p = FleetKeybinds::prescribed();
        let tui = p.intents_for(KeybindSurface::Tui);
        let names: Vec<&str> = tui.iter().map(|(n, _)| *n).collect();
        for want in [
            "select_next",
            "select_prev",
            "confirm",
            "back",
            "quit",
            "cycle_focus",
            "filter",
        ] {
            assert!(names.contains(&want), "TUI surface is missing `{want}`");
        }
        // …and nothing from another surface leaked in. A TUI handed
        // `copy: "D-c"` has no way to honour it.
        assert!(!names.contains(&"copy"));
        assert!(!names.contains(&"multiplexer_prefix"));
    }

    #[test]
    fn tui_chords_are_bare_keys_not_modified_chords() {
        // A full-screen TUI owns the keyboard, so its intents must not
        // require a modifier. A modifier creeping in here is how an
        // intent stops being reachable in a plain terminal.
        let p = FleetKeybinds::prescribed();
        for (name, chord) in p.intents_for(KeybindSurface::Tui) {
            assert!(
                !chord.contains('+') && !chord.contains('-'),
                "TUI intent `{name}` has a modified chord `{chord}` — TUI chords are bare keys"
            );
        }
    }

    #[test]
    fn tui_chords_survive_normalization_unchanged() {
        // Bare keys must be fixpoints of normalize_chord: a consumer
        // that normalizes atlas chords (because the shell/GUI tiers
        // need it) must not have the TUI tier mangled on the way
        // through.
        let p = FleetKeybinds::prescribed();
        for (name, chord) in p.intents_for(KeybindSurface::Tui) {
            assert_eq!(
                normalize_chord(chord),
                chord,
                "TUI intent `{name}` is not a normalization fixpoint"
            );
        }
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
