{ nixpkgs ? <nixpkgs>, sources ? import ./sources.nix { }
, system ? builtins.currentSystem }:

let
  mozOverlay = import sources.nixpkgsMoz;
  pkgs = import nixpkgs {
    inherit system;
    overlays = [ (_: _: { crate2nix = import sources.crate2nix { }; }) ];
  };
in pkgs.mkShell {
  name = "hakkero-shell";
  nativeBuildInputs = with pkgs; [ niv nixfmt crate2nix qemu_kvm rustup zlib ];
  buildInputs = with pkgs; [ zlib ];
}
