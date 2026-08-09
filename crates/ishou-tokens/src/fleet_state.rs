//! Fleet state vocabulary — the typed cross-tool environment contract.
//!
//! Fleet tools broadcast their live state to the processes they spawn
//! through environment variables, and read each other's state the same
//! way: tear exports `TEAR_SESSION_NAME` / `TEAR_PANE_ID` / `TEAR_TIER`
//! into every pane; seki's prompt reads them to show which session +
//! pane you're in; seki also reads `MADO_SOCKET` / `MADO_TIER`. Today
//! that contract is **duplicated string literals** — the producer hand-
//! types `"TEAR_SESSION_NAME"` when it sets the var and the consumer
//! hand-types the same string when it reads it, on opposite sides of the
//! fleet, with nothing tying them together. A typo or rename on one side
//! silently breaks cross-tool awareness (the env read just returns
//! `None`, no error).
//!
//! [`FleetStateVar`] lifts that contract to one typed vocabulary (peer
//! of [`FleetKeybinds`](crate::FleetKeybinds) for chords,
//! [`FleetSignals`](crate::FleetSignals) for emoji): the env-var NAME
//! lives in exactly one place, producer and consumer both reference the
//! typed variant, and a rename is a compile-time change on both sides at
//! once. This is the cross-tool-awareness substrate the fleet's
//! symbiosis rests on — "which session / pane / window / tier am I in?"
//! answered through one typed surface instead of scattered `std::env`
//! string literals.
//!
//! ```rust,ignore
//! use ishou_tokens::FleetStateVar;
//! // producer (tear), setting a child's environment:
//! cmd.env(FleetStateVar::TearSessionName.name(), &session_name);
//! // consumer (seki prompt), reading it:
//! if let Some(name) = FleetStateVar::TearSessionName.read() { /* show it */ }
//! ```
//!
//! The variants codify the env-var names that are *already in use* across
//! tools (verified in tear + seki source) plus the fleet-wide
//! [`FleetStateVar::FleetTheme`] selector — NOT invented names. Adopting
//! the typed surface is a drift-free rename of existing literals.

/// A typed cross-tool environment variable — the canonical name lives
/// here, referenced by both the tool that exports it and the tool that
/// reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum FleetStateVar {
    // ── tear multiplexer → spawned panes (tear exports; seki reads) ──
    /// The session's display name — the praça emoji session name
    /// (`🌊 tide`). `TEAR_SESSION_NAME`.
    TearSessionName,
    /// The session's stable id. `TEAR_SESSION_ID`.
    TearSessionId,
    /// The pane id within the session. `TEAR_PANE_ID`.
    TearPaneId,
    /// The shikumi config tier tear is running. `TEAR_TIER`.
    TearTier,
    /// The tear daemon control socket. `TEAR_SOCKET`.
    TearSocket,
    /// The attached client's id. `TEAR_CLIENT_ID`.
    TearClientId,

    // ── mado terminal → spawned shells (mado exports; seki reads) ────
    /// The mado MCP / control socket. `MADO_SOCKET`.
    MadoSocket,
    /// The shikumi config tier mado is running. `MADO_TIER`.
    MadoTier,

    // ── fleet-wide ──────────────────────────────────────────────────
    /// The active fleet theme name (e.g. `vellum`), so a shell / editor
    /// / prompt that can't link `ishou` still renders the right palette.
    /// `FLEET_THEME`.
    FleetTheme,
}

