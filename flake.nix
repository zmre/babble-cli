{
  description = "babble flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, naersk, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rusttoolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        naersk-lib = naersk.lib."${system}";
      in rec {
        # `nix build`
        packages.babble-cli = naersk-lib.buildPackage {
          pname = "babble-cli";
          root = ./.;
        };
        defaultPackage = packages.babble-cli;

        # `nix run`
        apps.babble-cli = flake-utils.lib.mkApp { drv = packages.babble-cli; };
        defaultApp = apps.babble-cli;

        # nix develop
        devShell = pkgs.mkShell {
          buildInputs = [
            #openssl
            #pkg-config
            rusttoolchain
          ];
        };
      });
}
