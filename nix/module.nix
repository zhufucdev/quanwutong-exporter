{
  pkgs,
  config,
  lib,
  ...
}:
let
  cfg = config.services.quanwutong-exporter;
  staticUser = cfg.user != null && cfg.group != null;
in
{
  options.services.quanwutong-exporter = {
    enable = lib.mkEnableOption "Enable the promethesus exporter daemon";
    package = lib.mkPackageOption pkgs "quanwutong-exporter" { };

    user = lib.mkOption {
      type = with lib.types; nullOr str;
      default = null;
      example = "quanwutong";
      description = ''
        User account under which to run quanwutong-exporter.
        Defaults to [`DynamicUser`](https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html#DynamicUser=) when set to `null`.

        The user will automatically be created, if this option is set to a non-null value.
      '';
    };
    group = lib.mkOption {
      type = with lib.types; nullOr str;
      default = cfg.user;
      defaultText = lib.literalExpression "config.services.quanwutong-exporter.user";
      example = "quanwutong";
      description = ''
        Group under which to run quanwutong-exporter. Only used when `services.quanwutong-exporter.user` is set.

        The group will automatically be created, if this option is set to a non-null value.
      '';
    };

    token = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Token accessibile via MiTM. Prefix with \"file:\" for reading from one.";
    };
    extraEnv = lib.mkOption {
      default = null;
      type = lib.types.nullOr lib.types.envVar;
      description = "Extra environment variables to use.";
    };
  };

  config = lib.mkIf cfg.enable {
    users = lib.mkIf staticUser {
      users.${cfg.user} = {
        inherit (cfg) home;
        isSystemUser = true;
        group = cfg.group;
      };
      groups.${cfg.group} = { };
    };
    systemd.services.quanwutong-exporter = {
      description = "quanwutong prometheus exporter daemon";
      requires = [ "network-online.target" ];
      after = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig =
        lib.optionalAttrs staticUser {
          User = cfg.user;
          Group = cfg.group;
        }
        // {
          ExecStart = lib.getExe cfg.package;
          Environment = [
            (lib.optionalString (cfg.token != null) "TOKEN=${cfg.token}")
          ]
          ++ lib.optional (cfg.extraEnv != null) cfg.extraEnv;
          DynamicUser = true;
        };
    };
  };
}
