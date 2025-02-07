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
            # TODO: users shouldn't need to be root to open a desktop...
            extraGroups = [
              "wheel"
            ];
          };
          programs.sway.enable = true;
          services = {
            xserver = {
              enable = true;
              windowManager = {
                i3.enable = true;
              };
              desktopManager = {
                gnome.enable = true;
                xfce = {
                  enable = true;
                  noDesktop = true;
                  enableXfwm = false;
                  enableScreensaver = false;
                };
              };
            };
          };
          # reference dms
          services = {
            # displayManager.sddm = {
            #   enable = true;
            #   wayland.enable = true;
            # };
            # greetd = {
            #   enable = true;
            # };
          };

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
