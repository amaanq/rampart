# Consumers pass their own pkgs in. With `nixpkgs.buildPlatform !=
# hostPlatform` this becomes a true cross via the consumer's stdenv; do
# NOT use `pkgsCross.<sys>.rustPlatform` — that's binfmt-qemu emulation.
{
  lib,
  rustPlatform,
  pkg-config,
  patchelf,
  openssl,
}:

rustPlatform.buildRustPackage {
  pname = "rampart";
  version = "0.1.0";
  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    patchelf
  ];
  buildInputs = [
    openssl.dev
    openssl.out
  ];

  doCheck = false;

  # systemd-launched binary serves /static via RAMPART_STATIC_DIR; ServeDir's
  # "static" default doesn't resolve out of cwd.
  postInstall = ''
    mkdir -p $out/share/rampart
    cp -r ${../static} $out/share/rampart/static
  '';

  postFixup = ''
    patchelf --add-rpath ${lib.makeLibraryPath [ openssl.out ]} $out/bin/rampart
  '';

  meta = {
    description = "Forward-only email alias manager";
    mainProgram = "rampart";
    license = lib.licenses.agpl3Plus;
  };
}
