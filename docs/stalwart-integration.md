# Stalwart integration

`rampart` does not ship a stalwart server. It integrates with an existing
stalwart install by (a) rendering a Sieve script into a known file
path, and (b) requiring a few additions to stalwart's own config.
This doc is the operator-facing reference.

## Architecture recap

- At SMTP `RCPT TO:` time, stalwart runs a trusted Sieve script for
  the session. The script queries our `rampart_sieve_lookup` view, and
  either `set`s `envelope.to` to the forward target (causing stalwart
  to route as if the message were addressed there originally) or
  `reject`s with `550 5.1.1`.
- The `set "envelope.to" "..."` action emits a `SetEnvelope`
  modification that stalwart honors at RCPT
  (`stalwart/crates/smtp/src/inbound/rcpt.rs:127-133`). No catch-all,
  no `redirect`.
- The forward target is either an internal stalwart principal (common
  — `contact@example.com` on the same box) or an external address
  (requires `session.rcpt.relay=true`; phase 2 concern).

## Stalwart config snippet

Add the following to `stalwart.toml` (or set equivalent key/values in
a NixOS `services.stalwart.settings` block). Paths assume rampart runs on
the same host with the default `RAMPART_SIEVE_OUTPUT_PATH`.

```toml
# 1. Declare the alias domain(s) as locally-accepted virtual domains.
#    This is the same mechanism used for example.com, ameerq.com, etc.
[[server.virtual]]
domain = "addy.example.com"

# 2. Register a trusted system-level Sieve script, read from disk.
#    rampart writes this file on every domain add/remove, and again via
#    `rampart admin render-sieve` at startup.
[sieve.trusted.scripts.rampart_rcpt]
contents = "%{file:/var/lib/stalwart-mail/scripts/rampart_rcpt.sieve}%"

# 3. Wire the script into the RCPT stage. Every RCPT for a managed
#    alias domain runs our script; other recipients fall through
#    untouched (the script does an `if anyof (envelope :domain :is
#    "to" "...")` guard internally).
[session.rcpt]
script = "'rampart_rcpt'"

# 4. Register the rampart database as a SQL store under the name `sql`,
#    matching the `query('sql', ...)` calls in the rendered Sieve.
[store.sql]
type = "postgresql"
host = "localhost"
port = 5432
database = "rampart"
user = "rampart"
password = "%{file:/run/credentials/stalwart.service/rampart_db_password}%"
timeout = "5s"
pool.max-connections = 4

# 5. DKIM-sign mail leaving the alias domain. Reuses the existing
#    stalwart selector and key.
[signature."rsa-addy-example-com"]
domain = "addy.example.com"
private-key = "%{file:/run/credentials/stalwart.service/dkim_key}%"
selector = "stalwart"
algorithm = "rsa-sha256"
canonicalization = "relaxed/relaxed"

[[auth.dkim.sign]]
if = "sender_domain == 'addy.example.com'"
then = "['rsa-addy-example-com']"
```

## DNS (prerequisite)

Before stalwart will accept mail for `addy.example.com`:

- `addy.example.com. MX 10 mail.example.com.`
- `addy.example.com. TXT "v=spf1 mx -all"`
- `_dmarc.addy.example.com. TXT "v=DMARC1; p=reject; adkim=s; aspf=s; rua=mailto:dmarc@example.com"`
- `stalwart._domainkey.addy.example.com. CNAME stalwart._domainkey.example.com.`

## Smoke-test procedure (on nunatak after deploy)

```bash
# 1. Render the Sieve file and copy into place
rampart admin render-sieve --output /var/lib/stalwart-mail/scripts/rampart_rcpt.sieve
systemctl restart stalwart  # trusted scripts are not hot-reloaded

# 2. Seed one alias
curl -H "Authorization: Bearer $RAMPART_API_KEY" -H 'Content-Type: application/json' \
     -H "Origin: https://bunker.rampart.email" \
     -d '{"domain":"addy.example.com","note":"smoke test"}' \
     https://bunker.rampart.email/api/v1/alias/random
# note the returned address, e.g. aaaa1111bb@addy.example.com

# 3. Send a test message from an external host
swaks --to aaaa1111bb@addy.example.com \
      --from test@example.com \
      --server mail.example.com \
      --data "Subject: rampart smoke test\n\nhello"

# 4. Verify:
#    - the mail arrives at contact@example.com (or whichever mailbox
#      was the default)
#    - exactly one copy is delivered (no duplicate to a sink)
#    - rampart's `nb_forward` counter for the alias incremented
#    - stalwart logs show a SetEnvelope-driven rewrite

# 5. Disable the alias, re-swaks, verify 550
curl -XPUT -H "Authorization: Bearer $RAMPART_API_KEY" \
     -H "Origin: https://bunker.rampart.email" \
     https://bunker.rampart.email/api/v1/aliases/$ID/toggle
swaks ... # expect: "550 5.1.1 Unknown or disabled alias"
```

If the forward target is external (e.g. `someone@gmail.com` rather
than an internal `contact@example.com`), stalwart will reject the RCPT
with `550 5.1.2 Relay not allowed` unless `session.rcpt.relay`
evaluates to true for this message — see the bootstrap section below
which sets that on `MtaStageRcpt.allowRelaying`.

## Phase 2: LMTP-worker bootstrap

Phase 2 added reply-via-alias via an LMTP resubmit worker. Stalwart
0.16.x stores outbound queue config (routes, virtual queues, delivery
schedules, connection strategies, the outbound strategy) AND the
session.rcpt allow-relay expression in JMAP registry objects, not in
TOML — so a fresh stalwart deployment cannot serve rampart until the
following objects exist.

`rampart admin bootstrap-stalwart` creates them idempotently. Run after
stalwart starts and before rampart-worker:

```bash
rampart admin bootstrap-stalwart \
    --jmap-base-url http://127.0.0.1:8080 \
    --admin-username admin \
    --admin-password-file /run/credentials/rampart-bootstrap-stalwart.service/stalwart_admin_password \
    --rampart-notifier-password-file /run/credentials/rampart-bootstrap-stalwart.service/smtp_password \
    --rampart-notifier-address rampart-notifier@example.com \
    --lmtp-address 127.0.0.1 \
    --lmtp-port 8024
```

First run prints `created=N patched=M skipped=0`. Second run prints
`created=0 patched=0 skipped=N+M` — fully idempotent.

Add `--dry-run` to inspect the JSON that would be sent without
touching stalwart.

### Objects created / patched

| Object | Kind | Action | Why |
|---|---|---|---|
| `MtaStageRcpt` | singleton | GET → PATCH | OR-concat `rcpt_domain == 'internal.rampart.lmtp'` into `allowRelaying` so stalwart accepts our synthetic RCPTs. |
| `MtaRoute` `rampart-lmtp` | named | create | Relay/Lmtp/127.0.0.1:8024 — the worker's listener. |
| `MtaVirtualQueue` `ramplmtp` | named | create | 8 chars (stalwart caps at 8); dedicated to rampart traffic. |
| `MtaDeliverySchedule` `rampart_lmtp` | named | create | Loopback retry: 60s × 3 then bounce — failures are ours, not remote. |
| `MtaConnectionStrategy` `rampart_lmtp` | named | create | Short timeouts; no TLS requirement (Branch A — explicit own connection so OutboundStrategy.connection has a known target). |
| `MtaOutboundStrategy` | singleton | GET → PATCH | Patches **route**, **schedule**, **connection** expressions to dispatch `internal.rampart.lmtp` recipients to the names above. Existing `else` branches preserved. |
| Account `rampart-notifier@<domain>` | account | create | Submission identity. Same password rampart uses for SMTP AUTH. |

### What if the spike reveals stalwart wants something different?

The schema is pinned to stalwart 0.15.5 (`registry::schema::structs`).
If a stalwart bump rejects any of these JSON shapes, run
`rampart admin bootstrap-stalwart --dry-run`, compare against the actual
schema (in `~/projects/forks/stalwart/crates/registry/src/schema/structs_impl.rs`),
and update `rampart/src/bootstrap.rs` to match. Each upsert is one ~30 LOC
function with a `json!({...})` literal — easy to adjust.

If the JMAP shape itself is wrong (HTTP 400 / 500 from `/jmap`),
bootstrap fails fast with the response body in the error — no half
state.

Fallback if JMAP-via-HTTP doesn't work for your stalwart: shell out to
`stalwart-cli` via `tokio::process::Command` and translate the
`registry-create-object` arguments. Not currently implemented; would
be a ~150 LOC pivot.

## Live deploy smoke test (post-bootstrap)

After `bootstrap-stalwart` succeeds and `rampart-worker.service` is up:

1. Add a verified mailbox + create an alias as a regular user.
2. **Forward path:** send mail from an external account to the alias.
   Verify the mailbox receives it with rewritten From
   (`"<original sender>" <r.<token>@<alias-domain>>`).
3. **Reply path:** reply from the mailbox's real address to
   `r.<token>@<alias-domain>`. Verify the original sender receives it
   with `From: <alias>@<alias-domain>` and DKIM-valid for the alias
   domain.
4. **Spoof check:** send to `r.<token>@<alias-domain>` from an account
   without DMARC alignment (e.g. `friend@example.org` against a
   gmail-tenant mailbox). Stalwart's DSN should carry our
   `550 5.7.1 reply-policy: ...` reason.
5. **Cross-tenant reply rejection:** reply from `bob@gmail.com` to a
   reverse alias whose mailbox is `alice@gmail.com`. Expect
   `550 5.7.1` — exact-mailbox match is the default policy (Codex
   round-15/16). The `user.strict_reply` column is currently a no-op
   placeholder for a future authorized-sender list; do not toggle it
   to change behavior, it has none.
6. **Catch-all:** `UPDATE alias_domain SET catch_all = TRUE,
   default_mailbox_id = <verified mbox id> WHERE id = ...`. Send to a
   fresh local-part on that domain. Verify an `auto_created=TRUE`
   alias appears and the mailbox receives the mail.
7. **Catch-all cap:** `UPDATE alias_domain SET max_auto_created = 3
   WHERE id = ...`. Send to 4 distinct fresh local-parts. The 4th must
   be rejected with `550 5.1.1 Unknown or disabled alias`.
8. **Loop guard:** as a safety check, send LMTP traffic directly to
   port 8024 with RCPT `<foo@internal.rampart.lmtp>` (no `rampart-` / `rampart-reply-`
   prefix). Worker must respond `550 5.1.1 Not a routable rampart worker
   recipient` and never enter the pipeline.

If any step fails, check `journalctl -u rampart-worker -u stalwart` for
the actual reason. The worker's logs include `error = ?` with the
full pipeline error chain.

## Browser passkey (Yubikey) smoke

Manual hardware test, runs once per deploy. Uses the dashboard at
`https://rampart.<domain>/settings`.

1. Log in via password.
2. Click "register passkey" — touch the Yubikey, name it (e.g. "yk5c").
3. Logout.
4. Click "sign in with passkey" on the login page; type your email,
   touch Yubikey, expect to land at `/`.
5. Logout. Sign in via password again — confirm both paths still work.
6. Register a *second* passkey (e.g. a phone-bound one).
7. From `/settings`, delete the first passkey.
8. Logout, sign in via the second passkey. Confirm success.

If any step fails, check the browser console for webauthn errors and
the server logs for `passkey auth finish failed` (uniform error
message — root cause is in the tracing logs).

## DNS / mail-auth alignment check

After Step 6 of the live SMTP smoke (an external sender → alias →
mailbox forward arrived), pull up the mailbox's copy of the message
and inspect its full headers. The receiver-authored
`Authentication-Results:` header (the *top-most* one whose
`authserv-id` is the receiver's hostname — Gmail, Outlook, Fastmail,
etc., **NOT** ours) must satisfy these semantic checks:

- **DMARC pass** for `header.from=<alias-domain>`. The receiver
  computed DMARC against the visible `From:` we wrote (the reverse-
  contact reply-address on the alias domain) and accepted it.
- **DKIM pass aligned to the alias-domain** — at least one DKIM
  signature listed in the AR header must have `header.d=` matching
  the alias-domain in the visible `From:`. This proves stalwart's
  outbound signing rule covered the alias domain correctly.

Exact spacing/syntax varies between receivers (Gmail uses different
phrasing than Outlook); look for the semantic conjunction, not a
literal string match.

DNS prerequisites (one-time, separate from the message smoke):

```bash
dig MX <alias-domain>                # → nunatak's mail hostname
dig TXT <alias-domain>               # → "v=spf1 mx -all" (or equivalent)
dig TXT _dmarc.<alias-domain>        # → "v=DMARC1; p=reject; ..."
dig TXT stalwart._domainkey.<alias-domain>   # → DKIM public key
```

If any of these are absent / wrong, the DMARC pass at the receiver
won't happen — fix DNS before debugging anything else.

## Secret rotation (rampart-notifier SMTP password)

`bootstrap-stalwart` reconciles the `rampart-notifier` Account's
`credentials` with whatever's in `RAMPART_SMTP_PASSWORD_FILE` on every
run (round-5 hardening). Rotation is therefore a deploy + bootstrap
re-run, not a manual stalwart admin dance:

1. `openssl rand -base64 32 | head -c 32 > /tmp/new-secret`.
2. Update agenix entry: `agenix -e secrets/rampart-smtp-password.age`,
   paste the new value, save.
3. `nixos-rebuild switch` — systemd reloads
   `LoadCredential=smtp_password` for `rampart.service`, `rampart-worker.service`,
   and `rampart-bootstrap-stalwart.service`.
4. `systemctl start rampart-bootstrap-stalwart.service` — the oneshot is
   idempotent and re-pushes the credential. Confirm logs show
   `patched=1` for the Account.
5. `systemctl restart rampart.service rampart-worker.service` so they pick up
   the new password file.
6. Smoke: send a test forward through stalwart → alias → mailbox.
   Verify the `rampart-worker` log shows the SMTP AUTH submit succeeded.

If step 4 reports `created=0 patched=0 skipped=N` (i.e. credentials
not patched), the bootstrap thinks the existing password matches —
likely cause: the agenix entry still has the old value. Verify the
file contents at the LoadCredential mount before re-rerunning.

## Backup / restore drill

`rampart-backup.service` runs daily and writes `rampart-<UTC ts>.sql.gz` to
`/var/lib/rampart/backups` (configurable via `services.rampart.backups.*` in
the NixOS module). Atomic write: `pg_dump` streams to `$tmp`, then
`mv $tmp $out` only on success — a partial dump never masquerades
as a retained backup.

**Loud caveat**: local dumps are NOT disaster recovery. They protect
against accidental SQL (DROP, runaway DELETE, schema corruption) and
ransomware-via-app, but **not against host loss**. Pair with an
off-host pickup — borg, restic, scp via cron, restic-rest, your
existing backup pipeline — before treating this as backup-complete.
The local backup directory is short-term staging.

Verify a backup is restorable with `scripts/restore-drill.sh`:

```bash
sudo -u rampart scripts/restore-drill.sh /var/lib/rampart/backups/rampart-20260425T120000Z.sql.gz
```

The script creates a temp DB, restores the dump, asserts the exact
expected migration version (currently V001/init), prints row counts
for the headline tables, then drops the temp DB. Any failure exits
non-zero. Run it on every backup-pipeline change and at minimum
monthly on a representative dump.

## Operator gotchas

A few code-level invariants that aren't obvious from the dashboard:

- **Domain CRUD via the API regenerates the Sieve file but doesn't
  restart stalwart.** The trusted Sieve script is loaded once at
  stalwart startup; it is NOT hot-reloaded when rampart rewrites the file.
  After every `POST /api/v1/domain` or `DELETE /api/v1/domain/:id`,
  run `systemctl start rampart-render-sieve.service` (the unit ships a
  `try-restart stalwart` ExecStartPost) — without this, mail to the
  newly-added domain is rejected at RCPT until next stalwart restart,
  and mail to a freshly-deleted domain is still accepted.

- **`rampart admin reset-password` runs in a separate process from
  `rampart serve`** and therefore cannot invalidate the in-process
  `VerifyCache` (LRU keyed on `(user_id, sha256(password))`, 60-second
  TTL). After resetting, the OLD password remains valid in the running
  server for up to 60 s on the Basic-auth path. Standard rotation
  procedure: `systemctl restart rampart.service` after `reset-password`.
  Cookie sessions are deleted by reset, so this only affects API/CLI
  callers using Basic auth in that 60 s window.

- **CSV export/import does not handle commas in `note` fields.**
  Export quotes them correctly; import does a naive `split(',')` and
  will corrupt aliases whose note contains a comma. Avoid commas in
  notes, or use the API directly for imports with arbitrary text.
  (Future: pull in the `csv` crate.)

- **Secure cookie attribute** is auto-toggled based on `RAMPART_PUBLIC_ORIGIN`.
  HTTPS origin → `Secure` set (browsers won't send the cookie over
  plain HTTP, which is what we want). HTTP origin (e.g. tailscale-only
  deploys without TLS) → `Secure` is omitted so logins work. If a
  deploy unexpectedly fails to keep users logged in, double-check the
  scheme of `RAMPART_PUBLIC_ORIGIN` matches the actual nginx vhost.

## Stalwart upgrade guard

`rampart admin bootstrap-stalwart` writes JMAP registry objects whose
schemas are pinned to stalwart 0.16.x. Authoritative file references
in the local stalwart checkout used by this codebase (round-13 caught
five wire-shape bugs against the live 0.16.1 deploy):
`crates/registry/src/schema/structs.rs` (object schemas — MtaRoute,
MtaVirtualQueue, MtaDeliverySchedule, MtaConnectionStrategy,
MtaOutboundStrategy, MtaStageRcpt, UserAccount, Domain),
`crates/registry/src/schema/structs_impl.rs` (Default impls + field
validators), `crates/registry/src/types/list.rs` (the indexed-object
`{"0":...,"1":...}` `List<T>` wire shape that bit us), and
`crates/registry/src/lib.rs` for method dispatch + `Action::ReloadSettings`.
Bumping stalwart could silently change object field names or add
required fields; the oneshot would fail at runtime, blocking
rampart-worker startup (correct behavior — the worker would have nothing
to talk to).

Procedure on every stalwart bump:

1. Read the diff in `crates/registry/src/schema/structs_impl.rs`
   (or `structs.rs`) for these object types: `MtaRoute`,
   `MtaVirtualQueue`, `MtaDeliverySchedule`,
   `MtaConnectionStrategy`, `MtaOutboundStrategy`, `MtaStageRcpt`,
   `UserAccount`, `Domain`. Look for:
   - new required fields (we'll need to populate them)
   - renamed `#[serde(rename = "...")]` JSON keys
   - `assert_read_only` additions (a previously-mutable field
     becoming read-only on update would break the reconcile path)
2. Run `rampart admin bootstrap-stalwart --dry-run` against a staging
   stalwart of the new version. Inspect the JSON in the output.
3. Run the live SMTP smoke (steps 1-11 above) against staging.
4. Only THEN bump the stalwart input pin in `flake.nix` /
   `nix/module.nix`.

The doc-only-pin path (no nix-level assertion) is fine for now;
upgrade discipline lives here.
