{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.rampart;
  usesScriptedLmtpAddress = cfg.lmtp.addToLoopback && !config.networking.useNetworkd;
  inherit (lib)
    mkEnableOption
    mkIf
    mkOption
    types
    optional
    concatStringsSep
    ;
in
{
  options.services.rampart = {
    enable = mkEnableOption "rampart alias manager";

    package = mkOption {
      type = types.package;
      default = pkgs.callPackage ../nix/package.nix { };
      defaultText = lib.literalExpression "pkgs.callPackage \"\${inputs.rampart}/nix/package.nix\" { }";
      description = ''
        The rampart binary. Default cross-compiles via the consuming host's
        pkgs (no qemu-user). Override for a pre-built artifact.
      '';
    };

    user = mkOption {
      type = types.str;
      default = "rampart";
    };

    group = mkOption {
      type = types.str;
      default = "rampart";
    };

    listen = mkOption {
      type = types.str;
      default = "[::1]:8090";
      description = "<host>:<port> that rampart binds for HTTP.";
    };

    lmtp = {
      address = mkOption {
        type = types.str;
        default = "198.18.0.1";
        description = ''
          IPv4 address rampart-worker binds for LMTP. Default 198.18.0.1
          is in the RFC 2544 benchmarking range (never routes publicly)
          but is non-loopback, so stalwart's outbound resolver accepts
          it as a Relay target without patching its is_loopback() guard.
          The module assigns it to `lo`; set `addToLoopback = false` if
          it already lives elsewhere.
        '';
      };
      port = mkOption {
        type = types.port;
        default = 8024;
      };
      addToLoopback = mkOption {
        type = types.bool;
        default = true;
      };
    };

    publicOrigin = mkOption {
      type = types.str;
      example = "https://bunker.rampart.email";
      description = "scheme+host the dashboard is served at; matched exactly against Origin.";
    };

    aliasDomains = mkOption {
      type = types.listOf types.str;
      default = [ ];
      example = [ "addy.example.com" ];
      description = ''
        Seed list unioned with `alias_domain` rows on each
        bootstrap-stalwart run. The DB is the source of truth; this is
        only for pre-seeding a fresh deploy.
      '';
    };

    smtp = {
      host = mkOption {
        type = types.str;
        default = "localhost";
      };
      port = mkOption {
        type = types.port;
        default = 465;
        description = "Implicit TLS (465) vs STARTTLS (587). rampart picks TLS mode from the port.";
      };
      user = mkOption {
        type = types.str;
        default = "rampart-notifier@${cfg.nginx.hostName}";
        description = "SMTP AUTH principal seeded into stalwart's directory.";
      };
      passwordFile = mkOption {
        type = types.path;
        description = "SASL PLAIN password file (typically agenix-materialized).";
      };
      notifierFrom = mkOption {
        type = types.str;
        default = "\"rampart\" <${cfg.smtp.user}>";
        description = "RFC5322 From for transactional mail.";
      };
    };

    backups = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = ''
          Local pg_dump only — pair with off-host pickup
          (borg/restic/scp) to be DR-grade. See the README.
        '';
      };
      destination = mkOption {
        type = types.path;
        default = "/var/lib/rampart/backups";
      };
      schedule = mkOption {
        type = types.str;
        default = "daily";
      };
      retainDays = mkOption {
        type = types.ints.positive;
        default = 30;
      };
    };

    stalwart = {
      jmapBaseUrl = mkOption {
        type = types.str;
        default = "http://127.0.0.1:8080";
      };
      adminUsername = mkOption {
        type = types.str;
        default = "admin";
      };
      adminPasswordFile = mkOption {
        type = types.path;
        description = "Stalwart admin password (agenix). Used by `rampart admin bootstrap-stalwart`.";
      };
      verpKeyFile = mkOption {
        type = types.path;
        description = ''
          ≥32 bytes of entropy for HMAC bounce-VERP signing. Generate via
          `openssl rand -base64 32`; rotation is harmless (in-flight VERPs
          signed with the old key fail verification and silently drop).
        '';
      };
      authservId = mkOption {
        type = types.str;
        default = cfg.nginx.hostName;
        description = ''
          Stalwart's `server.hostname` (the authserv-id it writes on
          its own Authentication-Results). The reply-path worker only
          accepts replies whose AR matches this id with DMARC=pass.
          Set explicitly when stalwart's mail hostname differs from the
          dashboard vhost — mismatch = silent reply-path outage.
        '';
      };
      publicMxHostname = mkOption {
        type = types.str;
        default = cfg.stalwart.authservId;
        description = "Public hostname users publish as the MX target for alias domains.";
      };
    };

    database = {
      name = mkOption {
        type = types.str;
        default = "rampart";
      };
      user = mkOption {
        type = types.str;
        default = "rampart";
      };
      host = mkOption {
        type = types.str;
        default = "/run/postgresql";
      };
      url = mkOption {
        type = types.str;
        default = "host=${cfg.database.host} user=${cfg.database.user} dbname=${cfg.database.name}";
      };
    };

    sieve = {
      outputPath = mkOption {
        type = types.path;
        default = "/var/lib/stalwart-mail/scripts/rampart_rcpt.sieve";
        description = "Path rampart writes the rendered Sieve to. Must be readable by stalwart-mail.";
      };
      stalwartUnit = mkOption {
        type = types.str;
        default = "stalwart.service";
      };
    };

    nginx = {
      enable = mkOption {
        type = types.bool;
        default = true;
      };
      hostName = mkOption {
        type = types.str;
        description = "Public hostname; usually publicOrigin sans scheme.";
      };
      listenAddresses = mkOption {
        type = types.listOf types.str;
        default = [ ];
        description = ''
          Bind nginx only to these (e.g. tailscale interface IP).
          Stronger than a source-IP allowlist alone.
        '';
      };
    };
  };

  config = mkIf cfg.enable {
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      home = "/var/lib/rampart";
      # Needs write on the stalwart scripts dir.
      extraGroups = [ "stalwart-mail" ];
    };
    users.groups.${cfg.group} = { };

    services.postgresql = {
      ensureDatabases = [ cfg.database.name ];
      ensureUsers = [
        {
          name = cfg.database.user;
          ensureDBOwnership = true;
        }
        # Stalwart's Sieve hook authenticates to postgres as the
        # "stalwart-mail" role (matches the Unix user via peer auth).
        # The schema grants this role access to the resolver functions used
        # by Stalwart's Sieve hook. Stalwart's own database and ownership
        # remain in its Nix module.
        {
          name = "stalwart-mail";
        }
      ];
    };

    networking.interfaces.lo.ipv4.addresses = lib.mkIf cfg.lmtp.addToLoopback [
      {
        address = cfg.lmtp.address;
        prefixLength = 32;
      }
    ];

    systemd.tmpfiles.rules = [
      "d /var/lib/rampart 0750 ${cfg.user} ${cfg.group} - -"
      # 2775 = group-writable + setgid so files inherit the stalwart-mail
      # group regardless of rampart's umask.
      "d ${builtins.dirOf cfg.sieve.outputPath} 2775 ${cfg.user} stalwart-mail - -"
    ]
    ++ lib.optionals cfg.backups.enable [
      "d ${cfg.backups.destination} 0700 ${cfg.user} ${cfg.group} - -"
    ];

    systemd.services.rampart = {
      description = "rampart — email alias manager";
      after = [ "postgresql.service" ];
      requires = [ "postgresql.service" ];
      wantedBy = [ "multi-user.target" ];
      # LoadCredential reads the secret once at process start; restart
      # on rotation so the new password is picked up.
      restartTriggers = [
        cfg.smtp.passwordFile
        cfg.stalwart.verpKeyFile
      ];
      serviceConfig = {
        Type = "notify";
        NotifyAccess = "main";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/rampart serve";
        Restart = "on-failure";
        RestartSec = "5s";
        StateDirectory = "rampart";
        LoadCredential = [
          "smtp_password:${cfg.smtp.passwordFile}"
          "stalwart_admin_password:${cfg.stalwart.adminPasswordFile}"
          "verp_key:${cfg.stalwart.verpKeyFile}"
        ];
        ProtectSystem = "strict";
        ProtectHome = "yes";
        PrivateTmp = "yes";
        NoNewPrivileges = "yes";
        RestrictSUIDSGID = "yes";
        RestrictRealtime = "yes";
        RestrictNamespaces = "yes";
        MemoryDenyWriteExecute = "yes";
        LockPersonality = "yes";
        ReadWritePaths = [ (builtins.dirOf cfg.sieve.outputPath) ];
      };
      environment = {
        RAMPART_DATABASE_URL = cfg.database.url;
        RAMPART_LISTEN = cfg.listen;
        RAMPART_PUBLIC_ORIGIN = cfg.publicOrigin;
        RAMPART_STATIC_DIR = "${cfg.package}/share/rampart/static";
        RAMPART_SIEVE_OUTPUT_PATH = toString cfg.sieve.outputPath;
        RAMPART_SMTP_HOST = cfg.smtp.host;
        RAMPART_SMTP_PORT = toString cfg.smtp.port;
        RAMPART_SMTP_USER = cfg.smtp.user;
        RAMPART_SMTP_PASSWORD_FILE = "%d/smtp_password";
        RAMPART_NOTIFIER_FROM = cfg.smtp.notifierFrom;
        RAMPART_VERP_KEY_FILE = "%d/verp_key";
        RAMPART_STALWART_JMAP_BASE_URL = cfg.stalwart.jmapBaseUrl;
        RAMPART_STALWART_ADMIN_USERNAME = cfg.stalwart.adminUsername;
        RAMPART_STALWART_ADMIN_PASSWORD_FILE = "%d/stalwart_admin_password";
        RAMPART_PUBLIC_MX_HOSTNAME = cfg.stalwart.publicMxHostname;
        RUST_LOG = "info,rampart=info,tower_http=info";
      };
    };

    # Hard-required by rampart-worker so a failed bootstrap blocks worker start;
    # systemd's Before= alone only orders, doesn't propagate failure.
    systemd.services.rampart-bootstrap-stalwart = {
      description = "rampart — seed stalwart JMAP registry (idempotent)";
      after = [
        "stalwart.service"
        "rampart-render-sieve.service"
      ];
      requires = [
        "stalwart.service"
        "rampart-render-sieve.service"
      ];
      before = [ "rampart-worker.service" ];
      wantedBy = [ "multi-user.target" ];
      restartTriggers = [
        cfg.smtp.passwordFile
        cfg.stalwart.adminPasswordFile
      ];
      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        RemainAfterExit = true;
        # Bootstrap waits up to ~63s for JMAP ready + 15s/request after.
        # Override systemd's 90s default so a flapping endpoint doesn't
        # cascade-fail the worker.
        TimeoutStartSec = "5min";
        ExecStart = ''
          ${cfg.package}/bin/rampart admin bootstrap-stalwart \
            --jmap-base-url ${cfg.stalwart.jmapBaseUrl} \
            --admin-username ${cfg.stalwart.adminUsername} \
            --admin-password-file %d/stalwart_admin_password \
            --rampart-notifier-password-file %d/smtp_password \
            --rampart-notifier-address ${cfg.smtp.user} \
            --lmtp-address ${cfg.lmtp.address} \
            --lmtp-port ${toString cfg.lmtp.port} \
            --database-url ${lib.escapeShellArg cfg.database.url} \
            --sieve-path ${toString cfg.sieve.outputPath} \
            ${concatStringsSep " " (map (d: "--alias-domain ${lib.escapeShellArg d}") cfg.aliasDomains)}
        '';
        LoadCredential = [
          "stalwart_admin_password:${cfg.stalwart.adminPasswordFile}"
          "smtp_password:${cfg.smtp.passwordFile}"
        ];
      };
    };

    systemd.services.rampart-worker = {
      description = "rampart — LMTP resubmit worker";
      after = [
        "postgresql.service"
        "rampart.service"
        "rampart-bootstrap-stalwart.service"
      ]
      ++ optional usesScriptedLmtpAddress "network-addresses-lo.service";
      requires = [
        "rampart.service"
        "rampart-bootstrap-stalwart.service"
      ]
      ++ optional usesScriptedLmtpAddress "network-addresses-lo.service";
      wantedBy = [ "multi-user.target" ];
      restartTriggers = [
        cfg.smtp.passwordFile
        cfg.stalwart.verpKeyFile
      ];
      serviceConfig = {
        Type = "notify";
        NotifyAccess = "main";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/rampart worker";
        Restart = "on-failure";
        RestartSec = "5s";
        LoadCredential = [
          "smtp_password:${cfg.smtp.passwordFile}"
          "verp_key:${cfg.stalwart.verpKeyFile}"
        ];
        ProtectSystem = "strict";
        ProtectHome = "yes";
        PrivateTmp = "yes";
        NoNewPrivileges = "yes";
        RestrictSUIDSGID = "yes";
        RestrictRealtime = "yes";
        RestrictNamespaces = "yes";
        MemoryDenyWriteExecute = "yes";
        LockPersonality = "yes";
      };
      environment = {
        RAMPART_DATABASE_URL = cfg.database.url;
        RAMPART_PUBLIC_ORIGIN = cfg.publicOrigin;
        RAMPART_LMTP_LISTEN = "${cfg.lmtp.address}:${toString cfg.lmtp.port}";
        RAMPART_LMTP_DRAIN_SECS = "20";
        RAMPART_STALWART_HOSTNAME = cfg.stalwart.authservId;
        RAMPART_SMTP_HOST = cfg.smtp.host;
        RAMPART_SMTP_PORT = toString cfg.smtp.port;
        RAMPART_SMTP_USER = cfg.smtp.user;
        RAMPART_SMTP_PASSWORD_FILE = "%d/smtp_password";
        RAMPART_NOTIFIER_FROM = cfg.smtp.notifierFrom;
        RAMPART_VERP_KEY_FILE = "%d/verp_key";
        RUST_LOG = "info,rampart=info";
      };
    };

    # Pure renderer; does NOT restart stalwart (would deadlock NixOS
    # activation when stalwart itself restarts in the same generation).
    # Domain CRUD via the API renders the file in-process; operator runs
    # `systemctl restart stalwart` when the file actually needs to take
    # effect (or it's picked up at the next stalwart restart).
    systemd.services.rampart-render-sieve = {
      description = "rampart — regenerate Sieve from current alias_domain rows";
      after = [ "postgresql.service" ];
      requires = [ "postgresql.service" ];
      before = [
        cfg.sieve.stalwartUnit
        "rampart-bootstrap-stalwart.service"
      ];
      wantedBy = [ "multi-user.target" ];
      # No RemainAfterExit: returns to inactive after each successful
      # run so `systemctl start` actually re-runs ExecStart.
      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/rampart admin render-sieve --output ${toString cfg.sieve.outputPath}";
      };
      environment.RAMPART_DATABASE_URL = cfg.database.url;
    };

    systemd.services.rampart-gc = {
      description = "rampart — scheduled garbage collection";
      after = [ "postgresql.service" ];
      requires = [ "postgresql.service" ];
      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/rampart admin gc";
      };
      environment = {
        RAMPART_DATABASE_URL = cfg.database.url;
        RUST_LOG = "info,rampart=info";
      };
    };
    systemd.timers.rampart-gc = {
      description = "rampart gc daily";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = "daily";
        Persistent = true;
        RandomizedDelaySec = "30m";
      };
    };

    systemd.services.rampart-backup = lib.mkIf cfg.backups.enable {
      description = "rampart — postgres pg_dump";
      after = [ "postgresql.service" ];
      requires = [ "postgresql.service" ];
      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = pkgs.writeShellScript "rampart-backup" ''
          set -euo pipefail
          ts=$(date -u +%Y%m%dT%H%M%SZ)
          out="${cfg.backups.destination}/rampart-$ts.sql.gz"
          tmp="$out.tmp"
          # Atomic: build to $tmp, mv only on full success.
          trap 'rm -f "$tmp"' EXIT
          ${config.services.postgresql.package}/bin/pg_dump --format=plain --clean --if-exists \
            "${cfg.database.url}" \
            | ${pkgs.gzip}/bin/gzip -9 > "$tmp"
          mv "$tmp" "$out"
          trap - EXIT
          ${pkgs.findutils}/bin/find "${cfg.backups.destination}" \
            -name 'rampart-*.sql.gz' -mtime +${toString cfg.backups.retainDays} -delete
        '';
      };
    };
    systemd.timers.rampart-backup = lib.mkIf cfg.backups.enable {
      description = "rampart backup timer";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = cfg.backups.schedule;
        Persistent = true;
        RandomizedDelaySec = "30m";
      };
    };

    # Stalwart-side config (Domain objects, rampart_rcpt Sieve script,
    # session.rcpt.script, SQL store, DKIM rules) is the operator's
    # responsibility — see the README's Stalwart integration section.
    # Stalwart 0.16 dropped the TOML `settings` tree, so this module can
    # no longer write into it.

    services.nginx.virtualHosts = mkIf cfg.nginx.enable {
      ${cfg.nginx.hostName} = {
        listenAddresses = cfg.nginx.listenAddresses;
        locations."/".proxyPass = "http://${cfg.listen}";
        locations."/".proxyWebsockets = true;
      };
    };
  };
}
