{
  description = "ishou (意匠) — pleme-io design system: typed tokens → every platform render target";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    substrate = {
      url = "github:pleme-io/substrate";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crate2nix = {
      url = "github:nix-community/crate2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, substrate, fenix, crate2nix, ... }:
    let
      systems = [ "aarch64-darwin" "x86_64-linux" "aarch64-linux" ];
      forEach = f: nixpkgs.lib.genAttrs systems (system: f system);

      mkPerSystem = system: let
        pkgs = import nixpkgs { inherit system; };
        fenixPkgs = fenix.packages.${system};
        rust = fenixPkgs.combine [ fenixPkgs.latest.cargo fenixPkgs.latest.rustc ];
        devTools = [ rust pkgs.pkg-config pkgs.openssl fenixPkgs.latest.clippy fenixPkgs.latest.rustfmt ];
        binPath = pkgs.lib.makeBinPath devTools;

        # rustPlatform.buildRustPackage vendors deps into the sandbox from
        # Cargo.lock so crates.io-resolved deps (irodori) are reachable
        # without network access during the build. The previous naive
        # `cargo build` inside mkDerivation only worked while every dep was
        # a local path-sibling — irodori moving to crates.io broke it.
        ishouBin = pkgs.rustPlatform.buildRustPackage {
          pname = "ishou";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "--bin" "ishou" "-p" "ishou-cli" ];
          doCheck = false;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        mkRenderApp = target: {
          type = "app";
          program = "${pkgs.writeShellScriptBin "ishou-render-${target}" ''
            set -e
            export PATH=${binPath}:$PATH
            exec ${ishouBin}/bin/ishou render --target ${target} "$@"
          ''}/bin/ishou-render-${target}";
        };
      in {
        packages = { default = ishouBin; ishou = ishouBin; };
        apps = {
          default  = { type = "app"; program = "${ishouBin}/bin/ishou"; };
          css      = mkRenderApp "css";
          tailwind = mkRenderApp "tailwind";
          scss     = mkRenderApp "scss";
          rust     = mkRenderApp "rust";
          json     = mkRenderApp "json";
          glsl     = mkRenderApp "glsl";
          ghostty  = mkRenderApp "ghostty";
          tui      = mkRenderApp "tui";
          svg      = mkRenderApp "svg";
          render-all = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "ishou-render-all" ''
              set -e
              export PATH=${binPath}:$PATH
              exec ${ishouBin}/bin/ishou render-all "$@"
            ''}/bin/ishou-render-all";
          };
        };
        devShells.default = pkgs.mkShellNoCC {
          buildInputs = devTools;
          shellHook = ''
            echo "ishou — pleme-io design system (home of all tokens)"
            echo ""
            echo "  cargo test                  run token snapshot tests"
            echo "  cargo run -p ishou-cli -- render --target css"
            echo "  nix run .#render-all -- --out-dir generated/"
          '';
        };
      };
    in {
      packages  = forEach (s: (mkPerSystem s).packages);
      apps      = forEach (s: (mkPerSystem s).apps);
      devShells = forEach (s: (mkPerSystem s).devShells);
      overlays.default = final: prev: {
        ishou = self.packages.${final.system}.default;
      };
    };
}
