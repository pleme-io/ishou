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

impl FleetShell {
    /// True if `cmd` names a runnable binary: an absolute or relative path
    /// that is a file, or a bare name found on `PATH`.
    #[must_use]
    pub fn is_executable(cmd: &str) -> bool {
        if cmd.trim().is_empty() {
            return false;
        }
        let direct = std::path::Path::new(cmd);
        if direct.is_absolute() || cmd.contains('/') {
            return direct.is_file();
        }
        std::env::var_os("PATH")
            .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
            .unwrap_or(false)
    }

    /// Resolve the shell to spawn.
    ///
    /// ── ★ THE LADDER LIVES WITH THE TOKEN ───────────────────────────────
    /// The prescription and the rules for applying it are one fact. Handing
    /// apps a `&'static str` and letting each write its own ladder is how six
    /// of them ended up disagreeing in the first place -- and every one of
    /// those ladders had to re-derive the same executable check, two of them
    /// forgot to, and the ones that remembered used three different floors.
    ///
    /// `configured` is an explicit choice from a flag, a config file or a
    /// request. `None` means NOBODY HAS SAID, which is where the prescription
    /// applies. Every rung is guarded, so this never returns a name that is
    /// not there.
    #[must_use]
    pub fn resolve(&self, configured: Option<&str>) -> String {
        // 1 — an explicit choice, when it exists. A configured shell that is
        //     missing is a misconfiguration worth reporting, not spawning.
        if let Some(c) = configured.filter(|c| !c.trim().is_empty()) {
            if Self::is_executable(c) {
                return c.to_string();
            }
        }
        // 2 — the fleet's shell.
        if Self::is_executable(self.prescribed) {
            return self.prescribed.to_string();
        }
        // 3 — the operator's own login shell. On a fleet node this IS the
        //     prescription, so this rung is for machines that are not ours.
        if let Ok(s) = std::env::var("SHELL") {
            if Self::is_executable(&s) {
                return s;
            }
        }
        // 4 — the declared fallbacks, in order, then the POSIX floor.
        for candidate in self.fallbacks {
            if Self::is_executable(candidate) {
                return (*candidate).to_string();
            }
        }
        self.fallbacks
            .last()
            .map_or_else(|| "/bin/sh".to_string(), |f| (*f).to_string())
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
    fn an_explicit_choice_that_does_not_exist_falls_through() {
        // Two of the six ladders this replaces had no guard here and would
        // have handed the string straight to execvp.
        let s = FleetShell::prescribed();
        let got = s.resolve(Some("/nonexistent/definitely-not-a-shell"));
        assert_ne!(got, "/nonexistent/definitely-not-a-shell");
    }

    #[test]
    fn an_empty_choice_is_the_same_as_no_choice() {
        // The apps spelled this two ways -- `is_empty()` and a `.filter()` --
        // which is two chances to disagree about one rule.
        let s = FleetShell::prescribed();
        assert_eq!(s.resolve(Some("")), s.resolve(None));
        assert_eq!(s.resolve(Some("   ")), s.resolve(None));
    }

    #[test]
    fn the_result_is_never_empty() {
        let s = FleetShell::prescribed();
        for input in [None, Some(""), Some("/nope")] {
            assert!(!s.resolve(input).is_empty(), "resolve({input:?}) was empty");
        }
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
