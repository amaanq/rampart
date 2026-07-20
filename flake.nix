{
  description = "Forward-only email alias manager";

  outputs =
    args:
    let
      inputs = (import ./.tack) { overrides = args.tackOverrides or { }; };
      inherit (inputs) fenix nixpkgs;
      inherit (nixpkgs) lib;

      forEachSystem =
        f:
        lib.genAttrs lib.systems.doubles.linux (
          system:
          let
            pkgs = nixpkgs.legacyPackages.${system} or (import nixpkgs { inherit system; });
            fenixPackages = fenix.packages.${system} or null;
            fenixToolchain = if fenixPackages == null then null else fenixPackages.stable.toolchain;
            toolchain =
              if fenixToolchain == null then
                with pkgs;
                [
                  cargo
                  rustc
                  rustfmt
                ]
              else
                [ fenixToolchain ];
          in
          f {
            inherit pkgs;
            codegenInputs = [
              pkgs.postgresql
              pkgs.cornucopia
              pkgs.taplo
            ]
            # Cornucopia needs rustfmt on PATH to produce stable output.
            ++ toolchain;
            rustfmt = if fenixPackages == null then pkgs.rustfmt else fenixPackages.latest.rustfmt;
            rustPlatform =
              if fenixToolchain == null then
                pkgs.rustPlatform
              else
                pkgs.makeRustPlatform {
                  cargo = fenixToolchain;
                  rustc = fenixToolchain;
                };
            nativeDeps =
              lib.optionals
                (
                  pkgs.stdenv.hostPlatform.isLinux
                  && (pkgs.stdenv.hostPlatform.isx86_64 || pkgs.stdenv.hostPlatform.isAarch64)
                )
                [
                  pkgs.wild
                  pkgs.clang
                ];
          }
        );
    in
    {
      nixosModules = {
        rampart = ./nix/module.nix;
        default = ./nix/module.nix;
      };

      packages = forEachSystem (
        { pkgs, rustPlatform, ... }:
        {
          default = pkgs.callPackage ./nix/package.nix { inherit rustPlatform; };
        }
      );

      devShells = forEachSystem (
        {
          nativeDeps,
          codegenInputs,
          pkgs,
          rustfmt,
          ...
        }:
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              rustfmt
            ]
            ++ codegenInputs
            ++ [
              pkgs.pkg-config
            ]
            ++ nativeDeps;
            buildInputs = [
              pkgs.cargo-watch
              pkgs.cargo-deny
              pkgs.swaks
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
        {
          codegenInputs,
          pkgs,
          rustPlatform,
          rustfmt,
          ...
        }:
        {
          build = pkgs.callPackage ./nix/package.nix { inherit rustPlatform; };
          codegen =
            let
              script = pkgs.writeScriptBin "codegen-check" /* nu */ ''
                #!${lib.getExe pkgs.nushell}

                let pgdata = ($env.TMPDIR | path join pgdata)
                let socket = ($env.TMPDIR | path join socket)
                let work = ($env.TMPDIR | path join work)

                mkdir $socket
                ^initdb -D $pgdata --no-locale --encoding=UTF8 --auth=trust
                ^pg_ctl -D $pgdata -l ($env.TMPDIR | path join server.log) -o $"-k ($socket) -c listen_addresses=" start
                ^createdb -h $socket rampart_check

                ^psql -v ON_ERROR_STOP=1 -h $socket -d rampart_check -f ${./crates/rampart/schema.sql}

                mkdir $work
                cp -r ${./queries} ($work | path join queries)
                cp ${./cornucopia.toml} ($work | path join cornucopia.toml)
                cp ${./.rustfmt.toml} ($work | path join .rustfmt.toml)
                cd $work
                ^cornucopia live $"host=($socket) dbname=rampart_check"
                ^taplo fmt --config ${./.taplo.toml} crates/rampart-codegen/Cargo.toml

                let difference = (^diff -ru ${./crates/rampart-codegen} crates/rampart-codegen | complete)
                if $difference.exit_code != 0 {
                  print $difference.stdout
                  print -e $difference.stderr
                  error make { msg: "Generated bindings are out of date. Run `cornucopia live $RAMPART_TEST_DB_URL` from the dev shell." }
                }
                touch $env.out
              '';
            in
            derivation {
              name = "codegen-check";
              system = pkgs.stdenv.hostPlatform.system;
              builder = "${script}/bin/codegen-check";
              PATH = lib.makeBinPath (
                [
                  rustfmt
                  pkgs.diffutils
                ]
                ++ codegenInputs
              );
              preferLocalBuild = true;
              allowSubstitutes = false;
            };
        }
      );

      formatter = forEachSystem (
        {
          codegenInputs,
          pkgs,
          rustfmt,
          ...
        }:
        pkgs.writeScriptBin "rampart-fmt" /* nu */ ''
          #!${lib.getExe pkgs.nushell}

          $env.PATH = (
            $env.PATH
            | prepend ("${
              lib.makeBinPath (
                [
                  rustfmt
                  pkgs.nixfmt
                ]
                ++ codegenInputs
              )
            }"
            | split row ":")
          )

          mut root = $env.PWD
          while not ($root | path join flake.nix | path exists) {
            if $root == "/" { error make { msg: "not inside the rampart repo" } }
            $root = ($root | path dirname)
          }
          cd $root

          ^cargo fmt --all
          ^taplo fmt --config .taplo.toml
          ^nixfmt flake.nix .tack/default.nix ...(glob nix/*.nix)
        ''
      );
    };
}
