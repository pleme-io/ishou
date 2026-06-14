# Fleet Theme Framework — design brief

> Status: DRAFT (mandate + architecture + constraints captured 2026-06-13).
> Detailed spec (exact hex, contrast ratios, file:line projector map) is
> produced by the design workflow on top of this brief.

## The mandate (operator, 2026-06-13)

Build the **fleet theme framework**: review *everything* that affects theming
and coloring across pleme-io, unify it into one typed framework that **meets
stylix** (base16) and **every other pleme-io theming surface**, works **across
all Rust apps that opt in**, ships a **library/array of themes**, and lets us
**change theme dynamically** via **shikumi-style tiered configuration**.

First deliverable theme + default: **Nord-Matte** — matte (low glow/bloom),
more Nord, **darker than Borealis**, with **excellent visibility/contrast**.
"A class default look to use for years."

Two live bugs the framework must structurally fix:
1. **vim renders all white** in the embedded-mado window (see §Bugs).
2. **Ctrl-R / Ctrl-T pickers (skim) are thematically out of sync** — a
   different, brighter, hand-synced Nord than everything else.

## What already exists (build ON this, do not duplicate)

- **ishou** (衣装 = attire) is the fleet visual/theming library.
  - `ishou-tokens`: `color.rs`, `borealis.rs`, `fleet_defaults.rs`,
    `fleet_theme.rs`, `themed_config.rs` (`FleetThemedConfig` trait +
    `convergence::Guard`), `brand.rs`, `motion/shadow/shader`.
  - `ishou-render`: **already bridges Rust → stylix** —
    `src/stylix.rs`, `src/stylix_fonts.rs`, `src/borealis.rs`,
    `src/nix_ast.rs`. Emits a base16 scheme YAML stylix consumes.
