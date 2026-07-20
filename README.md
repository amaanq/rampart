# rampart

Rampart is a self-hosted email alias manager built around Stalwart. It creates
random or custom addresses on your own domains, forwards their mail to a
verified mailbox, and lets you reply without exposing that mailbox.

Rampart runs alongside Stalwart rather than replacing it. Rampart manages
aliases, users, and forwarding policy, while Stalwart handles SMTP, queues,
DKIM, and delivery. Both use the same PostgreSQL database, so Stalwart can
resolve recipients during the SMTP transaction without having to make an HTTP round
trip call.

## What it does

Rampart allows you to create aliases that stand in for your real address. Any email
that's sent to an alias lands in the normal inbox, and replies go back out with
the alias as the visible sender. If an alias starts collecting junk, you can block
the sender or just turn the alias off.

The dashboard gives an overview as to what each alias has forwarded, replied to,
blocked, and bounced, and it also checks the domain's MX, SPF, DMARC, and DKIM records
so it's clear delivery will actually work. Additionally, there's a Firefox/Chromium
extension for creating and filling aliases at signup forms without having to even
open the dashboard. It's also multi-user, so you can invite your friends and give
them their own mailboxes and limits.

## Mail flow

Inbound mail takes this path.

```text
sender
  -> Stalwart session.rcpt Sieve
  -> PostgreSQL alias lookup
  -> internal LMTP recipient
  -> rampart worker
  -> authenticated SMTP resubmit
  -> user's mailbox
```

The worker replaces the visible sender with a per-contact reply address before
resubmitting the message. When the user replies to that address, the same Sieve
and worker path runs in reverse. Rampart checks the authenticated sender,
restores the original contact as the recipient, and sends with the alias as the
visible sender.

The synthetic LMTP address carries the alias (or contact ID) between Stalwart's
RCPT stage and Rampart's full-message processing. This avoids trusting the
inbound envelope sender and gives the worker access to Stalwart's
`Authentication-Results` before it accepts a reply.

## Deployment

The documented deployment path is the included NixOS module. It provisions the
Rampart web service, LMTP worker, migrations, Sieve renderer, Stalwart registry
bootstrap, garbage collection, PostgreSQL backup timer, and optional nginx
virtual host.

You can add the flake as an input and import `rampart.nixosModules.default`. Afterwards,
enable `services.rampart` and set the package, public origin, alias domains, nginx
host, SMTP password file, Stalwart admin password file, and VERP key file.
The complete option definitions and defaults live in [`nix/module.nix`](nix/module.nix).

