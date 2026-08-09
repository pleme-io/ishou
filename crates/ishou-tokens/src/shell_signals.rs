//! Shell error signals — frost-and-snow Nord errors with a little
//! Brazilian warmth.
//!
//! When a shell command fails — `command not found`, `permission
//! denied`, `no such file or directory`, a signal kill, a frozen pipe —
//! the fleet shells (frost, frostmourne) and the tools that run commands
//! (escriba, tear panes) shouldn't bark a cold grey error. They should
//! say it in the fleet's voice: **Nord frost for *what* froze up**, with
//! **a little Brazilian warmth for "tudo bem, we're still here."** A
//! `❄️` for the chill of the miss, a `🌊` maré / `☀️` sol for the warmth
//! that says it's okay and the prompt is ready again.
//!
//! This is the error-state layer of the fleet emoji language (peer of
//! [`FleetSignals`](crate::FleetSignals) for status and the per-app
//! [`app_signals`](crate::app_signals)). One typed vocabulary, sourced
//! once here, consumed by every shell + command-runner so a failing
//! command looks the same warm-frost way in mado, frostmourne, escriba,
//! and tear.
//!
//! ## The blend
//!
//! Each [`ShellSignal`] class carries a [`Signal`] whose **emoji** is the
//! frost/snow mark for that failure, plus a single-width Nord **glyph**
//! for tight status columns and a plain **label** for logs / no-emoji
//! terminals. Layered on top is one shared **warmth** accent — the
//! Brazilian sol/maré — that a consumer can append for the "it's okay,
//! warm prompt's back" coda via [`ShellSignals::render_warm`].
//!
//! ```rust,ignore
//! use ishou_tokens::{ShellSignals, ShellSignal, SignalMode};
//! let s = ShellSignals::prescribed();
//! // frost prints this on a missing command:
//! format!("{} command not found: git", s.signal(ShellSignal::CommandNotFound).render(SignalMode::Emoji));
//! //  ❄️ command not found: git
//! // …and the warm-frost variant for a friendlier shell:
//! s.render_warm(ShellSignal::CommandNotFound, SignalMode::Emoji); // "❄️ … 🌊"
//! ```

use crate::fleet_signals::{Signal, SignalMode};

/// A shell failure class — the typed border the shells map their errors
/// onto. Closed enum: every shell error resolves to exactly one of these
/// (an unmapped errno/exit code folds to [`ShellSignal::General`], never
/// an ad-hoc string).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum ShellSignal {
    /// `command not found` (exit 127 / ENOENT on exec). The classic.
    CommandNotFound,
    /// `permission denied` (EACCES / exit 126).
    PermissionDenied,
    /// `no such file or directory` (ENOENT on a path).
    NoSuchFile,
    /// `is a directory` (EISDIR).
    IsADirectory,
    /// `not a directory` (ENOTDIR).
    NotADirectory,
    /// The command was interrupted / signal-killed (SIGINT/SIGTERM,
    /// exit 130/143).
    Interrupted,
    /// The command crashed — segfault / abort (SIGSEGV/SIGABRT).
    Crashed,
    /// A pipe failed (a *frozen pipe* — the frost pun writes itself).
    PipeFailed,
    /// A parse / syntax error in the command line.
    SyntaxError,
    /// `no such builtin` (a frost builtin that doesn't exist).
    NoSuchBuiltin,
    /// The command timed out (frozen in time).
    TimedOut,
    /// `no space left on device` (ENOSPC).
    DiskFull,
    /// `exec format error` (ENOEXEC) — not a runnable binary.
    ExecFormat,
    /// Any other non-zero failure with no more specific class.
    General,
}

