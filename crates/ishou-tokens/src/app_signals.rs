//! Per-software signal sets — a correct, curated emoji vocabulary for
//! every fleet tool.
//!
//! [`FleetSignals`] (the sibling module) is the SHARED communication
//! vocabulary every TUI speaks (status / git / session / notify). This
//! module adds the layer the operator asked for — "a correct set for
//! every software we have": one curated set per fleet application, each
//! **composing** the shared [`FleetSignals`] base (so every app still
//! speaks the common language) **plus** the signals specific to that
//! app's domain (a terminal's bell + link + zoom; a prompt's
//! language/runtime icons; an editor's modes; a browser's tab + lock).
//!
//! Centralized here — not scattered across the app repos — for the same
//! reason [`FleetKeybinds`](crate::FleetKeybinds) keeps per-app chord
//! sections in one atlas: the fleet emoji vocabulary is *seen and
//! curated together*, so a tool never drifts onto a private symbol for a
//! concept another tool already names. Each app consumes its set via a
//! `prescribed()` default (the shikumi `TieredConfig` shape) and may
//! override per-field.
//!
//! Layering, top to bottom:
//! ```text
//! ishou-emoji (ALL Unicode emoji, generated)   ← pull any emoji by name
//!        │
//! FleetSignals (shared, curated, width-aware)  ← the common language
//!        │
//! app_signals::<App>Signals (this module)      ← per-software curation
//! ```
//!
//! Each app-specific [`Signal`] is the same `{ emoji, glyph, label }`
//! triple — emoji-first for status lines, single-width `glyph` for
//! prompts/dense grids, `label` for a11y/no-emoji terminals.

use crate::fleet_signals::{FleetSignals, Signal};

/// **mado** — the GPU terminal emulator. Shared base + terminal-surface
/// signals (bell/copy/search live in [`FleetSignals`]; these are the
/// terminal-specific extras).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MadoSignals {
    /// The shared fleet vocabulary (status, git, session, notify, …).
    pub fleet: FleetSignals,
    /// A clickable URL / OSC-8 hyperlink in the grid.
    pub link: Signal,
    /// Selection copied to the clipboard.
    pub clipboard: Signal,
    /// In-grid search overlay open / a live match.
    pub search: Signal,
    /// An inline image (kitty / iTerm / sixel) placement.
    pub image: Signal,
    /// Font zoom (Cmd-=/Cmd--) active.
    pub zoom: Signal,
    /// Fullscreen window state.
    pub fullscreen: Signal,
    /// Ambient effect layer (snow / aurora) — the fleet frost mood.
    pub ambience: Signal,
}

impl MadoSignals {
    /// Bare tier — shared base bare + empty terminal extras (labels kept).
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            fleet: FleetSignals::bare(),
            link: Signal::off("link"),
            clipboard: Signal::off("copied"),
            search: Signal::off("search"),
            image: Signal::off("image"),
            zoom: Signal::off("zoom"),
            fullscreen: Signal::off("fullscreen"),
            ambience: Signal::off("ambience"),
        }
    }

    /// Prescribed tier — the fleet base + curated terminal signals.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            fleet: FleetSignals::prescribed(),
            link: Signal::new("🔗", "↗", "link"),
            clipboard: Signal::new("📋", "⎘", "copied"),
            search: Signal::new("🔍", "/", "search"),
            image: Signal::new("🖼️", "▦", "image"),
            zoom: Signal::new("🔎", "⊕", "zoom"),
            fullscreen: Signal::new("⛶", "⤢", "fullscreen"),
            ambience: Signal::new("❄️", "❄", "ambience"),
        }
    }
}

impl Default for MadoSignals {
    fn default() -> Self {
        Self::prescribed()
    }
}

/// **tear** — the terminal multiplexer. Shared base (session/pane/prefix
/// already live in [`FleetSignals`]) + the layout/window signals tear
/// adds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TearSignals {
    /// The shared fleet vocabulary.
    pub fleet: FleetSignals,
    /// A window (a tab / group of panes).
    pub window: Signal,
    /// A horizontal split.
    pub split_horizontal: Signal,
    /// A vertical split.
    pub split_vertical: Signal,
    /// Input synchronized across panes (send-to-all).
    pub sync_panes: Signal,
    /// Copy-mode / scrollback navigation active.
    pub copy_mode: Signal,
}

