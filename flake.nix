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
      render-all = {
        type = "app";
        program = "${pkgs.writeShellScriptBin "ishou-render-all" ''
          exec ${ishouBin}/bin/ishou render-all "$@"
        ''}/bin/ishou-render-all";
      };
    };
  in
    toolOutputs
    // {
      apps = nixpkgs.lib.genAttrs systems (system:
        (toolOutputs.apps.${system} or {}) // (mkRenderApps system)
      );
    };
}