- **stylix is wired fleet-wide.**
  `blackmatter/modules/home-manager/blackmatter/themes/stylix.nix` is the hub
  ("central Nord/Stylix color hub"; all components read
  `config.lib.stylix.colors`). The **base16 ↔ Nord map is already documented
  there** (base00=#2E3440 … base0F=#5E81AC). `themes/nord/default.nix` exposes
  `base16SchemePath` so the nix repo points stylix at **ishou's** rendered
  base16 (palette "born in ishou", no foreign nord.yaml).
  - stylix-OWNED targets (auto-themed): gtk, bat, fzf, zsh-syntax, console, k9s
    (via lib.stylix.colors), …
  - **blackmatter-OWNED targets (stylix disabled, hand-rolled):** `ghostty`,
    `neovim` (nord.nvim plugin + custom highlights), `starship`. ← these are
    the drift sources; the framework should fold them back to base16.
- **mado** already has **8 built-in themes** (`theme.rs`: "nord, dracula, …"),
  a `Theme { ansi:[Color;16], foreground, background, agent_accent,
  search_current, search_others }`, and prescribes `borealis-night`. ANSI flows
  via `renderer.set_ansi_colors` + mirror `Terminal::apply_theme` + OSC 11.
- **skim-tab** `NORD_COLORS` (`src/lib.rs:29`) is hand-coded standard Nord,
  **dual-mirrored** to `nix/lib/skim-theme.nix` (comment-enforced byte-equality).
- **nvim** (`blackmatter-nvim`): `termguicolors=true` **unconditional**
  (`groups/common/settings.lua:15`); `colorscheme nord` plugin
  (`groups/theming/colorscheme.lua:4`), NOT stylix base16; lualine/bufferline
  read `colorscheme.base`.

## The destination architecture (one source → many projections)

```
                 ishou-tokens: Theme (the typed keystone)
   base16[16] + semantic roles + EffectProfile + font + Contrast(asserted)
                              │
        ┌─────────────────────┼───────────────────────────────┐
        ▼                     ▼                                 ▼
  base16 YAML            Rust projectors                  (metadata)
  (ishou-render)     mado / seki / skim / escriba      Contrast guard,
        │            (typed Theme → each surface)       convergence::Guard
        ▼                     │
     STYLIX  ───────────────► themes nvim, gtk, bat, fzf, console, ghostty,
   (Nix world)                starship, k9s  (base16, ONE palette)
```

- **`Theme` is the typed keystone** (in ishou-tokens): 16 base16 colors +
  semantic role map (bg/fg/comment/red/green/yellow/blue/cyan/purple/accent…) +
  `EffectProfile` (matte = bloom/glow≈0, optional faint vignette/grain) +
  font + a `Contrast` proof (WCAG ratios asserted at construction → visibility
  is a type invariant, not a hope).
- **Theme library / array**: named typed themes — `nord-matte` (default),
  `borealis` (existing), `nord-classic`, + lift mado's existing 8. Each is one
  `Theme` value; a registry exposes them by name.
- **Projectors** (one per surface, the ishou-render bridge generalized):
  - `→ base16 YAML` for stylix (themes the whole Nix world incl. nvim).
  - `→ mado Theme` (ansi[16]+fg/bg+search+accent) + EffectProfile.
  - `→ seki StyleSpec` (prompt), `→ skim NORD_COLORS` (kills the dual-mirror;
    nix reads the generated one), `→ escriba syntax`.
- **Selection + dynamic switch = shikumi**: `theme = "<name>"` in each app's
  `shikumi::TieredConfig` (bare / discovered / prescribed_default=nord-matte).
  Rust apps **hot-reload** (notify → re-resolve Theme → re-theme live). The
  stylix/Nix surfaces (nvim, system) switch on **rebuild** (stylix is
  build-time) — documented future path: a runtime theme file nvim re-reads to
  make it hot too. So: runtime-dynamic where natural, rebuild where stylix owns.

## Constraints (all must hold)

- **Visibility is non-negotiable** — Nord is low-contrast; matte+darker raises
  the risk. Every `Theme` asserts contrast (fg/bg, comment/bg, each syntax
  color/bg, statusline) ≥ a chosen WCAG floor. Contrast is a type invariant.
- **Matte** = effect profile with glow/bloom at/near zero; no neon. Texture
  (if any) is a barely-perceptible grain/vignette, shared-clock composed.
- **Cohesive across** mado, nvim, frostmourne (seki + reedline), escriba, skim
  Ctrl-R/Ctrl-T — same palette by construction.
- **Single source of truth** — no hand-synced duplicates (skim dual-mirror and
  the hand-rolled nvim/ghostty/starship palettes get folded to base16/ishou).
- **shikumi compliance** — selection + hot-reload through TieredConfig.
- **convergence::Guard** — every consumer pins its theme at test time.

## Bugs

### vim all-white (embedded mado)
Symptom: dark navy bg survives but ALL text/UI is white — lualine has no
colored segments (separators white). `mado/src/caps.rs:184` documents the
class: the **embedded-tear spawn path** projects a weaker capability env than
the local-PTY path (missing `COLORTERM=truecolor` / wrong `TERMINFO`). nvim
has `termguicolors=true` unconditional, so it EMITS 24-bit escapes — meaning
the failure is either (a) mado's embedded renderer dropping truecolor SGR, or
(b) the nord plugin/lualine `base` palette not populating. Fix = **one typed
CapsEnv projector used by EVERY spawn path** (TERM, COLORTERM=truecolor,
TERMINFO, TERM_PROGRAM identical everywhere) **+** move nvim to stylix base16
with **cterm fallbacks** so it degrades to colored-not-white. Confirm exact
path in the understand workflow (vim-white-env-bug reader).

### skim picker drift
`skim-tab/src/lib.rs:29 NORD_COLORS` hand-coded + dual-mirrored to nix. Fix =
generate from the `Theme` (skim projector); nix consumes the generated file.

## Plan (destination-first; phases are the path down)

1. **Understand** (workflow running) — map ishou-tokens / mado pipeline /
   vim-white env paths / engawa matte effects / seki+escriba / nvim. Extend to
   stylix-target inventory + ishou-render bridge depth + shikumi switch.
2. **Design** — the `Theme` type + role map + `EffectProfile` + `Contrast`;
   the projector set; the theme library; the shikumi switch; **the Nord-Matte
   palette (real hex + contrast ratios)** + matte effect profile. Present
   swatches for operator review.
3. **Implement** (phased) — Theme + Nord-Matte in ishou-tokens; ishou-render
   base16 projector → stylix; mado/seki/skim/escriba projectors; nvim→stylix
   base16 + cterm fallback; the CapsEnv fix; Guards.
4. **Review** — contrast audit, cohesion screenshots, convergence guards green.

---

## FINDINGS (understand phase, 2026-06-13) — grounded file:line map

### The fleet is split across TWO palettes (the cohesion gap, quantified)
- **Borealis** (ishou `BorealisPalette::night`, bg `#1F222F`, green `#67D191`, cyan
  `#73C6D9`, fg `#D4D9E3`, agent `#B69AE9`): used by **mado** (default
  `borealis-night`) + **seki** prompt (Borealis-native, reads ishou tokens).
- **Classic Nord** (`irodori::NORD`, bg `#2E3440`, green `#A3BE8C`, cyan `#88C0D0`,
  fg `#D8DEE9`): used by **nvim** (shaunsingh/nord.nvim), **escriba** (3 copies:
  TUI ratatui literals + GPU `irodori::NORD` + syntax lisp), **frostmourne**
  command-line highlighter (frost `nord_default()`), **skim-tab** `NORD_COLORS`
  (+ its `nix/lib/skim-theme.nix` dual-mirror).
- Result: skim Ctrl-R/Ctrl-T (`#2E3440`) sit next to mado (`#1F222F`) → the
  "out of sync" the operator feels. Nord-Matte, derived everywhere, closes it.

### Per-surface extension points (where each consumes the new palette)
- **ishou-tokens** (keystone): add palette in `borealis.rs` (`::matte()` sibling
  of `::night()`), `ResolvedTheme::nord_matte()` in `fleet_theme.rs`, new
  `FleetTheme::NordMatte` variant + `resolve()` arm. `resolve()` is the single
  choke point. Flip `FleetTheme::prescribed_default()` → every app converges next
  compile (Guard pins it). `TokenSet::default()` still on legacy Nord — a fork to
  note (CSS/Tailwind render targets key off TokenSet, not Borealis).
- **ishou-render**: `stylix.rs` already emits base16→stylix; add the Nord-Matte
  base16 emit (this is the stylix bridge). `borealis.rs`/`nix_ast.rs` adjacent.
- **stylix**: `blackmatter/.../themes/stylix.nix` hub; `themes/nord/default.nix`
  `base16SchemePath` ← nix repo points at ishou's rendered base16. stylix auto-
  themes gtk/bat/fzf/console/k9s; blackmatter-owned (must fold in): ghostty,
  **neovim**, starship.
- **mado**: add `nord_matte_theme()` builder in `theme.rs` sourced from ishou
  (the fleet-native path — NOT irodzuki preset, which lacks agent/search bands).
  `FALLBACK_THEME` (auto_detect.rs) + `FleetTheme::prescribed_default()` drive
  default. Hot-reload (`ux/config_apply.rs`) picks it up free.
- **mado matte effects**: `ambience.rs` — add `AmbiencePreset::Matte` arm whose
  `compose()` drops bloom + glow_on_bell (the two glow sources), aurora at
  Off/trace, optional faint scanline/grain. "If you can point at it, it's too
  loud" law already aims near-matte. `engawa-wgpu/src/catalog/` is the live
  effect catalog (ENGAWA.md is stale).
- **seki**: repoint `seki-shikumi/src/borealis.rs` `PromptPalette` to the matte
  band (or add `::matte()`). Already ishou-native; per-segment StyleSpec rebuilds.
- **escriba**: collapse its 3 Nord copies onto one ishou palette (TUI render.rs +
  GPU gpu.rs irodori::NORD + syntax `blnvim-defaults.lisp` (defhighlight/defpalette)).
- **frostmourne**: author a `(deftheme …)` for the reedline highlighter (today
  unthemed → frost classic `nord_default()`); token-derive `61-tools-skim.lisp`.
- **skim-tab**: regenerate `NORD_COLORS` (`src/lib.rs:29`) from the Theme; nix
  consumes the generated file (kills the dual-mirror).

### vim all-white — ROOT CAUSE (nvim-side defect + stale binary)
- mado's **embedded path is FIXED in source** (`caps.rs` `EnvProjection::prescribed`
  projects `COLORTERM=truecolor` + `TERMINFO=xterm-ghostty` w/ `Tc`; `pty.rs` +
  `gui_tear_attach.rs:796` use it; nvim 0.11 honors COLORTERM; mado renders 24-bit
  SGR). **Residual mado gap:** the *daemon* runtime path (`run_against_pane`) has no
  `SetSpawnEnv` wire RPC → drops the env. Operator is on `embedded`, so not the live
  hit, but land the `SetSpawnEnv` RPC for parity.