impl TearSignals {
    /// Bare tier.
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            fleet: FleetSignals::bare(),
            window: Signal::off("window"),
            split_horizontal: Signal::off("split-h"),
            split_vertical: Signal::off("split-v"),
            sync_panes: Signal::off("sync"),
            copy_mode: Signal::off("copy-mode"),
        }
    }

    /// Prescribed tier.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            fleet: FleetSignals::prescribed(),
            window: Signal::new("🪟", "▦", "window"),
            split_horizontal: Signal::new("⬓", "─", "split-h"),
            split_vertical: Signal::new("⬔", "│", "split-v"),
            sync_panes: Signal::new("🔗", "⇉", "sync"),
            copy_mode: Signal::new("📜", "↕", "copy-mode"),
        }
    }
}

impl Default for TearSignals {
    fn default() -> Self {
        Self::prescribed()
    }
}

/// **seki** — the prompt renderer (starship fork). Shared base (the git
/// signals live in [`FleetSignals`], sourced verbatim) + the
/// language/runtime + shell-context icons a prompt surfaces. The glyph
/// column is prompt-safe single-width by construction.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SekiSignals {
    /// The shared fleet vocabulary (incl. the canonical git glyphs).
    pub fleet: FleetSignals,
    /// Rust toolchain / crate context.
    pub lang_rust: Signal,
    /// A Nix shell / flake dev environment.
    pub lang_nix: Signal,
    /// Python venv / interpreter.
    pub lang_python: Signal,
    /// Node.js / npm context.
    pub lang_node: Signal,
    /// Go module context.
    pub lang_go: Signal,
    /// A container / Docker context.
    pub container: Signal,
    /// A Kubernetes context.
    pub kubernetes: Signal,
    /// Command duration / a slow command.
    pub duration: Signal,
    /// Background jobs running.
    pub jobs: Signal,
    /// Battery / power state.
    pub battery: Signal,
    /// The current working directory segment.
    pub directory: Signal,
}

impl SekiSignals {
    /// Bare tier.
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            fleet: FleetSignals::bare(),
            lang_rust: Signal::off("rust"),
            lang_nix: Signal::off("nix"),
            lang_python: Signal::off("python"),
            lang_node: Signal::off("node"),
            lang_go: Signal::off("go"),
            container: Signal::off("container"),
            kubernetes: Signal::off("k8s"),
            duration: Signal::off("took"),
            jobs: Signal::off("jobs"),
            battery: Signal::off("battery"),
            directory: Signal::off("dir"),
        }
    }

    /// Prescribed tier — the language/runtime icon set seki's modules
    /// render. The single-width `glyph` column keeps a multi-segment
    /// prompt aligned; the emoji column is for a status-line variant.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            fleet: FleetSignals::prescribed(),
            lang_rust: Signal::new("🦀", "⊿", "rust"),
            lang_nix: Signal::new("❄️", "❄", "nix"),
            lang_python: Signal::new("🐍", "§", "python"),
            lang_node: Signal::new("⬢", "⬢", "node"),
            lang_go: Signal::new("🐹", "ʕ", "go"),
            container: Signal::new("🐳", "▢", "container"),
            kubernetes: Signal::new("☸️", "☸", "k8s"),
            duration: Signal::new("⏱️", "⏲", "took"),
            jobs: Signal::new("⚙️", "⚙", "jobs"),
            battery: Signal::new("🔋", "▮", "battery"),
            directory: Signal::new("📁", "▸", "dir"),
        }
    }
}

impl Default for SekiSignals {
    fn default() -> Self {
        Self::prescribed()
    }
}

