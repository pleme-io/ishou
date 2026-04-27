# ishou (意匠)

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.


The pleme-io design language — a centrally-maintained design system.
`ishou` literally means "design" in Japanese. Typed Rust tokens →
rendered to every platform target. Single source of truth for color,
typography, spacing, shadow, motion, shaders, and brand identity.

## Why

Before ishou: every pleme-io repo re-declared its colors, fonts, spacing,
shadows, and shader knobs locally. `irodori` owned the Nord palette, but
everything else was hand-copied. `lilitu-web`, `lilitu/web`, `zuihitsu`,
`tobira`, `ayatsuri`, `hikyaku`, `blackmatter-ghostty` all drifted.

After ishou: `TokenSet::pleme()` is the single source. Every consumer calls
a renderer (`nix run .#css`, `nix run .#tailwind`, `nix run .#glsl`, ...) and
gets byte-identical output.

## Layers

| Layer | Where | Owns |
|---|---|---|
| **Palette** | `crates/ishou-tokens/src/color.rs` | Nord (via irodori) + brand monochrome (ink / paper / shadow_tone) |
| **Semantic roles** | `color.rs::SemanticRoles` | `background / surface / text / primary / error / ...` → palette key |
| **Typography** | `typography.rs` | serif / sans / mono / display stacks + 9-step scale + weights + line heights |
| **Spacing** | `spacing.rs` | 4px-based scale, 13 steps |
| **Radius** | `radius.rs` | `none sm md lg xl pill full` |
| **Shadow** | `shadow.rs` | 6 elevation levels + **`brand-bold`** hard drop |
| **Motion** | `motion.rs` | durations + easings (`standard / decelerate / accelerate / sonic-boom / saber`) |
| **Shader** | `shader.rs` | typed uniforms for all 13 blackmatter-ghostty shaders |
| **Brand** | `brand.rs` | logo paths, mark variants, swerve geometry |

## Render targets

| Target | Output | Primary consumer |
|---|---|---|
| `css` | `:root { --ishou-*: ... }` + brand utility classes | zuihitsu, lilitu-web |
| `tailwind` | complete `tailwind.config.js` | any Tailwind consumer |
| `scss` | `$ishou-*: ...` variables | future SCSS consumers |
| `rust` | `pub const RGB_*` module | TUI apps not using serde |
| `json` | W3C Design Tokens JSON | Figma, Style Dictionary |
| `glsl` | `#define ISHOU_* vec3(...)` header | blackmatter-ghostty shaders |
| `ghostty` | Ghostty config block + shader stack | Ghostty terminal |
| `tui` | ratatui/crossterm `Color` table | tobira, ayatsuri, hikyaku |
| `svg` | brand mark (swerve inside viewbox) | any asset pipeline |

## CLI

```bash
ishou render --target css --out style/ishou.css
ishou render-all --out-dir generated/
ishou hash                                  # content hash of current token set
ishou targets                               # list all targets
```

## Nix apps

```bash
nix run .#css        > tokens.css
nix run .#tailwind   > tailwind.config.js
nix run .#glsl       > nord-common.glsl
nix run .#ghostty    > ~/.config/ghostty/config
nix run .#render-all -- --out-dir generated/
```

## Consumer pattern (zuihitsu is the pilot)

```nix
# In the consumer's flake.nix:
ishou.url = "github:pleme-io/ishou";
# ...
packages.tokens-css = pkgs.runCommand "tokens.css" {} ''
  ${inputs.ishou.packages.${system}.default}/bin/ishou render --target css --out $out
'';
```

Then bundle `packages.tokens-css` into the final output. The consumer's
CI runs `ishou hash` to check the local copy hasn't drifted from upstream.

## Determinism

`TokenSet::content_hash()` is deterministic. `cargo test -p ishou-tokens`
includes a snapshot test + cross-render determinism tests. Every renderer
has a `render_is_deterministic` test — a byte-difference between two runs
is a red build.

## Relationship to `arch-synthesizer`

ishou is designed to plug into `arch-synthesizer` as a new AST domain
(currently at 20; ishou would be the 21st — "visual"). Morphisms:

```
TokenSet → CSS         (ishou-render::css)
TokenSet → Tailwind    (ishou-render::tailwind)
TokenSet → GLSL        (ishou-render::glsl)
...
```

Each is provable-total + deterministic, which arch-synthesizer can wrap
with `Attested<T>` once the bridge module lands. That's follow-up work —
the renderers themselves are ready today.

## Adding a token

1. Extend the relevant struct in `ishou-tokens/src/<layer>.rs` with a new
   field + default value.
2. Update every `pairs()` / `entries()` accessor if you added to a keyed set.
3. Update every renderer that should surface the new token.
4. Run `cargo test -p ishou-tokens` — the snapshot test will flag + require
   re-acceptance (`cargo insta review`).
5. Commit. Downstream consumers pick it up on next `nix flake update`.

## Adding a render target

1. Create `ishou-render/src/<target>.rs` with a `pub fn render(&TokenSet) -> String`.
2. Add the variant to `Target` enum in `ishou-render/src/lib.rs`.
3. Add the filename in `ishou-cli/src/main.rs::filename()`.
4. Expose a Nix app in `flake.nix → mkRenderApp`.
5. Write a determinism test.

## Conventions

- Edition 2024, Rust 1.89.0+, MIT, public GitHub.
- `[lints.clippy] pedantic = "warn"` at workspace level.
- No allocation in `TokenSet::default()` — token construction is `const` where possible.
- Renderers never touch `std::env`, never read files. Pure `(TokenSet) → String`.
- Commit directly to `main` and push. No Claude co-author attribution.
- Repo created via pangea-github + repo-forge (not `gh repo create`).
