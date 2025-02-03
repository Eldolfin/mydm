{
  description = "Description for the project";

  inputs = {
    devenv-root = {
      url = "file+file:///dev/null";
      flake = false;
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    devenv.url = "github:cachix/devenv";
    nix2container.url = "github:nlewo/nix2container";
    nix2container.inputs.nixpkgs.follows = "nixpkgs";
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = inputs @ {
    flake-parts,
    devenv-root,
    flake-utils,
    crane,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
      ];
      systems = ["x86_64-linux" "i686-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"];

      perSystem = {
        config,
        pkgs,
        lib,
        # self',
        # inputs',
        # system,
        ...
      }: let
        craneLib = crane.mkLib pkgs;
        environmentVars = {
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          LD_LIBRARY_PATH = "${lib.makeLibraryPath commonArgs.buildInputs}:$LD_LIBRARY_PATH";
        };

        commonArgs = {
          src = let
            unfilteredRoot = ./.; # The original, unfiltered source
          in
            lib.fileset.toSource {
              root = unfilteredRoot;
              fileset = lib.fileset.unions [
                (craneLib.fileset.commonCargoSources unfilteredRoot)
                (lib.fileset.maybeMissing ./assets)
              ];
            };

          strictDeps = true;

          buildInputs = with pkgs; [
            libclang.lib
            # linux-pam
            libGL
            libxkbcommon
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            pam
            rustPlatform.bindgenHook
          ];
          env = environmentVars;
        };

        mydm = craneLib.buildPackage (commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          });
      in {
        packages.default = mydm;

        apps.default = flake-utils.lib.mkApp {
          drv = mydm;
        };

        devenv.shells.default = {
          devenv.root = let
            devenvRootFileContent = builtins.readFile devenv-root.outPath;
          in
            pkgs.lib.mkIf (devenvRootFileContent != "") devenvRootFileContent;

          name = "mydm";

          imports = [
            # This is just like the imports in devenv.nix.
            # See https://devenv.sh/guides/using-with-flake-parts/#import-a-devenv-module
            # ./devenv-foo.nix
          ];

          packages = with pkgs;
            [
              config.packages.default
              git
              entr
              cargo
              rust-analyzer
              rustPackages.clippy
              rustc
              rustfmt
              weston
            ]
            ++ commonArgs.buildInputs;

          enterShell = ''
            echo hello
          '';

          processes.watch.exec = "just weston-watch";
          env = environmentVars;
        };
      };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.
      };
    };
}