/// **frostmourne / frost** — the reedline shell distribution. Its prompt
/// IS seki's output and its chords come from
/// [`FleetKeybinds`](crate::FleetKeybinds), so it inherits most of its
/// vocabulary; these are the shell-line signals it adds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FrostmourneSignals {
    /// The shared fleet vocabulary.
    pub fleet: FleetSignals,
    /// The seki prompt vocabulary it renders (language/runtime icons).
    pub prompt: SekiSignals,
    /// A history-search picker (Ctrl-R).
    pub history: Signal,
    /// An abbreviation / alias expansion fired.
    pub abbreviation: Signal,
    /// A completion menu open.
    pub completion: Signal,
}

impl FrostmourneSignals {
    /// Bare tier.
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            fleet: FleetSignals::bare(),
            prompt: SekiSignals::bare(),
            history: Signal::off("history"),
            abbreviation: Signal::off("abbr"),
            completion: Signal::off("complete"),
        }
    }

    /// Prescribed tier.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            fleet: FleetSignals::prescribed(),
            prompt: SekiSignals::prescribed(),
            history: Signal::new("🕐", "↺", "history"),
            abbreviation: Signal::new("✨", "⌁", "abbr"),
            completion: Signal::new("📑", "≡", "complete"),
        }
    }
}

impl Default for FrostmourneSignals {
    fn default() -> Self {
        Self::prescribed()
    }
}

/// **namimado** — the browser (TUI + GPU). Shared base + the browser
/// chrome signals.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NamimadoSignals {
    /// The shared fleet vocabulary.
    pub fleet: FleetSignals,
    /// A browser tab.
    pub tab: Signal,
    /// A bookmark.
    pub bookmark: Signal,
    /// Navigate back.
    pub back: Signal,
    /// Navigate forward.
    pub forward: Signal,
    /// Reload the page.
    pub reload: Signal,
    /// A secure (TLS) origin.
    pub secure: Signal,
    /// An insecure origin.
    pub insecure: Signal,
    /// A download in progress / complete.
    pub download: Signal,
    /// The home page.
    pub home: Signal,
}

impl NamimadoSignals {
    /// Bare tier.
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            fleet: FleetSignals::bare(),
            tab: Signal::off("tab"),
            bookmark: Signal::off("bookmark"),
            back: Signal::off("back"),
            forward: Signal::off("forward"),
            reload: Signal::off("reload"),
            secure: Signal::off("secure"),
            insecure: Signal::off("insecure"),
            download: Signal::off("download"),
            home: Signal::off("home"),
        }
    }

    /// Prescribed tier.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            fleet: FleetSignals::prescribed(),
            tab: Signal::new("🗂️", "▭", "tab"),
            bookmark: Signal::new("🔖", "★", "bookmark"),
            back: Signal::new("⬅️", "←", "back"),
            forward: Signal::new("➡️", "→", "forward"),
            reload: Signal::new("🔄", "↻", "reload"),
            secure: Signal::new("🔒", "⚿", "secure"),
            insecure: Signal::new("🔓", "⚠", "insecure"),
            download: Signal::new("⬇️", "↓", "download"),
            home: Signal::new("🏠", "⌂", "home"),
        }
    }
}

impl Default for NamimadoSignals {
    fn default() -> Self {
        Self::prescribed()
    }
}

/// **escriba** — the modal text editor. Shared base + the editor signals
/// (modes, save state, lsp, file).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EscribaSignals {
    /// The shared fleet vocabulary.
    pub fleet: FleetSignals,
    /// Normal (command) mode.
    pub mode_normal: Signal,
    /// Insert mode.
    pub mode_insert: Signal,
    /// Visual (selection) mode.
    pub mode_visual: Signal,
    /// Command-line mode.
    pub mode_command: Signal,
    /// A saved buffer.
    pub saved: Signal,
    /// An unsaved / modified buffer.
    pub modified: Signal,
    /// A read-only buffer.
    pub readonly: Signal,
    /// An LSP server attached / active.
    pub lsp: Signal,
    /// A folded region.
    pub fold: Signal,
}

