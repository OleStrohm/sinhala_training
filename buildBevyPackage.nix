{
  lib,
  binaryen,
  removeReferencesToVendoredSourcesHook,
  removeReferencesToRustToolchainHook,
  vendorCargoDeps,
  mkCargoDerivation,
  crateNameFromCargoToml,
  buildDepsOnly,
  brotli,
}:
{
  wasm-bindgen-cli,
  bevy_cli,
  useBrotli ? false,
  ...
}@origArgs:
let
  cleanedArgs = builtins.removeAttrs origArgs [
    "wasm-bindgen-cli"
    "bevy_cli"
    "useBrotli"
  ];

  crateName = crateNameFromCargoToml cleanedArgs;

  # Avoid recomputing values when passing args down
  args = cleanedArgs // {
    pname = cleanedArgs.pname or crateName.pname;
    version = cleanedArgs.version or crateName.version;
    cargoVendorDir = cleanedArgs.cargoVendorDir or (vendorCargoDeps cleanedArgs);
  };
in
mkCargoDerivation (
  args
  // {
    pnameSuffix = "-bevy";

    cargoArtifacts =
      args.cargoArtifacts or (buildDepsOnly (
        args
         // {
           CARGO_BUILD_TARGET = args.CARGO_BUILD_TARGET or "wasm32-unknown-unknown";
           doCheck = args.doCheck or false;
         }
      ));

    buildPhaseCargoCommand =
      args.buildPhaseCargoCommand or ''
        echo "Building project with bevy_cli"
        bevy build web --bundle --wasm-opt -Oz --wasm-opt -all
      '';

    installPhaseCommand =
      args.installPhaseCommand or (''
        echo "Copying files"
        cp -r target/bevy_web/web/${crateName.pname} $out
      ''
      + lib.optionalString useBrotli ''
        echo "Compressing wasm files"
        for FILE in $out/**/*.wasm; do
            echo "Compressing $FILE with brotli"
            ${brotli}/bin/brotli -f "$FILE"
        done
      ''
      );

    # Installing artifacts on a distributable dir does not make much sense
    doInstallCargoArtifacts = args.doInstallCargoArtifacts or false;

    nativeBuildInputs = (args.nativeBuildInputs or [ ]) ++ [
      binaryen
      bevy_cli
      wasm-bindgen-cli
      # Store references are certainly false positives
      removeReferencesToRustToolchainHook
      removeReferencesToVendoredSourcesHook
    ];
  }
)
