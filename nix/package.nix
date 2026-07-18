{
  lib,
  rustPlatform,
  stdenv,
  pkg-config,
  patchelf,
  openssl,
  clang,
  wild ? null,
}:
let
  # wild + clang are only used on Linux tier-1 arches
  hasWild =
    stdenv.hostPlatform.isLinux && (stdenv.hostPlatform.isx86_64 || stdenv.hostPlatform.isAarch64);
in
rustPlatform.buildRustPackage {
  pname = "rampart";
  version = "0.1.0";
  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    patchelf
  ]
  ++ lib.optionals hasWild [
    wild
    clang
  ];

  env = lib.optionalAttrs hasWild {
    RUSTFLAGS = "-Clinker=${clang}/bin/clang -Clink-arg=--ld-path=wild";
  };
  buildInputs = [
    openssl.dev
    openssl.out
  ];

  doCheck = false;

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
