{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    nixGL.url = "github:guibou/nixGL";

    nixpkgs-mozilla = { url = "github:mozilla/nixpkgs-mozilla"; };
  };

  outputs = { self, nixpkgs, utils, nixpkgs-mozilla, nixGL }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [ (import nixpkgs-mozilla) nixGL.overlay ];
        };

        toolchain =
      (pkgs.rustChannelOf {
        date = "2024-01-15";
        channel = "nightly";
        sha256 = "sha256-wHrUcmkOYsEcgUBMamGxUKF/G7iu4NGKdK00QMIt20k=";
      }).rust.override {
          extensions = [
            "rust-src"
            "rust-analyzer-preview"
            "clippy-preview"
            "rustfmt-preview"
          ];
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" ];
        };

        bevyDeps = with pkgs; {
          # thanks, https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md#nixos !
          nativeBuildInputs = [ pkg-config llvmPackages.bintools vulkan-loader vulkan-tools 
            openssl # deps for building wasm. we have to cargo install -f wasm-bindgen-cli, the nixpkg is out of sync 
          ];
          buildInputs = [ # wip and not minimal. was trying to get stuff working on my desktop and didn't finish
            udev alsa-lib vulkan-loader
            xorg.libICE xorg.libSM xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi # To use x11 feature
            libxkbcommon wayland # To use wayland feature
            gdk-pixbuf atk pango cairo gtk3-x11 # additional dependencies for voxel-level-editor
          ];
          hook = ''
            export PATH=$PATH:$HOME/.cargo/bin
          '';
        };
      in {
        devShell = with pkgs;
          mkShell rec {
            nativeBuildInputs = [ toolchain bevyDeps.nativeBuildInputs nil mold clang ];
            buildInputs = bevyDeps.buildInputs;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          };
      });
}
