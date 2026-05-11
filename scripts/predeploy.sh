#!/usr/bin/env bash
# Predeploy gate. Codex round-4 P2.3: every DB-backed test suite must
# actually run before a deploy — silent-skip-when-unset has masked real
# regressions. This script enforces the hard gate.
#
# Run via `nix develop -c bash scripts/predeploy.sh` so the openssl /
# postgres / cargo toolchain are on PATH and `RAMPART_TEST_DB_URL` defaults
# from the shellHook.
set -euo pipefail

if [[ -z "${RAMPART_TEST_DB_URL:-}" ]]; then
    echo "predeploy: RAMPART_TEST_DB_URL must be set (the nix develop shell exports a default)" >&2
    exit 2
fi

echo "predeploy: cargo fmt --check"
cargo fmt --check

echo "predeploy: cargo build (warnings → errors via -D warnings? not yet)"
cargo build

echo "predeploy: cargo test with RAMPART_REQUIRE_DB_TESTS=1"
RAMPART_REQUIRE_DB_TESTS=1 cargo test

echo "predeploy: ok"