impl ShellSignal {
    /// Every class, declaration order — for tooling that iterates the
    /// vocabulary (a `shell-signals` dump, drift tests, docs).
    pub const ALL: &'static [ShellSignal] = &[
        ShellSignal::CommandNotFound,
        ShellSignal::PermissionDenied,
        ShellSignal::NoSuchFile,
        ShellSignal::IsADirectory,
        ShellSignal::NotADirectory,
        ShellSignal::Interrupted,
        ShellSignal::Crashed,
        ShellSignal::PipeFailed,
        ShellSignal::SyntaxError,
        ShellSignal::NoSuchBuiltin,
        ShellSignal::TimedOut,
        ShellSignal::DiskFull,
        ShellSignal::ExecFormat,
        ShellSignal::General,
    ];

    /// Map a raw libc errno (e.g. from a failed `exec`/`open`) to its
    /// shell-signal class. Unknown errno → [`ShellSignal::General`].
    /// The common POSIX values are wired; this is the bridge frost-exec
    /// uses to turn a `nix::errno::Errno` into a warm-frost mark.
    #[must_use]
    pub fn from_errno(errno: i32) -> Self {
        // Canonical Linux/Darwin-shared errno numbers.
        match errno {
            1 => ShellSignal::PermissionDenied,  // EPERM
            2 => ShellSignal::NoSuchFile,        // ENOENT
            8 => ShellSignal::ExecFormat,        // ENOEXEC
            13 => ShellSignal::PermissionDenied, // EACCES
            20 => ShellSignal::NotADirectory,    // ENOTDIR
            21 => ShellSignal::IsADirectory,     // EISDIR
            28 => ShellSignal::DiskFull,         // ENOSPC
            _ => ShellSignal::General,
        }
    }

    /// Map a shell exit code to a class for the cases an exit code alone
    /// is decisive: 126 (not executable / permission), 127 (command not
    /// found), 130 (SIGINT), 137 (SIGKILL), 139 (SIGSEGV), 143 (SIGTERM).
    /// A non-decisive non-zero code → [`ShellSignal::General`]; `0` (no
    /// failure) also folds to `General` (callers gate on `code != 0`).
    #[must_use]
    pub fn from_exit_code(code: i32) -> Self {
        match code {
            126 => ShellSignal::PermissionDenied,
            127 => ShellSignal::CommandNotFound,
            130 | 143 => ShellSignal::Interrupted, // 128+SIGINT / 128+SIGTERM
            134 | 139 => ShellSignal::Crashed,     // 128+SIGABRT / 128+SIGSEGV
            _ => ShellSignal::General,
        }
    }
}

/// The fleet shell-error vocabulary — one typed atlas the shells consume.
/// Frost emoji per failure class + a shared Brazilian-warmth accent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ShellSignals {
    command_not_found: Signal,
    permission_denied: Signal,
    no_such_file: Signal,
    is_a_directory: Signal,
    not_a_directory: Signal,
    interrupted: Signal,
    crashed: Signal,
    pipe_failed: Signal,
    syntax_error: Signal,
    no_such_builtin: Signal,
    timed_out: Signal,
    disk_full: Signal,
    exec_format: Signal,
    general: Signal,
    /// The Brazilian-warmth accent — the `🌊` maré / `~` that says "tudo
    /// bem, the warm prompt is back." Appended by [`Self::render_warm`].
    warmth: Signal,
}

impl ShellSignals {
    /// **Tier 0 — bare**: emoji + glyph empty, labels kept so a bare
    /// shell still names the failure in plain text (never silence).
    /// Matches the `bare()` everything-off contract of
    /// `shikumi::TieredConfig`.
    #[must_use]
    pub const fn bare() -> Self {
        Self {
            command_not_found: Signal::off("command not found"),
            permission_denied: Signal::off("permission denied"),
            no_such_file: Signal::off("no such file or directory"),
            is_a_directory: Signal::off("is a directory"),
            not_a_directory: Signal::off("not a directory"),
            interrupted: Signal::off("interrupted"),
            crashed: Signal::off("crashed"),
            pipe_failed: Signal::off("pipe failed"),
            syntax_error: Signal::off("syntax error"),
            no_such_builtin: Signal::off("no such builtin"),
            timed_out: Signal::off("timed out"),
            disk_full: Signal::off("no space left"),
            exec_format: Signal::off("exec format error"),
            general: Signal::off("error"),
            warmth: Signal::off(""),
        }
    }