impl EscribaSignals {
    /// Bare tier.
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            fleet: FleetSignals::bare(),
            mode_normal: Signal::off("normal"),
            mode_insert: Signal::off("insert"),
            mode_visual: Signal::off("visual"),
            mode_command: Signal::off("command"),
            saved: Signal::off("saved"),
            modified: Signal::off("modified"),
            readonly: Signal::off("readonly"),
            lsp: Signal::off("lsp"),
            fold: Signal::off("fold"),
        }
    }

    /// Prescribed tier.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            fleet: FleetSignals::prescribed(),
            mode_normal: Signal::new("🟦", "◆", "normal"),
            mode_insert: Signal::new("🟩", "▸", "insert"),
            mode_visual: Signal::new("🟪", "▮", "visual"),
            mode_command: Signal::new("🟨", ":", "command"),
            saved: Signal::new("💾", "✓", "saved"),
            modified: Signal::new("📝", "●", "modified"),
            readonly: Signal::new("🔒", "⚿", "readonly"),
            lsp: Signal::new("⚙️", "⚙", "lsp"),
            fold: Signal::new("📁", "▸", "fold"),
        }
    }
}

impl Default for EscribaSignals {
    fn default() -> Self {
        Self::prescribed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fleet_signals::SignalMode;

    #[test]
    fn every_app_set_embeds_the_shared_fleet_vocabulary() {
        // The whole point: each app set carries the SAME FleetSignals so
        // a tool never loses the common language — `success`/`error`/git
        // are identical across every app.
        assert_eq!(MadoSignals::prescribed().fleet, FleetSignals::prescribed());
        assert_eq!(TearSignals::prescribed().fleet, FleetSignals::prescribed());
        assert_eq!(SekiSignals::prescribed().fleet, FleetSignals::prescribed());
        assert_eq!(
            FrostmourneSignals::prescribed().fleet,
            FleetSignals::prescribed()
        );
        assert_eq!(
            NamimadoSignals::prescribed().fleet,
            FleetSignals::prescribed()
        );
        assert_eq!(
            EscribaSignals::prescribed().fleet,
            FleetSignals::prescribed()
        );
    }

    #[test]
    fn frostmourne_inherits_sekis_prompt_vocabulary() {
        // frostmourne's prompt IS seki's output — the language icons must
        // be the SAME set, never a private fork.
        assert_eq!(
            FrostmourneSignals::prescribed().prompt,
            SekiSignals::prescribed()
        );
    }

    #[test]
    fn seki_language_glyphs_are_single_width_prompt_safe() {
        // A prompt segment can't afford a two-cell emoji misaligning the
        // line — every language glyph must be one scalar.
        let s = SekiSignals::prescribed();
        for (name, sig) in [
            ("rust", &s.lang_rust),
            ("nix", &s.lang_nix),
            ("python", &s.lang_python),
            ("node", &s.lang_node),
            ("go", &s.lang_go),
            ("k8s", &s.kubernetes),
            ("container", &s.container),
        ] {
            assert_eq!(
                sig.glyph.chars().count(),
                1,
                "{name} prompt glyph must be single-width, got {:?}",
                sig.glyph
            );
        }
    }

    #[test]
    fn nix_reuses_the_fleet_frost_mark() {
        // The fleet frost ❄ is the Nix mark in the prompt — consistent
        // with FleetSignals.fleet_mark and mado's ambience.
        assert_eq!(SekiSignals::prescribed().lang_nix.glyph, "❄");
        assert_eq!(MadoSignals::prescribed().ambience.glyph, "❄");
    }

    #[test]
    fn bare_app_sets_keep_labels_but_empty_emoji() {
        let m = MadoSignals::bare();
        assert_eq!(m.link.emoji, "");
        assert_eq!(m.link.glyph, "");
        assert_eq!(m.link.label, "link");
    }

    #[test]
    fn app_specific_glyphs_render_by_mode() {
        let n = NamimadoSignals::prescribed();
        assert_eq!(n.secure.render(SignalMode::Emoji), "🔒");
        assert_eq!(n.back.render(SignalMode::Glyph), "←");
        assert_eq!(n.reload.render(SignalMode::Label), "reload");
    }
}
