# Dear Yuri fans,
# I have no idea how Nix or Nixos works.
# Do you want to know how I make new projects??
# I copy the same flake into every project I work on and just change the name.
# This file has a FAMILY HISTORY.
# But if it sucks, feel free to make a PR to improve it!
# I'm not really a Nix girl, I'm just stuck in this terrible ecosystem.

{
  description = "yuri";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in with pkgs; {
        devShells.default = mkShell rec {
          buildInputs = [
            # Rust
            rustup

            # misc. libraries
            pkg-config

            # needed for RR to work (you won't be able to use the terminal properly otherwise)
            bashInteractive

            # wayland/desktop libraries
            libxkbcommon
            libGL
            fontconfig
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
            wayland
            glib
            gobject-introspection
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            gst_all_1.gst-plugins-good

            # SDL3
            sdl3
            sdl3.dev

            # Vulkan stuff
            vulkan-tools
            vulkan-validation-layers
            vulkan-loader
            vulkan-headers

            # shader tools
            glslang
            shaderc
            spirv-headers
            spirv-cross
            spirv-tools

            # clang. most of this is probably overkill, but I don't know how I would even check...
            llvmPackages_19.bintools
            llvmPackages_19.stdenv
            llvmPackages_19.clang-tools
            llvmPackages_19.lldb
            llvmPackages_19.libllvm
            llvmPackages_19.libcxxClang
            llvmPackages_19.libclang
            llvmPackages_19.lld
            llvmPackages_19.libcxx
          ];

          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          CPATH = "${lib.makeIncludePath buildInputs}";
        };
      });
}