    /// **Tier 2 — prescribed**: the warm-frost vocabulary. Each `Signal`
    /// is `(frost emoji, single-width Nord glyph, label)`. The emoji is
    /// the chill of *what* froze; the [`warmth`](Self::render_warm) accent
    /// is the Brazilian sol/maré coda.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            // A snowflake that fell where the command should be — gone, frozen.
            command_not_found: Signal::new("❄️", "✕", "command not found"),
            // Frost-locked.
            permission_denied: Signal::new("🔒", "⊘", "permission denied"),
            // The path is snowed over / buried.
            no_such_file: Signal::new("🌨️", "∅", "no such file or directory"),
            is_a_directory: Signal::new("📁", "⊟", "is a directory"),
            not_a_directory: Signal::new("📄", "⊞", "not a directory"),
            // A storm cut it short.
            interrupted: Signal::new("🌩️", "↯", "interrupted"),
            // Shattered ice.
            crashed: Signal::new("🧊", "✺", "crashed"),
            // A FROZEN PIPE — the frost pun the whole vocabulary earns.
            pipe_failed: Signal::new("🚰", "¦", "pipe failed"),
            // Twisted / tangled.
            syntax_error: Signal::new("🌀", "⌇", "syntax error"),
            no_such_builtin: Signal::new("❄️", "✕", "no such builtin"),
            // Frozen in time.
            timed_out: Signal::new("⏳", "⧖", "timed out"),
            // A full mountain of snow.
            disk_full: Signal::new("🏔️", "▰", "no space left"),
            exec_format: Signal::new("🧩", "⊠", "exec format error"),
            general: Signal::new("❄️", "!", "error"),
            // 🌊 maré — the established fleet tide mark, Brazilian beach
            // warmth: it's okay, the warm prompt is back.
            warmth: Signal::new("🌊", "~", "ok"),
        }
    }

    /// The [`Signal`] for a failure class.
    #[must_use]
    pub fn signal(&self, class: ShellSignal) -> &Signal {
        match class {
            ShellSignal::CommandNotFound => &self.command_not_found,
            ShellSignal::PermissionDenied => &self.permission_denied,
            ShellSignal::NoSuchFile => &self.no_such_file,
            ShellSignal::IsADirectory => &self.is_a_directory,
            ShellSignal::NotADirectory => &self.not_a_directory,
            ShellSignal::Interrupted => &self.interrupted,
            ShellSignal::Crashed => &self.crashed,
            ShellSignal::PipeFailed => &self.pipe_failed,
            ShellSignal::SyntaxError => &self.syntax_error,
            ShellSignal::NoSuchBuiltin => &self.no_such_builtin,
            ShellSignal::TimedOut => &self.timed_out,
            ShellSignal::DiskFull => &self.disk_full,
            ShellSignal::ExecFormat => &self.exec_format,
            ShellSignal::General => &self.general,
        }
    }

    /// The Brazilian-warmth accent signal (🌊 maré).
    #[must_use]
    pub fn warmth(&self) -> &Signal {
        &self.warmth
    }

    /// Render just the failure mark for `class` in `mode` — the frost
    /// emoji/glyph/label, no warmth. This is the common shell prefix:
    /// `<mark> command not found: git`.
    #[must_use]
    pub fn render(&self, class: ShellSignal, mode: SignalMode) -> &'static str {
        self.signal(class).render_or_fallback(mode)
    }

    /// Render the warm-frost pairing — the frost mark, the message space,
    /// and a trailing Brazilian-warmth accent — as a `(prefix, suffix)`
    /// the consumer wraps the message in:
    /// `format!("{prefix} {msg} {suffix}")` → `❄️ command not found: git 🌊`.
    /// In a tier/mode where warmth is empty, the suffix is `""` (the
    /// pairing degrades to a plain frost prefix — never a dangling space
    /// the caller must trim, because the caller appends only a non-empty
    /// suffix).
    #[must_use]
    pub fn render_warm(
        &self,
        class: ShellSignal,
        mode: SignalMode,
    ) -> (&'static str, &'static str) {
        (
            self.signal(class).render_or_fallback(mode),
            self.warmth.render(mode),
        )
    }
}