- **nvim is the live defect** (`blackmatter-nvim`): `settings.lua:15`
  `termguicolors=true` **unconditional**; `colorscheme.lua:4` `colorscheme nord`
  (shaunsingh/nord.nvim) **gui-only, zero cterm fallback**, no `pcall`. So the
  instant COLORTERM is missing (stale mado binary, or daemon mode, or a colorscheme
  load failure → E185) → every highlight collapses to terminal default → ALL WHITE
  (matches the screenshot's uncolored lualine). 4-part fix: (1) guard termguicolors
  on COLORTERM/truecolor; (2) cterm fallbacks on every hl; (3) `pcall` colorscheme +
  fallback; (4) move nvim onto stylix base16 Nord-Matte (matches fleet + gets cterm).
- **Stale binary**: deployed mado via home-manager-path; last HM activation
  2026-06-09; env fix landed 2026-06-13; flake.lock now pins mado `556b685` (has the
  fix) but not yet activated. → **rebuild to deploy the env half.**

### Useful invariants
- ishou Color = 8-bit sRGB `Rgb`; gamma via `space.rs` typed Srgb/Linear; only
  `From<LinearRgba> for wgpu::Color` (washed-out bug unrepresentable). New palette
  authors use `Rgb::new`.
- mado truecolor SGR passes through verbatim; ANSI index→`ansi_colors[idx]` at
  SGR-parse; out-of-range 256-index falls back to `Color::WHITE` (a white source).
- `FleetThemedConfig` + `convergence::Guard` exist; NONE of seki/frostmourne/escriba/
  nvim ship a Guard yet — adding them makes cohesion a test guarantee.

---

## DESIGN LOCKED — "Nord Matte · Vellum" (operator-approved 2026-06-13)

Direction the operator converged on: *matte → deeper → "more paper" → cozy/worn →
warm → rounded edges.* Final = a **warm aged-parchment Nord**: deep warm charcoal,
faded worn-pigment accents, paper grain + candlelight vignette, rounded corners
fleet-wide. Operator: "yes I love these changes… make this the default and bring it
to mado and all its crew, in a uniform way." Every ratio recomputed (exact WCAG,
ruby) — ALL ≥ AA, fg ≥ AAA.

