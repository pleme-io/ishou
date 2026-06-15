//! Fleet signal vocabulary — the typed emoji-first communication
//! language of the pleme-io TUI world.
//!
//! Every fleet TUI (mado, tear, seki, frostmourne, namimado, escriba)
//! speaks the SAME emoji language for status, health, session, git, and
//! notification signals. Before this primitive each app hand-picked its
//! own glyphs — seki used `⇡⇣✘` for git, mado/tear shared `🌊❄` only by
//! coincidence, escriba had a bespoke icon set — so the operator saw a
//! different visual vocabulary in every pane. [`FleetSignals`] lifts the
//! vocabulary to one typed atlas (peer of [`FleetKeybinds`] for chords
//! and [`FleetDefaults`] for theme): touch it once, every consumer's
//! signal language moves together on the next compile.
//!
//! ## Width-aware by construction
//!
//! TUI emoji rendering has a hard constraint: most status emoji are
//! East-Asian-Wide (two cells), which misaligns a tight prompt segment
//! or a fixed-width status column. A naive "just use emoji" vocabulary
//! breaks seki's prompt the moment it adopts it. So every signal is a
//! [`Signal`] triple — an emoji-first primary, a single-width geometric
//! `glyph` fallback for tight layouts, and a plain-text `label` for
//! a11y / no-emoji terminals / logs. The consumer picks the
//! [`SignalMode`] that fits its width budget; the *vocabulary* (which
//! mark means "error") stays fleet-uniform regardless of mode.
//!
//! ```rust,ignore
//! use ishou_tokens::{FleetSignals, SignalMode};
//! let sig = FleetSignals::prescribed();
//! // status line / notification → full emoji
//! format!("{} build complete", sig.success.render(SignalMode::Emoji)); // ✅ build complete
//! // tight prompt segment → single-width glyph (no misalignment)
//! format!("{}{}", sig.git_ahead.render(SignalMode::Glyph), n);          // ⇡3
//! // no-emoji terminal / log → plain text
//! format!("[{}]", sig.error.render(SignalMode::Label));                 // [error]
//! ```
//!
//! ## Tiers
//!
//! Mirrors `shikumi::TieredConfig`: [`FleetSignals::bare`] is the
//! everything-off floor — empty emoji + glyph, `label` retained (a bare
//! app still communicates, just in plain text, never silence).
//! [`FleetSignals::prescribed`] is the fleet vocabulary. The
//! [`FleetSignalsConsumer`] trait + [`FleetSignals::apply_to`] let an
//! app derive its signal config mechanically, exactly like
//! [`FleetKeybindsConsumer`].

use serde::Serialize;

/// How a [`Signal`] renders into a string, chosen by the consumer's
/// width budget + terminal capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SignalMode {
    /// Emoji-first primary (may be two cells wide). Status lines,
    /// notifications, banners — anywhere width is flexible. This is the
    /// default the directive ("emoji-based communication") asks for.
    Emoji,
    /// Single-width geometric glyph. Prompt segments, fixed-width
    /// status columns, dense grids — anywhere a two-cell emoji would
    /// misalign the layout.
    Glyph,
    /// Plain-text label. No-emoji terminals, logs, screen readers, the
    /// bare tier.
    Label,
}

/// One fleet signal — an emoji-first mark with width-safe fallbacks.
///
/// The triple is `const`-constructible so the whole atlas is a `const`
/// value. A consumer renders the field that fits its surface via
/// [`Signal::render`]; the *meaning* (which mark = "error") is the same
/// across every fleet TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Signal {
    /// Emoji-first primary (e.g. `"✅"`). May be two cells wide.
    pub emoji: &'static str,
    /// Single-width geometric fallback (e.g. `"✓"`). Prompt/grid-safe.
    pub glyph: &'static str,
    /// Plain-text label (e.g. `"ok"`). a11y / no-emoji / logs.
    pub label: &'static str,
}

impl Signal {
    /// Construct a signal triple.
    #[must_use]
    pub const fn new(emoji: &'static str, glyph: &'static str, label: &'static str) -> Self {
        Self { emoji, glyph, label }
    }

    /// The empty signal — used by the bare tier for emoji + glyph while
    /// retaining the `label` so a bare app still communicates in text.
    #[must_use]
    pub const fn off(label: &'static str) -> Self {
        Self { emoji: "", glyph: "", label }
    }

