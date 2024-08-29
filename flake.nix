{
  description = "An experiment in implementing a code editor from scratch";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};
      inherit (pkgs) lib;

      craneLib =
        (crane.mkLib pkgs)
        .overrideToolchain
        (pkgs.rust-bin.stable.latest.default.override
          {extensions = ["rust-src" "rust-analyzer"];});

      commonArgs = {
        src = let
          cargoOrIcon = path: type:
            !(builtins.elem path [./exalted.png ./IosevkaTerm-Regular.ttf])
            || (craneLib.filterCargoSources path type);
        in
          lib.cleanSourceWith {
            src = ./.;
            filter = cargoOrIcon;
            name = "source";
          };
        strictDeps = true;
        nativeBuildInputs = with pkgs; [
          xorg.libxcb
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          pkg-config
        ];
        buildInputs = with pkgs; [
          xorg.libX11
          wayland
          libxkbcommon
        ];
      };

      exalted-crate = craneLib.buildPackage (commonArgs
        // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          LD_LIBRARY_PATH = "/run/opengl-driver/lib/:${lib.makeLibraryPath [pkgs.libGL pkgs.libGLU]}";
        });
    in {
      checks = {exalted-crate = exalted-crate;};
      packages.default = exalted-crate;
      apps.default = flake-utils.lib.mkApp {drv = exalted-crate;};
      devShells.default = craneLib.devShell rec {
        checks = self.checks.${system};
        packages = with pkgs; [
          libGL
          pkg-config
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          wayland
          libxkbcommon
          taplo
        ];
        LD_LIBRARY_PATH =
          builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" packages;
      };
    });
}
