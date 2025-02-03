{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  # xcfg = config.services.xserver;
  # dmcfg = config.services.displayManager;
  cfg = config.eldolfin.services.mydm;
  # xEnv = config.systemd.services.display-manager.environment;
in {
  options.eldolfin.services.mydm = {
    enable = mkEnableOption "Whether to enable mydm as the display manager.";
  };

  config =
    mkIf cfg.enable
    {
      # assertions = [
      #   # {
      #   #   assertion = xcfg.enable || cfg.wayland.enable;
      #   #   message = ''
      #   #     MyDM requires either services.xserver.enable or services.displayManager.sddm.wayland.enable to be true
      #   #   '';
      #   # }
      #   # {
      #   #   assertion = !dmcfg.autoLogin.enable;
      #   #   message = ''
      #   #     MyDM doesn't support autoLogin
      #   #   '';
      #   # }
      # ];
      services.displayManager = {
        enable = true;
        execCmd = "exec /run/current-system/sw/bin/mydm";
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
      users.users.mydm = {
        createHome = true;
        home = "/var/lib/mydm";
        group = "mydm";
        uid = config.ids.uids.mydm;
      };
    };
}