    /// Render the field matching `mode`.
    #[must_use]
    pub fn render(&self, mode: SignalMode) -> &'static str {
        match mode {
            SignalMode::Emoji => self.emoji,
            SignalMode::Glyph => self.glyph,
            SignalMode::Label => self.label,
        }
    }

    /// Render `mode`, but fall back to the glyph (then the label) when
    /// the chosen field is empty — so a bare-tier emoji request still
    /// yields the label instead of an empty string, and a glyph-mode
    /// signal that has no compact glyph degrades to its emoji rather
    /// than vanishing. Never returns empty unless all three are empty.
    #[must_use]
    pub fn render_or_fallback(&self, mode: SignalMode) -> &'static str {
        let primary = self.render(mode);
        if !primary.is_empty() {
            return primary;
        }
        // Degradation order: emoji → glyph → label (most expressive that
        // is non-empty), so no surface ever shows nothing for a signal
        // that has *some* representation.
        for f in [self.emoji, self.glyph, self.label] {
            if !f.is_empty() {
                return f;
            }
        }
        ""
    }
}

/// The fleet emoji-signal vocabulary — one typed atlas every fleet TUI
/// consumes. Grouped by the communication categories that recur across
/// ≥2 tools (the bar for earning an atlas slot, per the org's
/// macros-everywhere / no-over-abstraction rule).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FleetSignals {
    // ── Outcome / status (every tool) ────────────────────────────
    /// An operation succeeded.
    pub success: Signal,
    /// An operation failed.
    pub error: Signal,
    /// A non-fatal warning.
    pub warning: Signal,
    /// Informational note.
    pub info: Signal,
    /// In-progress / working / spinner-anchor.
    pub working: Signal,
    /// A question / prompt for the operator.
    pub question: Signal,

    // ── Connectivity / health (mado, tear, namimado, seki fleet_node) ─
    /// Connected / healthy / up.
    pub online: Signal,
    /// Disconnected / down.
    pub offline: Signal,
    /// Degraded / partial / flaky.
    pub degraded: Signal,

    // ── Session / multiplexer (mado, tear, praça) ────────────────
    /// An attached, live session — the fleet `🌊` tide mark.
    pub session_active: Signal,
    /// A detached / sleeping session.
    pub session_detached: Signal,
    /// A zoomed / maximized pane.
    pub pane_zoomed: Signal,
    /// The multiplexer prefix key is armed (awaiting a command key).
    pub prefix_armed: Signal,

    // ── VCS / git (seki prompt, escriba, any repo-aware tool) ────
    /// Working tree clean.
    pub git_clean: Signal,
    /// Working tree has uncommitted changes.
    pub git_dirty: Signal,
    /// Local is ahead of upstream.
    pub git_ahead: Signal,
    /// Local is behind upstream.
    pub git_behind: Signal,
    /// Local and upstream have diverged.
    pub git_diverged: Signal,
    /// Staged changes present.
    pub git_staged: Signal,
    /// Merge conflict present.
    pub git_conflict: Signal,
    /// Stash entries present.
    pub git_stashed: Signal,

    // ── Notification (all) ───────────────────────────────────────
    /// Terminal bell / attention.
    pub bell: Signal,
    /// A delivered notification.
    pub notify: Signal,
    /// An urgent alert.
    pub alert: Signal,

    // ── Identity (the fleet mark) ────────────────────────────────
    /// The pleme-io fleet mark — Nord frost `❄`. Appears in mado's
    /// theme, seki's prompt, the fleet session palette.
    pub fleet_mark: Signal,
}

