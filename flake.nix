{
    description = "A very basic rust dev env flake";

    inputs = {
        nixpkgs-stable.url = "github:NixOS/nixpkgs";
        nixpkgs-unstable.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    };

    outputs = { self , nixpkgs-stable, nixpkgs-unstable }:
    let
        rust-overlay = (import (builtins.fetchTarball {
            url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
            sha256 = "05xyk469bj6zkvkk4gmc58rkiyavamn4xhfglwkdqlanqiyfwdfz";
        }));
        pkgs-stable = (import nixpkgs-stable {
                system = "x86_64-linux";
                overlays = [ rust-overlay ];
        });
        pkgs-unstable = ( import nixpkgs-unstable {
            system = "x86_64-linux";
            overlays = [ rust-overlay ];
        });
    in
    {
        devShells."x86_64-linux".default = pkgs-stable.mkShell {
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
    };
}