impl FleetStateVar {
    /// Every variant, in declaration order — for tooling that iterates
    /// the contract (a `fleet-state` dump, drift tests, docs).
    pub const ALL: &'static [FleetStateVar] = &[
        FleetStateVar::TearSessionName,
        FleetStateVar::TearSessionId,
        FleetStateVar::TearPaneId,
        FleetStateVar::TearTier,
        FleetStateVar::TearSocket,
        FleetStateVar::TearClientId,
        FleetStateVar::MadoSocket,
        FleetStateVar::MadoTier,
        FleetStateVar::FleetTheme,
    ];

    /// The canonical environment-variable name. The ONE place the string
    /// lives — producer and consumer both go through here.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            FleetStateVar::TearSessionName => "TEAR_SESSION_NAME",
            FleetStateVar::TearSessionId => "TEAR_SESSION_ID",
            FleetStateVar::TearPaneId => "TEAR_PANE_ID",
            FleetStateVar::TearTier => "TEAR_TIER",
            FleetStateVar::TearSocket => "TEAR_SOCKET",
            FleetStateVar::TearClientId => "TEAR_CLIENT_ID",
            FleetStateVar::MadoSocket => "MADO_SOCKET",
            FleetStateVar::MadoTier => "MADO_TIER",
            FleetStateVar::FleetTheme => "FLEET_THEME",
        }
    }

    /// Read this var from the current process environment. `None` when
    /// unset or non-UTF-8 — the consumer treats absence as "not in that
    /// context" (e.g. no `TEAR_SESSION_NAME` ⇒ not inside a tear pane).
    #[must_use]
    pub fn read(self) -> Option<String> {
        std::env::var(self.name()).ok()
    }

    /// `true` iff the variable is present + non-empty — the common
    /// "am I inside X?" guard (a tear pane sets a non-empty
    /// `TEAR_SESSION_NAME`; bare shells leave it unset).
    #[must_use]
    pub fn is_present(self) -> bool {
        self.read().is_some_and(|v| !v.is_empty())
    }
}

/// A snapshot of every fleet state var readable in the current
/// environment — the consumer-side convenience for a prompt / status
/// line that wants the whole cross-tool picture at once.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct FleetState {
    /// `(var name, value)` for each present var, in [`FleetStateVar::ALL`]
    /// order.
    pub present: Vec<(&'static str, String)>,
}

impl FleetState {
    /// Snapshot the environment now.
    #[must_use]
    pub fn snapshot() -> Self {
        let present = FleetStateVar::ALL
            .iter()
            .filter_map(|v| v.read().map(|val| (v.name(), val)))
            .collect();
        Self { present }
    }

    /// Look up a var's value in this snapshot.
    #[must_use]
    pub fn get(&self, var: FleetStateVar) -> Option<&str> {
        self.present
            .iter()
            .find(|(n, _)| *n == var.name())
            .map(|(_, v)| v.as_str())
    }

    /// `true` iff we appear to be inside a tear pane (has a session name).
    #[must_use]
    pub fn in_tear_session(&self) -> bool {
        self.get(FleetStateVar::TearSessionName)
            .is_some_and(|v| !v.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_the_existing_cross_tool_contract() {
        // These are the env-var names ALREADY in use (tear sets them,
        // seki reads them) — the typed surface is a drift-free rename,
        // not an invention. Pinning them here makes an accidental rename
        // a compile+test failure on the single source of truth.
        assert_eq!(FleetStateVar::TearSessionName.name(), "TEAR_SESSION_NAME");
        assert_eq!(FleetStateVar::TearPaneId.name(), "TEAR_PANE_ID");
        assert_eq!(FleetStateVar::TearSessionId.name(), "TEAR_SESSION_ID");
        assert_eq!(FleetStateVar::TearTier.name(), "TEAR_TIER");
        assert_eq!(FleetStateVar::MadoSocket.name(), "MADO_SOCKET");
        assert_eq!(FleetStateVar::MadoTier.name(), "MADO_TIER");
        assert_eq!(FleetStateVar::FleetTheme.name(), "FLEET_THEME");
    }

    #[test]
    fn all_is_complete_and_unique() {
        // Every variant appears in ALL exactly once, and names don't
        // collide.
        let mut names: Vec<&str> = FleetStateVar::ALL.iter().map(|v| v.name()).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            count,
            "duplicate env-var name in FleetStateVar"
        );
        assert_eq!(count, 9);
    }

    #[test]
    fn read_absent_var_is_none() {
        // A var that is not set reads as None (the "not in that context"
        // signal). TEAR_SESSION_ID is exceedingly unlikely to be set in
        // the test environment.
        // (We don't mutate process env in tests to stay thread-safe.)
        if std::env::var(FleetStateVar::TearSessionId.name()).is_err() {
            assert_eq!(FleetStateVar::TearSessionId.read(), None);
            assert!(!FleetStateVar::TearSessionId.is_present());
        }
    }

    #[test]
    fn snapshot_get_is_consistent_with_read() {
        let snap = FleetState::snapshot();
        for v in FleetStateVar::ALL {
            assert_eq!(snap.get(*v).map(str::to_owned), v.read());
        }
    }
}
