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

if [[ ! -f "$DUMP" ]]; then
    echo "restore drill: dump not found: $DUMP" >&2
    exit 2
fi

createdb -h "$HOST" "$DBNAME"
trap "dropdb -h '$HOST' '$DBNAME' 2>/dev/null || true" EXIT

zcat "$DUMP" | psql -h "$HOST" -d "$DBNAME" -v ON_ERROR_STOP=1 -q

# Confirm the dump contains the canonical schema objects and extension API
# columns. This catches partial or stale backups without relying on a version
# table.
schema_ok=$(psql -h "$HOST" -d "$DBNAME" -At -v ON_ERROR_STOP=1 -c \
    "SELECT
         to_regclass('public.alias') IS NOT NULL
         AND to_regclass('public.api_idempotency') IS NOT NULL
         AND to_regprocedure('public.rampart_resolve_or_create(text)') IS NOT NULL
         AND (
             SELECT COUNT(*) = 4
             FROM information_schema.columns
             WHERE table_schema = 'public'
               AND table_name = 'api_key'
               AND column_name IN ('scopes', 'kind', 'token_prefix', 'expires_at')
         )")
if [[ "$schema_ok" != "t" ]]; then
    echo "restore drill: required schema objects are missing" >&2
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
