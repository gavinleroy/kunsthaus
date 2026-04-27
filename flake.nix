{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            !(pkgs.lib.hasSuffix ".pdf" path) &&
            !(pkgs.lib.hasSuffix ".tar.gz" path) &&
            !(pkgs.lib.hasSuffix ".zip" path) &&
            (
              (craneLib.filterCargoSources path type) ||
              (pkgs.lib.hasSuffix ".html" path) ||
              (pkgs.lib.hasSuffix ".scss" path) ||
              (pkgs.lib.hasSuffix ".toml" path) ||
              (pkgs.lib.hasInfix "/assets/" path)
            );
        };

        commonArgs = {
          inherit src;
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          doCheck = false;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        kunsthaus = craneLib.buildTrunkPackage (commonArgs // {
          inherit cargoArtifacts;
          trunkIndexPath = "index.html";
          trunkExtraBuildArgs = "--public-url .";
          nativeBuildInputs = with pkgs; [
            dart-sass
            binaryen
          ];
        });
      in {
        packages.default = kunsthaus;

        devShell = with pkgs; mkShell {
          nativeBuildInputs = [ pkg-config ];
          buildInputs = [
            llvmPackages_latest.llvm
            llvmPackages_latest.lld
            toolchain
            rust-analyzer
            poppler-utils
            trunk
            binaryen
            dart-sass
          ] ++ lib.optionals stdenv.isLinux [
            alsa-lib.dev
            udev.dev
          ];
          RUSTC_LINKER = "${llvmPackages.clangUseLLVM}/bin/clang";
        };
      });
}