impl FleetSignals {
    /// **Tier 0 — bare**: emoji + glyph empty, `label` retained so a
    /// bare app still communicates in plain text. Matches the `bare()`
    /// everything-off contract of `shikumi::TieredConfig`.
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            success: Signal::off("ok"),
            error: Signal::off("error"),
            warning: Signal::off("warn"),
            info: Signal::off("info"),
            working: Signal::off("..."),
            question: Signal::off("?"),
            online: Signal::off("up"),
            offline: Signal::off("down"),
            degraded: Signal::off("degraded"),
            session_active: Signal::off("active"),
            session_detached: Signal::off("detached"),
            pane_zoomed: Signal::off("zoom"),
            prefix_armed: Signal::off("prefix"),
            git_clean: Signal::off("clean"),
            git_dirty: Signal::off("dirty"),
            git_ahead: Signal::off("ahead"),
            git_behind: Signal::off("behind"),
            git_diverged: Signal::off("diverged"),
            git_staged: Signal::off("staged"),
            git_conflict: Signal::off("conflict"),
            git_stashed: Signal::off("stash"),
            bell: Signal::off("bell"),
            notify: Signal::off("notify"),
            alert: Signal::off("alert"),
            fleet_mark: Signal::off("pleme"),
        }
    }

    /// **Tier 2 — prescribed**: the fleet emoji-signal vocabulary.
    /// Each `Signal` is `(emoji, single-width glyph, label)`. The glyph
    /// column is what seki's prompt + dense status columns adopt; the
    /// emoji column is what status lines + notifications adopt; the
    /// meaning is identical across both.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            // outcome
            success: Signal::new("✅", "✓", "ok"),
            error: Signal::new("❌", "✗", "error"),
            warning: Signal::new("⚠️", "▲", "warn"),
            info: Signal::new("ℹ️", "•", "info"),
            working: Signal::new("⏳", "…", "working"),
            question: Signal::new("❓", "?", "question"),
            // connectivity
            online: Signal::new("🟢", "●", "online"),
            offline: Signal::new("🔴", "○", "offline"),
            degraded: Signal::new("🟡", "◐", "degraded"),
            // session — 🌊 tide is the established active-session mark
            session_active: Signal::new("🌊", "≈", "active"),
            session_detached: Signal::new("💤", "z", "detached"),
            pane_zoomed: Signal::new("🔍", "⤢", "zoom"),
            prefix_armed: Signal::new("⌨️", "⌗", "prefix"),
            // git — glyph column matches seki's existing prompt vocabulary
            git_clean: Signal::new("✨", "✓", "clean"),
            git_dirty: Signal::new("📝", "✚", "dirty"),
            git_ahead: Signal::new("⬆️", "⇡", "ahead"),
            git_behind: Signal::new("⬇️", "⇣", "behind"),
            git_diverged: Signal::new("🔀", "⇕", "diverged"),
            git_staged: Signal::new("📦", "●", "staged"),
            git_conflict: Signal::new("💥", "✘", "conflict"),
            git_stashed: Signal::new("🗄️", "⚑", "stash"),
            // notification
            bell: Signal::new("🔔", "‼", "bell"),
            notify: Signal::new("📢", "✸", "notify"),
            alert: Signal::new("🚨", "!", "alert"),
            // identity — Nord frost
            fleet_mark: Signal::new("❄️", "❄", "pleme"),
        }
    }

    /// Every signal in field order — for tooling that iterates the
    /// vocabulary (catalog reflection, a `signals` MCP/CLI dump, drift
    /// tests). Returns `(name, &Signal)` pairs.
    #[must_use]
    pub fn all(&self) -> Vec<(&'static str, &Signal)> {
        vec![
            ("success", &self.success),
            ("error", &self.error),
            ("warning", &self.warning),
            ("info", &self.info),
            ("working", &self.working),
            ("question", &self.question),
            ("online", &self.online),
            ("offline", &self.offline),
            ("degraded", &self.degraded),
            ("session_active", &self.session_active),
            ("session_detached", &self.session_detached),
            ("pane_zoomed", &self.pane_zoomed),
            ("prefix_armed", &self.prefix_armed),
            ("git_clean", &self.git_clean),
            ("git_dirty", &self.git_dirty),
            ("git_ahead", &self.git_ahead),
            ("git_behind", &self.git_behind),
            ("git_diverged", &self.git_diverged),
            ("git_staged", &self.git_staged),
            ("git_conflict", &self.git_conflict),
            ("git_stashed", &self.git_stashed),
            ("bell", &self.bell),
            ("notify", &self.notify),
            ("alert", &self.alert),
            ("fleet_mark", &self.fleet_mark),
        ]
    }

    /// Materialize any [`FleetSignalsConsumer`] from this atlas. Mirrors
    /// [`FleetKeybinds::apply_to`](crate::FleetKeybinds::apply_to).
    #[must_use]
    pub fn apply_to<C: FleetSignalsConsumer>(&self) -> C {
        C::from_signals(self)
    }
}

