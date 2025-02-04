test: {
  pkgs,
  testModules,
  ...
}: let
  inherit (pkgs) lib;
  nixos-lib = import (pkgs.path + "/nixos/lib") {};
in
  (
    nixos-lib.runTest {
      hostPkgs = pkgs;
      # This speeds up the evaluation by skipping evaluating documentation
      defaults.documentation.enable = lib.mkDefault false;
      imports = [test];
      enableOCR = true;
      nodes = {
        c = _: {
          imports = [] ++ testModules;
          users.users.test = {
            password = "test";
            isNormalUser = true;
          };
          services = {
            xserver.enable = true;
          };
          # reference dm
          # services.displayManager.sddm = {
          #   enable = true;
          #   wayland.enable = true;
          # };
          eldolfin.services.mydm = {
            enable = true;
            wayland.enable = true;
          };
          virtualisation = {
            memorySize = 4096;
            diskSize = 8192;
            cores = 4;
            resolution = {
              x = 1920;
              y = 1080;
            };
          };
        };
      };
    }
  )
  .config
  .result
