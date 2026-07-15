#!/usr/bin/env bash
# Quick-start for UI work. Starts the preview server (no Postgres,
# SMTP, or auth). All pages render with hardcoded mock data.
#
# Usage: ./scripts/dev-ui.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

export RAMPART_LISTEN="${RAMPART_LISTEN:-127.0.0.1:8090}"

echo "starting preview server on http://$RAMPART_LISTEN"
echo ""

exec cargo watch -x 'run -- preview'
