{
  description = "Flake for hakkero";

  inputs = {
    nixCargoIntegration = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs: inputs.nixCargoIntegration.lib.makeOutputs {
    root = ./.;
    buildPlatform = "crate2nix";
    overrides = {
      shell = common: prev:
      let
        cargoMake = { stdenv, fetchzip, autoPatchelfHook }: (stdenv.mkDerivation rec {
          pname = "cargo-make";
          version = "0.34.0";
          
          src = fetchzip {
            url = "https://github.com/sagiegurari/cargo-make/releases/download/${version}/cargo-make-v${version}-x86_64-unknown-linux-musl.zip";
            sha256 = "sha256-XaAVQ9pNhne+ozAO2Ji7A/QHUVDWKYH6bq4UIh4Ua2A=";
          };
          
          nativeBuildInputs = [ autoPatchelfHook ];
          
          installPhase = "install -m755 -D cargo-make $out/bin/cargo-make";
        });
      in
      with common; {
        packages = prev.packages ++ [
          pkgs.qemu
          pkgs.cargo-binutils
          (pkgs.callPackage cargoMake { })
        ];
      };
    };
  };
}
