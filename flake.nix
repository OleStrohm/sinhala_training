{
  description = "Rust flake with nightly";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/4e6868b1aa3766ab1de169922bb3826143941973";
    bevy_cli.url = "github:CupOfTeaJay/bevy_cli";
  };

  outputs = { flake-utils, rust-overlay, nixpkgs, crane, bevy_cli, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) {
          inherit system overlays;
        };
        inherit (pkgs) lib;

        rustToolchainFor = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-analyzer" "clippy" "rust-src" "rustc-codegen-cranelift-preview" ];
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" ];
        });
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.css" path) ||
            (lib.hasSuffix "\.txt" path) ||
            (lib.hasInfix "/assets/" path) ||
            (lib.hasInfix "/icons/" path) ||
            (lib.hasInfix "/hooks/" path) ||
            (craneLib.filterCargoSources path type);
        };
        buildInputs = with pkgs; [ 
          udev alsa-lib vulkan-loader python312Packages.fonttools
          xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
          libxkbcommon wayland pkg-config brotli bash
        ];
        commonArgs = {
          inherit src buildInputs;
          strictDeps = true;
          cargoExtraArgs = "--no-default-features";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          CARGO_PROFILE = "release";
          #RUSTFLAGS = "-Zfmt-debug=none -Zlocation-detail=none";
        };
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          doCheck = false;
        });

        buildBevyPackage = pkgs.lib.callPackageWith (pkgs // craneLib) ./buildBevyPackage.nix {};
        binWeb = buildBevyPackage (commonArgs // {
          inherit cargoArtifacts;

          bevy_cli = bevy_cli.packages."x86_64-linux".default;
          wasm-bindgen-cli = pkgs.wasm-bindgen-cli_0_2_100;
          useBrotli = true;

          nativeBuildInputs = with pkgs; [ bash ];
        });

        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "sinhala_training";
          tag = "latest";
          contents = with pkgs; [ brotli ];
          config = {
            Cmd = [
              "${pkgs.static-web-server}/bin/static-web-server"
              "--root" "${binWeb}"
              "--port" "8080"
              "--compression-static" "true"
            ];
            # static-web-server -d ./ -a 127.0.0.1 -p 8080 --compression-static true
            ExposedPorts = {
                "8080" = {};
            };
          };
        };
        moldDevShell = craneLib.devShell.override {
          mkShell = pkgs.mkShell.override {
            stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
          };
        };
      in
      with pkgs;
      {
        packages =
          {
            inherit binWeb dockerImage;
            default = binWeb;
          };
          devShells.default = moldDevShell {
            inherit buildInputs;
            LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
            packages = with pkgs; [ trunk wasm-pack binaryen wasm-bindgen-cli_0_2_100 ];
          };
      }
    );
}
