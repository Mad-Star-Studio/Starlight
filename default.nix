{ pkgs ? import <nixpkgs> { }
, stdenv ? pkgs.stdenv
, lib ? stdenv.lib
# A set providing `buildRustPackage :: attrsets -> derivation`
, rustPlatform ? pkgs.rustPlatform
, fetchFromGitHub ? pkgs.fetchFromGitHub
, gitignoreSrc ? null
, pkgconfig ? pkgs.pkgconfig
, gtk3 ? pkgs.gtk3
, glib ? pkgs.glib
, gobject-introspection ? pkgs.gobject-introspection
}:

let
  gitignoreSource =
    if gitignoreSrc != null
    then gitignoreSrc.gitignoreSource
    else (import (fetchFromGitHub {
      owner = "hercules-ci";
      repo = "gitignore";
      rev = "c4662e662462e7bf3c2a968483478a665d00e717";
      sha256 = "0jx2x49p438ap6psy8513mc1nnpinmhm8ps0a4ngfms9jmvwrlbi";
    }) { inherit lib; }).gitignoreSource;
in
rustPlatform.buildRustPackage rec {
  pname = "starlight_engine";
  version = "0.0.1";

  src = gitignoreSource ./.;

  buildInputs = with pkgs; [
    udev alsa-lib vulkan-loader
    xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
    libxkbcommon wayland # To use the wayland feature
    renderdoc
  ];
  nativeBuildInputs = with pkgs; [ pkg-config ];

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
  '';

  meta = with stdenv.lib; {
    homepage = "";
    description = "";
    license = "MIT License";
  };
}