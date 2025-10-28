{
    description = "A very basic Rust dev env flake";

    inputs = {
        nixpkgs-stable.url   = "github:NixOS/nixpkgs";
        nixpkgs-unstable.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
        rust-overlay.url     = "github:oxalica/rust-overlay";
        flake-utils.url      = "github:numtide/flake-utils";
    };

    outputs = { self, nixpkgs-stable, nixpkgs-unstable, rust-overlay, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
        let
            overlays = [ (import rust-overlay) ];
            pkgs-stable = import nixpkgs-stable {
                inherit system overlays;
            };

            pkgs-unstable = import nixpkgs-unstable {
                inherit system overlays;
            };

            manifest = (pkgs-stable.lib.importTOML ./Cargo.toml).package;

            rustPackage = pkgs-stable.rustPlatform.buildRustPackage {
                inherit (manifest) name version; 

                meta = with pkgs-stable.lib; {
                    description = manifest.description;
                    # Add other optional meta fields:
                    # homepage = manifest.homepage; # if you have this in Cargo.toml
                    license = licenses.mit;
                };

                cargoLock.lockFile = ./Cargo.lock;
                src = pkgs-stable.lib.cleanSource ./.;
            };
        in
        {
            devShells.default = with pkgs-stable; mkShell {
                name = "${manifest.name}-devshell";

                buildInputs = [
                    pkgs-stable.cargo
                    pkgs-stable.rustup
                    pkgs-stable.rust-analyzer
                ];

                shellHook = ''
                    export PATH="${pkgs-stable.rust-analyzer}/bin:$PATH"
                    echo "Using rust-analyzer from Nixpkgs: $(which rust-analyzer)"
                '';
            };

            packages.default = rustPackage;

            apps.default = flake-utils.lib.mkApp {
                drv = rustPackage;
            };
        }
    );
}
