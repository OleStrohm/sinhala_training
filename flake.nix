{
  description = "Rust flake with nightly";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/4e6868b1aa3766ab1de169922bb3826143941973";
  };

  outputs = { self, flake-utils, rust-overlay, nixpkgs, crane, nixpkgs-for-wasm-bindgen }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) {
          inherit system overlays;
        };
        inherit (pkgs) lib;

        rustToolchain = p: p.rust-bin.stable.latest.default.override {
          extensions = [ "rust-analyzer" "clippy" "rust-src" ];
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" ];
        };
        craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchain).overrideScope (_final: _prev: {
          inherit (import nixpkgs-for-wasm-bindgen { inherit system; }) wasm-bindgen-cli;
        });

        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.css" path) ||
            (lib.hasInfix "/assets/" path) ||
            (craneLib.filterCargoSources path type);
        };
        buildInputs = with pkgs; [ 
            udev alsa-lib vulkan-loader
            xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
            libxkbcommon wayland pkg-config
        ];
        commonArgs = {
          inherit src buildInputs;
          strictDeps = true;
          cargoExtraArgs = "--locked --no-default-features";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          doCheck = false;
        });
        binWeb = craneLib.buildTrunkPackage (commonArgs // {
          inherit cargoArtifacts;

          wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override {
            version = "0.2.93";
            hash = "sha256-DDdu5mM3gneraM85pAepBXWn3TMofarVR4NbjMdz3r0=";
            cargoHash = "sha256-birrg+XABBHHKJxfTKAMSlmTVYLmnmqMDfRnmG6g/YQ=";
          };
        });

        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "sinhala_training";
          tag = "latest";
          config = {
            Cmd = [ "${pkgs.python3Minimal}/bin/python3" "-m" "http.server" "--directory" "${binWeb}" "8080" ];
            ExposedPorts = {
                "8080" = {};
            };
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
          devShells.default = mkShell {
              inputsFrom = [ binWeb ];
              buildInputs = with pkgs; [ trunk ];
              LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
          };
      }
    );
}
