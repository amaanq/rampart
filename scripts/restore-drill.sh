#!/usr/bin/env bash
# Usage: scripts/restore-drill.sh <dump.sql.gz> [temp-dbname]
#
# Restores a rampart pg_dump into a throwaway database and runs smoke queries.
# Exits non-zero on any failure. Cleans up the temp DB on success.
#
# This is the deploy-day proof that local backups can actually be
# restored. Run it after every backup pipeline change, and on a
# representative dump at least monthly. A backup that has never been
# restored is theoretical — see the README.

set -euo pipefail

DUMP="${1:?path to .sql.gz dump required}"
DBNAME="${2:-rampart_restore_drill_$(date +%s)}"

# NixOS deploys postgres on a Unix socket at /run/postgresql by default
# (cfg.database.host). Prefer it; else honor PGHOST env; else default
# to localhost.
if [[ -z "${PGHOST:-}" && -d /run/postgresql ]]; then
    export PGHOST=/run/postgresql
fi
HOST="${PGHOST:-localhost}"

# Migration version expected to be the latest in the dump. If/when V002
# lands, bump these. Mismatch == real failure (the dump came from an
# older rampart, or someone tampered with refinery_schema_history).
EXPECTED_VERSION=1
EXPECTED_NAME="init"

if [[ ! -f "$DUMP" ]]; then
    echo "restore drill: dump not found: $DUMP" >&2
    exit 2
fi

createdb -h "$HOST" "$DBNAME"
trap "dropdb -h '$HOST' '$DBNAME' 2>/dev/null || true" EXIT

zcat "$DUMP" | psql -h "$HOST" -d "$DBNAME" -v ON_ERROR_STOP=1 -q

# Migration check. psql variable substitution does NOT happen inside
# dollar-quoted PL/pgSQL bodies (the `$$ ... $$` block), so we have to
# do the comparison in bash. -At = unaligned, tuples-only, perfect for
# scripted parsing.
actual=$(psql -h "$HOST" -d "$DBNAME" -At -v ON_ERROR_STOP=1 -c \
    "SELECT version || '|' || name FROM refinery_schema_history \
     ORDER BY version DESC LIMIT 1")
if [[ -z "$actual" ]]; then
    echo "restore drill: no migrations in dump" >&2
    exit 1
fi
expected="${EXPECTED_VERSION}|${EXPECTED_NAME}"
if [[ "$actual" != "$expected" ]]; then
    echo "restore drill: expected migration '$expected', got '$actual'" >&2
    exit 1
fi

# Row counts (informational only; dump may legitimately be empty).
psql -h "$HOST" -d "$DBNAME" -v ON_ERROR_STOP=1 -c "
    SELECT 'users    ' AS table, COUNT(*) FROM \"user\"
    UNION ALL SELECT 'mailbox  ', COUNT(*) FROM mailbox
    UNION ALL SELECT 'aliases  ', COUNT(*) FROM alias
    UNION ALL SELECT 'sieve_v  ', COUNT(*) FROM rampart_sieve_lookup;
"

echo "restore drill: OK ($DUMP into $DBNAME)"
