//! The fleet's prescribed command shell.
//!
//! ── ★ WHY THIS IS A TOKEN AND NOT A CONST IN EACH APP ───────────────────
//! Same reasoning as [`crate::fleet_keybinds::FleetKeybinds`]: code, not
//! config. Its whole value is being the single hand-edited source every fleet
//! app shares, so one edit here moves the fleet on next compile.
//!
//! It exists because the second independent derivation appeared. Measured
//! 2026-08-28, a sweep for shell-selection sites found 151 across 18 repos,
//! and inside the terminal stack alone there were SIX ladders that disagreed:
//! four in mado (two of them unguarded, with three different `/bin/sh`
//! floors), and two in tear -- where `tear up` resolved the shell client-side
//! four lines under a comment saying resolution belongs to the daemon, so the
//! daemon's configured default was never consulted at all.
//!
//! ── ★ THE FALLBACKS ARE PART OF THE TOKEN, DELIBERATELY ─────────────────
//! A prescription with no ladder underneath it is how a machine that does not
//! have frostmourne gets a dead window instead of a shell. The consumer is
//! expected to try each rung and take the first that is actually runnable --
//! the prescription is TRIED, never assumed.
//!
//! `/bin/sh` is last because it is the POSIX guarantee: reaching it means
//! nothing better was present.

/// The fleet's shell, and what to try when it is not installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FleetShell {
    /// The prescribed shell. A BARE NAME on purpose -- resolved through
    /// `PATH`, so a nix profile, a home-manager profile and a release download
    /// each get whatever they actually have rather than a store path frozen at
    /// compile time.
    pub prescribed: &'static str,
    /// Tried in order when the prescription is absent, before falling back to
    /// the user's own `$SHELL`. Every rung is a real path for the same reason
    /// `prescribed` is not: these are the shells a machine has when it does
    /// not have ours.
    pub fallbacks: &'static [&'static str],
}

impl FleetShell {
    /// The fleet's selection.
    ///
    /// frostmourne since 2026-07-26 on Darwin and 2026-08-28 fleet-wide -- the
    /// gap between those dates is the reason this is a token. The selection
    /// was made once and reached only the half of the fleet that could see the
    /// place it was written.
    #[must_use]
    pub const fn prescribed() -> Self {
        Self {
            prescribed: "frostmourne",
            fallbacks: &["/bin/zsh", "/bin/sh"],
        }
    }
}

impl Default for FleetShell {
    fn default() -> Self {
        Self::prescribed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prescription_is_a_bare_name() {
        // If this ever becomes a path, every consumer's PATH-based resolution
        // silently stops working the way it was designed to.
        let s = FleetShell::prescribed();
        assert!(
            !s.prescribed.contains('/'),
            "the prescribed shell must resolve through PATH, got {:?}",
            s.prescribed
        );
    }

    #[test]
    fn the_floor_is_the_posix_guarantee() {
        let s = FleetShell::prescribed();
        assert_eq!(
            s.fallbacks.last(),
            Some(&"/bin/sh"),
            "the last rung must be /bin/sh -- reaching it means nothing else was present"
        );
    }
}
