{
  description = "rampart - forward-only email alias manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f system);
      pkgsFor = system: import nixpkgs { inherit system; };

      # nixpkgs ships clorinde 1.4.1, which lacks `attributes-nullable`
      # and silently emits the wrong serde adapter on Option<OffsetDateTime>
      # fields. Pin to post-1.4.1 git until nixpkgs ships >= 1.4.2.
      clorindeFor =
        system:
        let
          pkgs = pkgsFor system;
          src = pkgs.fetchFromGitHub {
            owner = "halcyonnouveau";
            repo = "clorinde";
            rev = "e7354d2eef7b19c36d461f66aa272afee7cec05c";
            hash = "sha256-vCY/H1fBKg5401mTldvRYnaMHkgJ+uIyEMfGE40WbvM=";
          };
        in
        pkgs.clorinde.overrideAttrs (old: {
          version = "1.4.1-git-e7354d2";
          inherit src;
          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            name = "clorinde-1.4.1-git-vendor";
            inherit src;
            hash = "sha256-Idrn6VEOk1ByDaHrzZRI3qUC5IMlnCRROrEyQ7O47so=";
          };
          # `clorinde --version` still prints 1.4.1 — Cargo.toml not bumped on main.
          doInstallCheck = false;
        });
    in
    {
      # Named `rampart`, not `default`: ~/dotfiles auto-imports every input's
      # `nixosModules.default` into every host. Consumers must explicitly
      # `imports = [ inputs.rampart.nixosModules.rampart ]`.
      nixosModules.rampart = ./nix/module.nix;

      packages = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
          fenixPkgs = fenix.packages.${system};
          toolchain = fenixPkgs.stable.toolchain;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          # `nix build .#default` from a dev shell. NixOS hosts with a
          # cross-configured pkgs use `services.rampart.package` (consumes the
          # host's pkgs) instead, so this stays a native build.
          default = pkgs.callPackage ./nix/package.nix {
            inherit rustPlatform;
          };
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
          fenixPkgs = fenix.packages.${system};
          toolchain = fenixPkgs.stable.toolchain;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              toolchain
              pkgs.pkg-config
            ];
            buildInputs = [
              pkgs.openssl
              pkgs.postgresql_16
              pkgs.cargo-watch
              pkgs.swaks
              (clorindeFor system)
            ];
            # Default to a local-socket dev DB so DB-backed tests don't
            # silently skip. Operator creates it once: `createdb rampart_test`.
            shellHook = ''
              : "''${RAMPART_TEST_DB_URL:=host=/tmp dbname=rampart_test}"
              export RAMPART_TEST_DB_URL
            '';
          };
        }
      );

      # Fails if db/rampart-codegen/ drifts from queries/ × migrations/.
      checks = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
          clorinde = clorindeFor system;
        in
        {
          codegen-up-to-date =
            pkgs.runCommand "rampart-codegen-up-to-date"
              {
                nativeBuildInputs = [
                  pkgs.postgresql_16
                  pkgs.diffutils
                  # clorinde reformats output via rustfmt-on-PATH; without
                  # it the check emits unformatted output and false-positives.
                  fenix.packages.${system}.stable.toolchain
                  clorinde
                ];
              }
              ''
                set -euo pipefail
                export PGDATA=$TMPDIR/pgdata
                export PGHOST=$TMPDIR/sock
                mkdir -p "$PGHOST"
                initdb -D "$PGDATA" --no-locale --encoding=UTF8 --auth=trust >/dev/null
                {
                  echo "unix_socket_directories = '$PGHOST'"
                  echo "listen_addresses = '''"
                } >> "$PGDATA/postgresql.conf"
                pg_ctl -D "$PGDATA" -l "$TMPDIR/server.log" start
                createdb -h "$PGHOST" rampart_check
                psql -h "$PGHOST" -d rampart_check -f ${./migrations/V001__init.sql} >/dev/null

                mkdir -p "$TMPDIR/work"
                cp -r ${./queries} "$TMPDIR/work/queries"
                cp ${./clorinde.toml} "$TMPDIR/work/clorinde.toml"
                (cd "$TMPDIR/work" && clorinde live "host=$PGHOST dbname=rampart_check")
                pg_ctl -D "$PGDATA" stop -m fast >/dev/null

                if ! diff -ru ${./db/rampart-codegen} "$TMPDIR/work/db/rampart-codegen"; then
                  echo ""
                  echo "ERROR: db/rampart-codegen/ is out of date relative to queries/*.sql + migrations/."
                  echo "Regenerate by running, from a dev shell:"
                  echo "  clorinde live \"\$RAMPART_TEST_DB_URL\""
                  echo "and commit the resulting db/rampart-codegen/ tree."
                  exit 1
                fi
                touch $out
              '';
        }
      );

      formatter = forEachSystem (system: (pkgsFor system).nixfmt);
    };
}
