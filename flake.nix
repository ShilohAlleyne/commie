{
    description = "A very basic rust dev env flake";

    inputs = {
        nixpkgs-stable.url = "github:NixOS/nixpkgs";
        nixpkgs-unstable.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = { self , nixpkgs-stable, nixpkgs-unstable, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
        let
            rust-overlay = (import (builtins.fetchTarball {
                url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
                sha256 = "sha256:0vic9rv7p1yr390j0bq9df03glc37zqsx67msazsxz82114q5jgg";
            }));
            pkgs-stable = import nixpkgs-stable {
                inherit system;
                overlays = [ rust-overlay ];
            };
            pkgs-unstable = import nixpkgs-unstable {
                inherit system;
                overlays = [ rust-overlay ];
            };

            # Building nixpkgs
            crateMeta = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            crateName = crateMeta.package.name;
        in
        {
            devShells.default = pkgs-stable.mkShell {
                buildInputs = [
                    (pkgs-stable.rust-bin.stable.latest.default.override {
                        extensions = ["rust-src"];
                    })
                    pkgs-stable.cargo
                    pkgs-stable.rustup
                ];
                shellHook = ''
                    rustup component add rust-analyzer
                '';
            };

            # Build with nix
            packages.${system}.${crateName} = pkgs-stable.rustPlatform.buildRustPackage {
                src = ./.;
                cargoLock = {
                    lockFile = ./Cargo.lock;
                };
            };

            apps.${system}.${crateName} = {
                type = "app";
                program = "${self.packages.${system}.${crateName}}/bin/${crateName}";
            };
        }
    );
}
