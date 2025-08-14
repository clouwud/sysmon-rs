{
  description = "Rust System Monitor in TUI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rust-sysmon";
          version = "0.1.0";
          src = ./.;
          cargoSha256 = pkgs.lib.fakeSha256; # run nix build once to get real hash
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.rust-analyzer
            pkgs.clippy
            pkgs.rustfmt
            pkgs.pkg-config
          ];
        };
      }
    );
}
