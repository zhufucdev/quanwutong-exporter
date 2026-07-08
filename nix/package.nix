{
  lib,
  pkgs,
  craneLib,
  rustPlatform,
  pkg-config,
  openssl,
  fetchFromGitHub,
  stdenv,
  version ? "dev",
  zlib,
  ...
}:
let
  nativeBuildInputs = [
    pkg-config
    rustPlatform.bindgenHook
  ];

  buildInputs = [
    openssl
  ];

  # Common args shared between dep-only and full builds
  commonArgs = {
    inherit
      nativeBuildInputs
      buildInputs
      ;

    cargoVendorDir = craneLib.vendorCargoDeps {
      src = craneLib.cleanCargoSource ../.;

      overrideVendorCargoPackage =
        p: drv:
        if p.name == "prometheus-client" && p.version == "0.25.0" then
          drv.overrideAttrs (
            old:
            old
            // {
              patches = [ ../patches/prometheus-client+0.25.0.patch ];
            }
          )
        else
          # Nothing to change, leave the derivations as is
          drv;
    };

    # Tell crane not to run tests in the build phase
    doCheck = false;
  };

  # Build only dependencies first (allows caching the heavy compile step)
  cargoArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      src = craneLib.cleanCargoSource ../.;
      version = "0.0.0";
    }
  );

in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;
    pname = "quanwutong-exporter";
    inherit version;
    src = lib.sources.cleanSourceWith {
      src = ../.;
      filter = craneLib.filterCargoSources;
      name = "source";
    };
    APP_VERSION = version;
    meta = {
      mainProgram = "quanwutong-exporter";
    };
  }
)
