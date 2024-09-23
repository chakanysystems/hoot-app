{ pkgs ? import <nixpkgs> { }
}:
with pkgs;

let
  x11libs = lib.makeLibraryPath [ xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi libglvnd vulkan-loader vulkan-validation-layers libxkbcommon ];
in
mkShell ({
  nativeBuildInputs = [
    rustup
    rustfmt
    libiconv
    pkg-config
    fontconfig
  ] ++ lib.optional stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.OpenGL
    darwin.apple_sdk.frameworks.CoreServices
    darwin.apple_sdk.frameworks.AppKit
  ];

} // (
  lib.optionalAttrs (!stdenv.isDarwin) {
    LD_LIBRARY_PATH = "${x11libs}";
  }
))
