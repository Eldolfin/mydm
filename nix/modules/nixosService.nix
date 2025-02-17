# adapted from https://github.com/NixOS/nixpkgs/blob/nixos-unstable/nixos/modules/services/display-managers/sddm.nix
{self}: {
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  xcfg = config.services.xserver;
  dmcfg = config.services.displayManager;
  cfg = config.eldolfin.services.mydm;
  xEnv = config.systemd.services.display-manager.environment;
  mydm = cfg.package;
  ymlFmt = pkgs.formats.yaml {};
  # mydm = cfg.package.override (old: {
  #   withWayland = cfg.wayland.enable;
  #   withLayerShellQt = cfg.wayland.compositor == "kwin";
  # });
  # commented parts come from sddm config but are not implemented yet
  xserverWrapper = pkgs.writeShellScript "xserver-wrapper" ''
    ${concatMapStrings (n: "export ${n}=\"${getAttr n xEnv}\"\n") (attrNames xEnv)}
    exec systemd-cat -t xserver-wrapper ${xcfg.displayManager.xserverBin} ${toString xcfg.displayManager.xserverArgs} "$@"
  '';

  Xsetup = pkgs.writeShellScript "Xsetup" ''
    ${cfg.setupScript}
    ${xcfg.displayManager.setupCommands}
  '';

  Xstop = pkgs.writeShellScript "Xstop" ''
    ${cfg.stopScript}
  '';
  defaultConfig =
    {
      # General =
      #   {
      #     HaltCommand = "/run/current-system/systemd/bin/systemctl poweroff";
      #     RebootCommand = "/run/current-system/systemd/bin/systemctl reboot";
      #     Numlock =
      #       if cfg.autoNumlock
      #       then "on"
      #       else "none"; # on, off none

      #     # Implementation is done via pkgs/applications/display-managers/sddm/sddm-default-session.patch
      #     DefaultSession = optionalString (
      #       config.services.displayManager.defaultSession != null
      #     ) "${config.services.displayManager.defaultSession}.desktop";

      #     DisplayServer =
      #       if cfg.wayland.enable
      #       then "wayland"
      #       else "x11";
      #   }
      #   // optionalAttrs (cfg.wayland.enable && cfg.wayland.compositor == "kwin") {
      #     GreeterEnvironment = "QT_WAYLAND_SHELL_INTEGRATION=layer-shell";
      #     InputMethod = ""; # needed if we are using --inputmethod with kwin
      #   };

      # Theme =
      #   {
      #     Current = cfg.theme;
      #     ThemeDir = "/run/current-system/sw/share/sddm/themes";
      #     FacesDir = "/run/current-system/sw/share/sddm/faces";
      #   }
      #   // optionalAttrs (cfg.theme == "breeze") {
      #     CursorTheme = "breeze_cursors";
      #     CursorSize = 24;
      #   };

      # Users = {
      #   MaximumUid = config.ids.uids.nixbld;
      #   HideUsers = concatStringsSep "," dmcfg.hiddenUsers;
      #   HideShells = "/run/current-system/sw/bin/nologin";
      # };

      session_dir = "${dmcfg.sessionData.desktops}/share";
      wayland = {
        # EnableHiDPI = cfg.enableHidpi;
        compositor = lib.optionalString cfg.wayland.enable cfg.wayland.compositorCommand;
      };
    }
    // optionalAttrs xcfg.enable {
      x11 = {
        # minimum_vt =
        #   if xcfg.tty != null
        #   then xcfg.tty
        #   else 7;
        server_path = toString xserverWrapper;
        xephyr_path = "${pkgs.xorg.xorgserver.out}/bin/Xephyr";
        session_command = toString dmcfg.sessionData.wrapper;
        session_dir = "${dmcfg.sessionData.desktops}/share/xsessions";
        xauth_path = "${pkgs.xorg.xauth}/bin/xauth";
        display_command = toString Xsetup;
        displaystop_command = toString Xstop;
        # enable_hidpi = cfg.enableHidpi;
      };
    }
    # // optionalAttrs dmcfg.autoLogin.enable {
    #   Autologin = {
    #     User = dmcfg.autoLogin.user;
    #     Session = autoLoginSessionName;
    #     Relogin = cfg.autoLogin.relogin;
    #   };
    # }
    ;

  cfgFile = ymlFmt.generate "config.yml" (lib.recursiveUpdate defaultConfig cfg.settings);
  compositorCmds = {
    # This is basically the upstream default, but with Weston referenced by full path
    # and the configuration generated from NixOS options.
    weston = let
      westonIni = (pkgs.formats.ini {}).generate "weston.ini" {
        libinput = {
          enable-tap = config.services.libinput.mouse.tapping;
          left-handed = config.services.libinput.mouse.leftHanded;
        };
        keyboard = {
          keymap_model = xcfg.xkb.model;
          keymap_layout = xcfg.xkb.layout;
          keymap_variant = xcfg.xkb.variant;
          keymap_options = xcfg.xkb.options;
        };
      };
    in "${getExe pkgs.weston} --shell=kiosk -c ${westonIni}";
  };