The module doesn't install or fully configure Stalwart. The bootstrap command
covers the registry objects Rampart owns, but the base Stalwart install and its
TOML side are on you. See [Stalwart integration](#stalwart-integration) below
before deploying, as the JMAP registry and TOML split is easy to get subtly
wrong.

After the services are up, open `publicOrigin`. The first request redirects to
`/setup`, where the first administrator account is created.

### Secrets

The minimal module configuration references three files.

- `smtp.passwordFile` is the password for the Stalwart submission account used
  when Rampart resubmits mail.
- `stalwart.adminPasswordFile` lets the idempotent bootstrap command reconcile
  Stalwart's JMAP registry objects.
- `stalwart.verpKeyFile` is at least 32 bytes of entropy used to authenticate
  bounce addresses. You can generate one with `openssl rand -base64 32`.

The built-in backup timer only writes local `pg_dump` archives. Copy them off
the host with restic, borg, or whatever you use for backups.

## Stalwart integration

Rampart expects a Stalwart installation where public SMTP, authenticated
submission, queues, and DKIM already work. On top of that,
`rampart admin bootstrap-stalwart` reconciles the pieces Rampart owns through
the JMAP admin API. That means the `sql` lookup store, the trusted
`rampart_rcpt` Sieve script and its RCPT stage branches, the alias domain
objects with their DKIM rules, the internal LMTP route, and the notifier
account used for submission. The `alias_domain` table is the source of truth,
and bootstrap only removes objects carrying Rampart's marker, so unrelated
domains and routing branches survive. It's safe to re-run after a failed
deploy or a manual Stalwart change.

Under the NixOS module, the bootstrap unit runs after Stalwart and before the
LMTP worker. A bootstrap failure blocks worker startup on purpose, since
accepting mail without a working route would only strand it in the queue.
Before the first deploy, make sure the JMAP admin endpoint is reachable, the
admin account can update registry objects, PostgreSQL accepts the local
`stalwart-mail` role through peer authentication, and the three secret files
above exist.

The domain setup page renders the exact DNS records for each alias domain.
Publish all of them before testing delivery, and don't copy DKIM selectors or
keys from another domain. Stalwart creates them per domain and may rotate
them later.

To smoke test a fresh install, create a random alias and send mail to it from
an external account. The verified mailbox should receive one copy with a
generated reply address. Reply from that mailbox and confirm the other side
sees the alias in `From`, then send to the reply address from a *different*
mailbox and confirm it's rejected. Finally, check the receiving provider's
authentication results for DKIM aligned with the alias domain and a DMARC
pass for the alias sender.

Creating or deleting a domain pushes the Sieve and registry changes to
Stalwart immediately, but the database mutation succeeds even when that sync
fails. Rampart logs a warning and the next bootstrap run reconciles the
difference. You can trigger that manually.

```bash
systemctl restart rampart-bootstrap-stalwart
```

The same restart is needed after rotating the SMTP password, since bootstrap
pushes the current notifier password on every run. Rotate the VERP key with
care, as messages already in flight carry signatures made with the old key
and their later bounce reports will be ignored.

When something breaks, start with the service logs.

```bash
journalctl -u rampart -u rampart-worker -u rampart-bootstrap-stalwart -u stalwart
```

- A stopped worker usually means bootstrap failed.
- Rejected mail to a new domain usually means domain or Sieve sync failed.
- A working forward path with broken replies usually means Stalwart's
  authentication results don't match `stalwart.authservId`.
- Failed outbound submission usually means the notifier password or SMTP port
  differs between Rampart and Stalwart.

After a Stalwart upgrade, run bootstrap against staging and repeat the smoke
test before deploying to the mail host. The registry object shapes are an
external API, and a version change can break reconciliation even when
Stalwart itself starts fine.

## Development

To begin hacking on rampart, enter the development shell.

```sh
nix develop
```

### Frontend

To start the mock UI:

```bash
cargo run -- preview
```

The `preview` command serves the complete dashboard with in-memory fixture data
on localhost port 8090. It doesn't need the database or Stalwart setup.

Database-backed tests create and drop a separate database per test. Point them
at a PostgreSQL database whose role has `CREATEDB`.

```bash
createdb rampart_test
export RAMPART_TEST_DB_URL="host=/tmp user=$USER dbname=rampart_test"
RAMPART_REQUIRE_DB_TESTS=1 cargo test
```

Without `RAMPART_REQUIRE_DB_TESTS=1`, database tests are skipped when PostgreSQL
isn't available. The reason being that this is useful for a quick unit-test pass.

SQL lives in `queries/` and is compiled into typed Rust bindings by Cornucopia.
After changing a query or migration, regenerate the checked-in crate.

```bash
cornucopia live "$RAMPART_TEST_DB_URL"
```

## Commands

The CLI has five top-level commands.

```text
rampart serve      run the HTTP server and dashboard
rampart preview    run the mock UI without external services
rampart worker     run the LMTP rewrite/resubmit worker
rampart migrate    apply pending PostgreSQL migrations
rampart admin ...  users, invites, mailboxes, import/export, GC, and bootstrap
```

Run `rampart --help` or `rampart admin --help` for the complete command list.

## Browser extension

[`extension/`](extension/) contains a Firefox and Chromium extension. The
toolbar popup only needs one-page `activeTab` access. Its optional inline field
helper requests access to HTTP(S) pages when enabled, rather than taking that
permission by default.

Build and load instructions are in
[`extension/README.md`](extension/README.md).

## License

This project is licensed under the AGPL-3.0-or-later license.
