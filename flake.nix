{
  description = "ishou (意匠) — pleme-io design system: typed tokens → every platform render target";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    crate2nix.url = "github:nix-community/crate2nix";
    flake-utils.url = "github:numtide/flake-utils";
    substrate = {
      url = "github:pleme-io/substrate";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Canonical pleme-io Rust-workspace-tool pattern: substrate's
  # rust-workspace-release-flake owns the whole build surface (packages,
  # apps, devShells, overlays, release metadata) from toolName +
  # packageName + src + repo. Cargo.nix is the crate2nix-generated
  # lockfile equivalent; regenerate via `nix run .#regenerate` when
  # Cargo.lock changes.
  #
  # The per-target render apps (css/tailwind/…) are convenience wrappers
  # on top of `ishou render --target X` — callers that already invoke
  # `ishou ...` directly (zuihitsu) do not need them, so this flake
  # leans on `nix run .#default -- render --target <X>` as the primary
  # shape and only re-exposes the common renderers for ergonomics.
  outputs = { self, nixpkgs, crate2nix, flake-utils, substrate, ... }: let
    toolOutputs = (import "${substrate}/lib/rust-workspace-release-flake.nix" {
      inherit nixpkgs crate2nix flake-utils;
    }) {
      toolName = "ishou";
      packageName = "ishou-cli";
      src = self;
      repo = "pleme-io/ishou";
    };

    systems = [ "aarch64-darwin" "x86_64-linux" "aarch64-linux" ];

    mkRenderApps = system: let
      pkgs = import nixpkgs { inherit system; };
      ishouBin = toolOutputs.packages.${system}.default;
      mk = target: {
        type = "app";
        program = "${pkgs.writeShellScriptBin "ishou-render-${target}" ''
          exec ${ishouBin}/bin/ishou render --target ${target} "$@"
        ''}/bin/ishou-render-${target}";
      };
    in {
      css      = mk "css";
      tailwind = mk "tailwind";
      scss     = mk "scss";
      rust     = mk "rust";
      json     = mk "json";
      glsl     = mk "glsl";
      ghostty  = mk "ghostty";
      tui      = mk "tui";
      svg      = mk "svg";
      stylix   = mk "stylix";
      stylix-fonts = mk "stylix-fonts";
      fleet-fonts = mk "fleet-fonts";
      nix      = mk "nix";
      render-all = {
        type = "app";
        program = "${pkgs.writeShellScriptBin "ishou-render-all" ''
          exec ${ishouBin}/bin/ishou render-all "$@"
        ''}/bin/ishou-render-all";
      };
    };

    # The stylix-base16 package output is the load-bearing piece that
    # closes the foreign-app theme loop. After M2 lands in pleme-io/nix,
    # `darwin-developer`'s `stylix.base16Scheme` is sourced from this
    # path → every GTK / GNOME / alacritty / kitty / btop / k9s app
    # stylix knows about inherits the ishou Nord. Build-time materialise
    # so flake consumers don't pay the runtime cost.
    mkStylixBase16 = system: let
      pkgs = import nixpkgs { inherit system; };
      ishouBin = toolOutputs.packages.${system}.default;
    in pkgs.runCommand "ishou-stylix-base16-nord-dark" {
      meta.description = "ishou-rendered base16 YAML for stylix consumers (Nord Dark)";
    } ''
      ${ishouBin}/bin/ishou render --target stylix > $out
    '';

    # Importable Nix attrset replacing every retiring
    # `<wrapper>/module/themes/nord/colors.nix`. Consumers can
    # `import (inputs.ishou.packages.${system}.nord-palette-nix)` and
    # get the same `{ polar = …; snow = …; frost = …; aurora = …; }`
    # shape they had locally — one Nord across the fleet's foreign-app
    # wrappers. See pleme-io/theory/THEME-ARCHITECTURE.md.
    mkNordPaletteNix = system: let
      pkgs = import nixpkgs { inherit system; };
      ishouBin = toolOutputs.packages.${system}.default;
    in pkgs.runCommand "ishou-nord-palette-nix" {
      meta.description = "ishou-rendered Nord palette as an importable Nix attrset";
    } ''
      ${ishouBin}/bin/ishou render --target nix > $out
    '';

    # Typed font primitive rendered to stylix.fonts shape. Consumed
    # in pleme-io/nix/darwinConfigurations/default.nix as:
    #   stylix.fonts = import inputs.ishou.packages.${system}.stylix-fonts { inherit pkgs; };
    # The single ishou Typography drives every foreign app stylix
    # touches (GTK, GNOME, alacritty, kitty, …) AND every pleme-io
    # GPU app via the typed `MonoFonts::pleme()` consumer pattern.
    mkStylixFonts = system: let
      pkgs = import nixpkgs { inherit system; };
      ishouBin = toolOutputs.packages.${system}.default;
    in pkgs.runCommand "ishou-stylix-fonts" {
      meta.description = "ishou-rendered stylix.fonts attrset (Nerd Font + calligraphic italic)";
    } ''
      ${ishouBin}/bin/ishou render --target stylix-fonts > $out
    '';

    # GPU-app-shaped fleet-fonts attrset — `{ primary, italic, bold,
    # symbols, emoji, fallback_chain }` consumed directly by every
    # pleme-io GPU app's HM module (blackmatter-mado today; future
    # blackmatter-ghostty / blackmatter-fumi / blackmatter-kagi …)
    # to name the canonical face AND install the underlying nixpkgs
    # font package via `home.packages`. Sibling of `stylix-fonts` —
    # same typed Typography source, different consumer-side shape.
    #
    # Consumed as:
    #   let fonts = import inputs.ishou.packages.${system}.fleet-fonts
    #                 { inherit pkgs; };
    #   in {
    #     options.myapp.font.family = lib.mkOption {
    #       default = fonts.primary.name;
    #     };
    #     config.home.packages = [
    #       fonts.primary.package
    #       fonts.italic.package
    #       fonts.symbols.package
    #     ];
    #   }
    mkFleetFonts = system: let
      pkgs = import nixpkgs { inherit system; };
      ishouBin = toolOutputs.packages.${system}.default;
    in pkgs.runCommand "ishou-fleet-fonts" {
      meta.description = "ishou-rendered fleet-fonts attrset (primary + italic + bold + symbols + emoji + fallback chain) for every pleme-io GPU app's HM module";
    } ''
      ${ishouBin}/bin/ishou render --target fleet-fonts > $out
    '';
  in
    toolOutputs
    // {
      apps = nixpkgs.lib.genAttrs systems (system:
        (toolOutputs.apps.${system} or {}) // (mkRenderApps system)
      );
      packages = nixpkgs.lib.genAttrs systems (system:
        (toolOutputs.packages.${system} or {}) // {
          stylix-base16 = mkStylixBase16 system;
          # Alias under the explicit theme name so consumers can
          # eventually switch schemes by changing the attribute name
          # (e.g. `stylix-base16-dracula`) without editing the
          # consumer-side reference shape.
          stylix-base16-nord-dark = mkStylixBase16 system;
          # Replaces the retiring foreign `themes/nord/colors.nix`
          # files in blackmatter-{ghostty,mado,opencode} per M6.1 of
          # the theme architecture rollout.
          nord-palette-nix = mkNordPaletteNix system;
          # Typed stylix.fonts attrset — sourced from ishou's
          # Typography primitive. See pleme-io/theory/THEME-ARCHITECTURE.md.
          stylix-fonts = mkStylixFonts system;
          # GPU-app-shaped fleet-fonts attrset. Every pleme-io GPU
          # app's HM module (mado, ghostty, fumi, kagi, …) imports
          # this to name the canonical font AND install the
          # underlying nixpkgs package via home.packages.
          fleet-fonts = mkFleetFonts system;
        }
      );
    };
}
