{
  description = "si7021-reader firmware for nRF54L15";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
            probe-rs-tools
            just
            git
            lazygit
            commitizen
            minicom
          ];
          buildInputs = with pkgs; [
            libusb1
          ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
            udev
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            with pkgs; [ libusb1 ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ udev ]
          );
        };
      });
}
