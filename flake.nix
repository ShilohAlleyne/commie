{
    description = "A very basic Rust dev env flake";

    inputs = {
        nixpkgs-stable.url = "github:NixOS/nixpkgs";
        nixpkgs-unstable.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = { self, nixpkgs-stable, nixpkgs-unstable, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
        let
            rust-overlay = import (builtins.fetchTarball {
                url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
                sha256 = "sha256:0vic9rv7p1yr390j0bq9df03glc37zqsx67msazsxz82114q5jgg";
            });

            pkgs = import nixpkgs-stable {
                inherit system;
                overlays = [ rust-overlay ];
            };

            rustPackage = pkgs.rustPlatform.buildRustPackage {
                pname = "commie";
                version = "0.1.0";
                src = ./.;
                cargoLock = {
                    lockFile = ./Cargo.lock;
                };
                meta.mainProgram = "commie";
            };
        in
        {
            devShells.default = pkgs.mkShell {
                buildInputs = [
                    (pkgs.rust-bin.stable.latest.default.override {
                        extensions = [ "rust-src" ];
                    })
                    pkgs.cargo
                    pkgs.rustup
                ];
                shellHook = ''
                    rustup component add rust-analyzer
                '';
            };

            packages.default = rustPackage;

            apps.default = {
                type = "app";
                program = pkgs.lib.getExe rustPackage;
            };
        }
    );
}
