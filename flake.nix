{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # We want to use packages from the binary cache
    flake-utils.url = "github:numtide/flake-utils";
    gitignore = { url = "github:hercules-ci/gitignore.nix"; flake = false; };
  };

  outputs = inputs@{ self, nixpkgs, flake-utils, ... }:
  flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: let
    pkgs = nixpkgs.legacyPackages.${system};
    gitignoreSrc = pkgs.callPackage inputs.gitignore { };
  in rec {
    packages.starlight_engine = pkgs.callPackage ./default.nix { inherit gitignoreSrc; };

    legacyPackages = packages;

    defaultPackage = packages.starlight_engine;

    devShell = pkgs.mkShell rec {
      CARGO_INSTALL_ROOT = "${toString ./.}/.cargo";

      buildInputs = with pkgs; [ cargo rustc git 
        udev
        alsa-lib vulkan-loader
        xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
        libxkbcommon wayland # To use the wayland feature
      ];
      nativeBuildInputs = [ pkgs.pkg-config ];
      env = {
        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
      };
    };
  });
}