impl Default for FleetSignals {
    fn default() -> Self {
        Self::prescribed()
    }
}

/// Marker + factory trait. Any pleme-io TUI config with signal/glyph
/// fields impls this so its prescribed tier is mechanically derived
/// from [`FleetSignals`] instead of hand-picking emoji. Peer of
/// [`FleetKeybindsConsumer`](crate::FleetKeybindsConsumer) and
/// [`FleetThemedConfig`](crate::FleetThemedConfig).
pub trait FleetSignalsConsumer: Sized {
    /// Compose a config instance from the fleet signal atlas. Touching
    /// [`FleetSignals`] propagates here on next compile — the compiler
    /// enforces the fleet vocabulary stays uniform.
    fn from_signals(signals: &FleetSignals) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_has_empty_emoji_and_glyph_but_keeps_label() {
        let b = FleetSignals::bare();
        for (name, sig) in b.all() {
            assert_eq!(sig.emoji, "", "{name}.emoji must be empty in bare");
            assert_eq!(sig.glyph, "", "{name}.glyph must be empty in bare");
            assert!(!sig.label.is_empty(), "{name}.label must survive bare");
        }
    }

    #[test]
    fn prescribed_has_a_full_triple_for_every_signal() {
        let p = FleetSignals::prescribed();
        for (name, sig) in p.all() {
            assert!(!sig.emoji.is_empty(), "{name}.emoji empty in prescribed");
            assert!(!sig.glyph.is_empty(), "{name}.glyph empty in prescribed");
            assert!(!sig.label.is_empty(), "{name}.label empty in prescribed");
        }
    }

    #[test]
    fn render_picks_the_field_for_the_mode() {
        let s = Signal::new("✅", "✓", "ok");
        assert_eq!(s.render(SignalMode::Emoji), "✅");
        assert_eq!(s.render(SignalMode::Glyph), "✓");
        assert_eq!(s.render(SignalMode::Label), "ok");
    }

    #[test]
    fn render_or_fallback_never_empties_when_some_field_is_set() {
        // A bare signal asked for emoji degrades to its label.
        let bare = Signal::off("error");
        assert_eq!(bare.render(SignalMode::Emoji), "");
        assert_eq!(bare.render_or_fallback(SignalMode::Emoji), "error");
        // A signal with no compact glyph degrades to emoji.
        let no_glyph = Signal::new("✅", "", "ok");
        assert_eq!(no_glyph.render_or_fallback(SignalMode::Glyph), "✅");
    }

    #[test]
    fn tide_is_the_active_session_mark() {
        // The established 🌊 tide convention (mado/tear/praça) is the
        // canonical active-session emoji — adoption is drift-free.
        assert_eq!(FleetSignals::prescribed().session_active.emoji, "🌊");
    }

    #[test]
    fn frost_is_the_fleet_mark() {
        assert_eq!(FleetSignals::prescribed().fleet_mark.glyph, "❄");
    }

    #[test]
    fn glyph_column_is_single_codepoint_width_safe() {
        // The compact glyph column must be prompt-safe — each glyph is a
        // single char (one scalar), so it can't be a two-cell emoji
        // sequence that misaligns a fixed-width segment.
        for (name, sig) in FleetSignals::prescribed().all() {
            assert_eq!(
                sig.glyph.chars().count(),
                1,
                "{name}.glyph must be a single scalar for width safety, got {:?}",
                sig.glyph
            );
        }
    }

    #[test]
    fn apply_to_materializes_a_consumer() {
        struct PromptGlyphs {
            ahead: &'static str,
            behind: &'static str,
        }
        impl FleetSignalsConsumer for PromptGlyphs {
            fn from_signals(s: &FleetSignals) -> Self {
                Self {
                    ahead: s.git_ahead.render(SignalMode::Glyph),
                    behind: s.git_behind.render(SignalMode::Glyph),
                }
            }
        }
        let p: PromptGlyphs = FleetSignals::prescribed().apply_to();
        assert_eq!(p.ahead, "⇡");
        assert_eq!(p.behind, "⇣");
    }
}
