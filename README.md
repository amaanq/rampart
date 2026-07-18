# rampart

Self-hosted, forward-only email alias manager in Rust, integrated with
Stalwart 0.15.5 via a session.rcpt Sieve script. Single-user. Phase 1.

See `DESIGN.md` for the architecture, `docs/stalwart-integration.md`
for the operator runbook.

## Shape at a glance

- ~1300 LOC Rust, single binary (`rampart serve` / `rampart migrate` / `rampart admin …`).
- Axum HTTP (JSON API + htmx-driven dashboard) gated by basic-auth or
  bearer token with an argon2 verify LRU and an Origin-check CSRF
  middleware on mutating routes.
- Postgres storage via `tokio-postgres` + `deadpool-postgres`. Migrations
  via `refinery`. (No `sqlx` — compile times.)
- Stalwart integration: a rendered Sieve script at
  `/var/lib/stalwart-mail/scripts/rampart_rcpt.sieve` issues a `SELECT` on
  `rampart_sieve_lookup` and either `set "envelope.to"` to the forward target
  or `reject`s. No `redirect`, no catch-all — stalwart applies the
  envelope modification at rcpt.rs:127-133 and routes normally.

## Dev quickstart

```bash
# Toolchain + deps via nix
nix develop
# ... or with plain cargo if you have postgres + rust on PATH:
cargo build

# Ephemeral postgres for tests (requires pg_ctl, initdb, libargon2):
export RAMPART_TEST_DB_URL="host=$(mktemp -d) user=$USER dbname=postgres"
# (bring the server up via initdb/pg_ctl; see /tmp/rampart-run-tests.sh pattern)
cargo test

# Manual end-to-end run
export RAMPART_DATABASE_URL="host=/tmp/pg user=rampart dbname=rampart"
export RAMPART_LISTEN="127.0.0.1:8090"
export RAMPART_BASIC_AUTH_USER=admin
echo 'hunter2' | argon2 "$(openssl rand -hex 8)" -id -t 2 -m 16 -e > /tmp/hash
openssl rand -hex 20 > /tmp/api-key
export RAMPART_BASIC_AUTH_HASH_FILE=/tmp/hash
export RAMPART_API_KEY_FILE=/tmp/api-key
export RAMPART_PUBLIC_ORIGIN=http://localhost:8090
rampart migrate
rampart admin add-mailbox contact@example.com --display-name me
rampart serve &
curl -u admin:hunter2 http://localhost:8090/api/v1/user/info
```

## Deploy (NixOS)

`flake.nix` exports `nixosModules.default`. Consume it as a flake input
from your host config. Minimal wiring:

```nix
{
  inputs.rampart.url = "github:amaanq/rampart";
  # or path:/path/to/local/checkout for dev

  outputs = { self, nixpkgs, rampart, ... }: {
    nixosConfigurations.nunatak = nixpkgs.lib.nixosSystem {
      modules = [
        rampart.nixosModules.default
        ({ pkgs, ... }: {
          services.rampart = {
            enable = true;
            package = rampart.packages.${pkgs.system}.default;
            publicOrigin = "https://bunker.rampart.email";
            aliasDomains = [ "addy.example.com" ];
            basicAuth.hashFile = "/run/agenix/rampart-basic-auth";
            apiKeyFile = "/run/agenix/rampart-api-key";
            nginx.hostName = "bunker.rampart.email";
            # Pin to the tailscale interface IP to avoid public WAN exposure.
            nginx.listenAddresses = [ "100.64.0.X" ];
          };
        })
      ];
    };
  };
}
```

### DNS records (per alias domain)

The dashboard renders the exact records for each domain and verifies them
automatically. The two DKIM selectors and public keys come from Stalwart and
are unique to the domain. A configured domain has this shape:

```
addy.example.com.                         MX    10 mx.rampart.email.
addy.example.com.                         TXT   "v=spf1 mx ~all"
_dmarc.addy.example.com.                  TXT   "v=DMARC1; p=quarantine;"
<rsa-selector>._domainkey.addy.example.com.      TXT   "v=DKIM1; k=rsa; h=sha256; p=..."
<ed25519-selector>._domainkey.addy.example.com.  TXT   "v=DKIM1; k=ed25519; h=sha256; p=..."
```

### Secrets

Three files, referenced by module options:

- `basicAuth.hashFile` — argon2id PHC string (output of `argon2 <salt> -id -e`)
- `apiKeyFile` — any high-entropy string (e.g. `openssl rand -hex 32`)
- (postgres password — via agenix-rekey, mode 0400, owner = stalwart-mail so
  the stalwart `store.sql` block can read it)

## Phase 2 (not implemented)

Reply-via-alias. Schema is already forward-compatible: the
`reverse_contact` table exists and `email_log.reverse_contact_id` is
nullable. See DESIGN.md §8 for where the new Sieve branch and LMTP
resubmit worker drop in.

## Layout

```
Cargo.toml
flake.nix
nix/module.nix              reusable NixOS module
rust-toolchain.toml
migrations/V001__init.sql   schema, triggers, rampart_sieve_lookup view
src/
    main.rs                 clap subcommand dispatch, AppState
    config.rs               env/file-based config
    db.rs                   deadpool pool + typed row models
    migrate.rs              `rampart migrate` — refinery runner
    admin.rs                `rampart admin` subcommands
    sieve.rs                Sieve template rendering
    auth.rs                 basic + bearer + argon2 cache + Origin CSRF
    error.rs                ApiError → HTTP response
    serve.rs                axum wiring + sd-notify
    api.rs                  /api/v1/* JSON handlers
    web.rs                  /, /mailboxes, /domains — askama templates
templates/
    layout.html
    aliases.html
    mailboxes.html
    domains.html
    _alias_row.html
    rampart_rcpt.sieve.tmpl
static/
    app.css
tests/
    integration.rs          per-test ephemeral DB
    support.rs               harness (CREATE DATABASE + migrate)
docs/
    stalwart-integration.md  runbook
```

## License

AGPL-3.0-or-later.