base16 (the keystone — every surface derives these):
```
base00 #16140E bg (warm, 1.47x deeper than Borealis #1F222F, R>B)
base01 #1F1C15 surface    base02 #2B2820 selection/elevated
base03 #90897B comment 5.30:1 (Nord's ~2.4:1 problem fixed)
base04 #ADA593 dim 7.52   base05 #E2DBC8 fg 13.33:1 (warm ivory)
base06 #EDE6D6            base07 #F4EFE2
base08 #C9837B red 6.15   base09 #CB9070 orange 6.81
base0A #D7C489 yellow 10.65  base0B #A9BB8C green 8.92
base0C #94BBB8 cyan 8.82  base0D #99AABE blue 7.76
base0E #B8A1B9 purple 7.75  base0F #B3886C brown 5.84
```
extras: cursor #ADD7A3 · agent_accent #B29EC4 · border #6E6857 (3.31) · search_current
#D9C285 · search_others #46412F · selection #2B2820 · statusline_bg #1F1C15 ·
statusline_fg #CDC7B6 · mode pills cyan/green/purple with **base00 dark text**
(7.75–8.92:1 → all-white lualine impossible).
ANSI16: slot0 #2B2820 (not pure black) … slot15 #F4EFE2 (not pure white).

Matte effect (mado `AmbiencePreset::Matte`): DROP bloom + glow_on_bell; aurora →
trace (~0.8%/Off); ADD luma paper grain (~1.5%, slow) + warm candlelight vignette
(corner darkening, non-emissive); one composed layer, shared clock/noise.

Shape (NEW fleet principle): **round edges where possible** — drive from ishou
`radius.rs`. nvim float/picker borders "rounded"; statusline pill caps; tab/selection/
popups soft; mado UI radius where feasible.

Preview: `/tmp/nord-matte-vellum.html`. Cool variant "Polar Veil" (#171A22) kept as a
library reference; Vellum is the default.
