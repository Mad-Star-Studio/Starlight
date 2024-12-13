let
  # Pinned nixpkgs, 24.11
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/8b27c1239e5c421a2bbc2c65d52e4a6fbf2ff296.tar.gz")) {};

in pkgs.mkShell {
  buildInputs = [ pkgs.cargo pkgs.rustc pkgs.act ];
}