in {
  options.eldolfin.services.mydm = {
    enable = mkEnableOption "Whether to enable mydm as the display manager.";
    package = mkPackageOption self.packages.${pkgs.system} ["default"] {};
    wayland = {
      enable = mkEnableOption "Wayland support";

      compositor = mkOption {
        description = "The compositor to use: ${lib.concatStringsSep ", " (builtins.attrNames compositorCmds)}";
        type = types.enum (builtins.attrNames compositorCmds);
        default = "weston";
      };

      compositorCommand = mkOption {
        type = types.str;
        internal = true;
        default = compositorCmds.${cfg.wayland.compositor};
        description = "Command used to start the selected compositor";
      };
    };
    settings = mkOption {
      type = ymlFmt.type;
      default = {};
      description = ''
        Extra settings merged in and overwriting defaults in mydm/config.conf.
      '';
    };
    setupScript = mkOption {
      type = types.str;
      default = "";
      example = ''
        # workaround for using NVIDIA Optimus without Bumblebee
        xrandr --setprovideroutputsource modesetting NVIDIA-0
        xrandr --auto
      '';
      description = ''
        A script to execute when starting the display server. DEPRECATED, please
        use {option}`services.xserver.displayManager.setupCommands`.
      '';
    };

    stopScript = mkOption {
      type = types.str;
      default = "";
      description = ''
        A script to execute when stopping the display server.
      '';
    };
  };

  config =
    mkIf cfg.enable
    {
      assertions = [
        {
          assertion = xcfg.enable || cfg.wayland.enable;
          message = ''
            MyDM requires either services.xserver.enable or eldolfin.services.mydm.wayland.enable to be true
          '';
        }
        {
          assertion = !dmcfg.autoLogin.enable;
          message = ''
            MyDM doesn't support autoLogin
          '';
        }
      ];
      services = {
        dbus.packages = [mydm];
        xserver = {
          displayManager.lightdm.enable = false;
          # To enable user switching, allow sddm to allocate TTYs/displays dynamically.
          tty = null;
          display = null;
        };
        displayManager = {
          enable = true;
          execCmd = "exec ${mydm}/bin/mydm";
        };
      };
      systemd.services.display-manager.enable = true;
      environment = {
        etc."mydm/config.yml".source = cfgFile;
        # pathsToLink = [
        #   "/share/sddm"
        # ];
        systemPackages = [mydm];
      };
      security.pam.services = {
        mydm.text = ''
          auth      substack      login
          account   include       login
          password  substack      login
          session   include       login
        '';

        mydm-greeter.text = ''
          auth     required       pam_succeed_if.so audit quiet_success user = mydm
          auth     optional       pam_permit.so

          account  required       pam_succeed_if.so audit quiet_success user = mydm
          account  sufficient     pam_unix.so

          password required       pam_deny.so

          session  required       pam_succeed_if.so audit quiet_success user = mydm
          session  required       pam_env.so conffile=/etc/pam/environment readenv=0
          session  optional       ${config.systemd.package}/lib/security/pam_systemd.so
          session  optional       pam_keyinit.so force revoke
          session  optional       pam_permit.so
        '';
      };
      users = {
        users.mydm = {
          createHome = true;
          home = "/var/lib/mydm";
          group = "mydm";
          isSystemUser = true;
        };
        groups = {
          mydm = {};
          video = {members = ["mydm"];};
        };
      };
      systemd = {
        # tmpfiles.packages = [mydm];

        # services.mydm = {
        #   enable = true;
        #   environment = {
        #     RUST_LOG = "debug";
        #   };
        #   aliases = ["display-manager.service"];
        #   description = "Very Simple Desktop Manager written in rust";
        #   partOf = ["graphical.target"];
        #   after = [
        #     "systemd-user-sessions.service"
        #     "getty@tty7.service"
        #     "plymouth-quit.service"
        #     "systemd-logind.service"
        #   ];
        #   conflicts = [
        #     "getty@tty7.service"
        #   ];
        #   startLimitBurst = 2;
        #   startLimitIntervalSec = 30;
        #   # serviceConfig = {
        #   #   Restart = "always";
        #   #   ExecStart = "${mydm}/bin/mydm";
        #   # };
        # };
      };
    };
}