impl Default for ShellSignals {
    fn default() -> Self {
        Self::prescribed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_class_has_a_signal_in_both_tiers() {
        let bare = ShellSignals::bare();
        let pre = ShellSignals::prescribed();
        for class in ShellSignal::ALL {
            // bare: emoji+glyph empty, label survives.
            let b = bare.signal(*class);
            assert_eq!(b.emoji, "", "{class:?} bare emoji must be empty");
            assert_eq!(b.glyph, "", "{class:?} bare glyph must be empty");
            assert!(!b.label.is_empty(), "{class:?} bare label must survive");
            // prescribed: a full triple.
            let p = pre.signal(*class);
            assert!(!p.emoji.is_empty(), "{class:?} prescribed emoji empty");
            assert!(!p.glyph.is_empty(), "{class:?} prescribed glyph empty");
            assert!(!p.label.is_empty(), "{class:?} prescribed label empty");
        }
    }

    #[test]
    fn command_not_found_is_a_lost_snowflake() {
        let s = ShellSignals::prescribed();
        assert_eq!(
            s.render(ShellSignal::CommandNotFound, SignalMode::Emoji),
            "❄️"
        );
        assert_eq!(
            s.render(ShellSignal::CommandNotFound, SignalMode::Label),
            "command not found"
        );
    }

    #[test]
    fn pipe_failure_is_a_frozen_pipe() {
        // The frost pun that justifies the whole vocabulary.
        assert_eq!(
            ShellSignals::prescribed().render(ShellSignal::PipeFailed, SignalMode::Emoji),
            "🚰"
        );
    }

    #[test]
    fn warmth_is_the_brazilian_tide() {
        // 🌊 maré — the fleet's warm "tudo bem" accent, consistent with
        // the established tide session mark.
        assert_eq!(ShellSignals::prescribed().warmth().emoji, "🌊");
    }

    #[test]
    fn render_warm_pairs_frost_with_warmth() {
        let (prefix, suffix) =
            ShellSignals::prescribed().render_warm(ShellSignal::CommandNotFound, SignalMode::Emoji);
        assert_eq!(prefix, "❄️");
        assert_eq!(suffix, "🌊");
        // bare → both degrade: prefix is the label (render_or_fallback),
        // suffix is empty (warmth has no label set), so no dangling accent.
        let (bp, bs) =
            ShellSignals::bare().render_warm(ShellSignal::CommandNotFound, SignalMode::Emoji);
        assert_eq!(bp, "command not found");
        assert_eq!(bs, "");
    }

    #[test]
    fn errno_maps_to_the_right_frost_class() {
        assert_eq!(ShellSignal::from_errno(2), ShellSignal::NoSuchFile); // ENOENT
        assert_eq!(ShellSignal::from_errno(13), ShellSignal::PermissionDenied); // EACCES
        assert_eq!(ShellSignal::from_errno(21), ShellSignal::IsADirectory); // EISDIR
        assert_eq!(ShellSignal::from_errno(28), ShellSignal::DiskFull); // ENOSPC
        assert_eq!(ShellSignal::from_errno(8), ShellSignal::ExecFormat); // ENOEXEC
        assert_eq!(ShellSignal::from_errno(9999), ShellSignal::General); // unknown
    }

    #[test]
    fn exit_code_maps_the_decisive_ones() {
        assert_eq!(
            ShellSignal::from_exit_code(127),
            ShellSignal::CommandNotFound
        );
        assert_eq!(
            ShellSignal::from_exit_code(126),
            ShellSignal::PermissionDenied
        );
        assert_eq!(ShellSignal::from_exit_code(130), ShellSignal::Interrupted); // SIGINT
        assert_eq!(ShellSignal::from_exit_code(139), ShellSignal::Crashed); // SIGSEGV
        assert_eq!(ShellSignal::from_exit_code(3), ShellSignal::General);
    }

    #[test]
    fn glyph_column_is_single_width_for_prompt_safety() {
        // Tight shell status columns need single-scalar glyphs.
        for class in ShellSignal::ALL {
            let g = ShellSignals::prescribed().signal(*class).glyph;
            assert_eq!(
                g.chars().count(),
                1,
                "{class:?} glyph must be single-width, got {g:?}"
            );
        }
    }
}
