{
  description = "rampart - forward-only email alias manager";

  outputs =
    { ... }@args:
    let
      inputs = (import ./.tack) { overrides = args.tackOverrides or { }; };
      inherit (inputs) fenix nixpkgs;
      forEachSystem = nixpkgs.lib.genAttrs nixpkgs.lib.systems.doubles.linux;
      pkgsFor = system: nixpkgs.legacyPackages.${system} or (import nixpkgs { inherit system; });

      hasFenix = system: fenix.packages ? ${system};

      # wild + clang are only used on Linux tier-1 arches
      hasWild = plat: plat.isLinux && (plat.isx86_64 || plat.isAarch64);
      nativeDepsFor =
        system:
        let
          pkgs = pkgsFor system;
        in
        nixpkgs.lib.optionals (hasWild pkgs.stdenv.hostPlatform) [
          pkgs.wild
          pkgs.clang
        ];
      nightlyRustfmtFor =
        system:
        if hasFenix system then fenix.packages.${system}.latest.rustfmt else (pkgsFor system).rustfmt;
      rustPlatformFor =
        system:
        let
          pkgs = pkgsFor system;
          toolchain = fenix.packages.${system}.stable.toolchain;
        in
        if hasFenix system then
          pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          }
        else
          pkgs.rustPlatform;
      toolchainFor =
        system:
        if hasFenix system then
          [ fenix.packages.${system}.stable.toolchain ]
        else
          (with pkgsFor system; [
            cargo
            rustc
            rustfmt
          ]);
    in
    {
      nixosModules = {
        rampart = ./nix/module.nix;
        default = ./nix/module.nix;
      };

      packages = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.callPackage ./nix/package.nix {
            rustPlatform = rustPlatformFor system;
          };
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              (nightlyRustfmtFor system)
            ]
            ++ toolchainFor system
            ++ [
              pkgs.pkg-config
            ]
            ++ nativeDepsFor system;
            buildInputs = [
              pkgs.openssl
              pkgs.postgresql_16
              pkgs.cargo-watch
              pkgs.cargo-deny
              pkgs.swaks
              pkgs.cornucopia
              pkgs.taplo
            ];
            # Use a local socket so DB-backed tests don't silently skip.
            shellHook = /* sh */ ''
              : "''${RAMPART_TEST_DB_URL:=host=/tmp dbname=rampart_test}"
              export RAMPART_TEST_DB_URL
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}:''${LD_LIBRARY_PATH:-}"
            '';
          };
        }
      );

      checks = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          codegen-up-to-date =
            pkgs.runCommand "rampart-codegen-up-to-date"
              {
                nativeBuildInputs = [
                  pkgs.postgresql_16
                  pkgs.diffutils
                  pkgs.cornucopia
                ]
                # Cornucopia needs rustfmt on PATH to produce stable output.
                ++ toolchainFor system;
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
                for migration in ${./migrations}/*.sql; do
                  psql -h "$PGHOST" -d rampart_check -f "$migration" >/dev/null
                done

                mkdir -p "$TMPDIR/work"
                cp -r ${./queries} "$TMPDIR/work/queries"
                cp ${./cornucopia.toml} "$TMPDIR/work/cornucopia.toml"
                (cd "$TMPDIR/work" && cornucopia live "host=$PGHOST dbname=rampart_check")
                pg_ctl -D "$PGDATA" stop -m fast >/dev/null

                if ! diff -ru ${./db/rampart-codegen} "$TMPDIR/work/db/rampart-codegen"; then
                  echo ""
                  echo "ERROR: db/rampart-codegen/ is out of date relative to queries/*.sql + migrations/."
                  echo "Regenerate by running, from a dev shell:"
                  echo "  cornucopia live \"\$RAMPART_TEST_DB_URL\""
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
