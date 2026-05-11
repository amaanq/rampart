# Reply-via-alias architecture (phase 2)

Operator-facing deep dive on the LMTP-worker architecture rampart uses
for reply-via-alias. Captures what stalwart needs configured, how
messages flow, and why the design looks the way it does.

## Flow

```
external sender
      │ SMTP :25
      ▼
┌────────────────────────────────────────────────────────────────┐
│ Stalwart MTA                                                   │
│                                                                │
│  session.rcpt Sieve:                                           │
│    - alias exists → set envelope.to = rampart-<alias_id>@internal   │
│    - reply-addr matches → set envelope.to = rampart-reply-<rc_id>@internal
│    - else → reject 550                                         │
│                                                                │
│  session.rcpt.relay expression:                                │
│    allow when rcpt_domain == "internal.rampart.lmtp"                │
│                                                                │
│  session.data: stalwart writes Authentication-Results header   │
│                                                                │
│  Queue routing (MtaRoute registry object):                     │
│    rampart-lmtp: RoutingStrategy::Relay                             │
│             protocol=lmtp, address=127.0.0.1, port=8024        │
│  Outbound strategy selects rampart-lmtp when                        │
│    rcpt_domain == "internal.rampart.lmtp"                           │
└────────────────────────────────────────────────────────────────┘
      │ LMTP :8024
      ▼
┌────────────────────────────────────────────────────────────────┐
│ rampart-worker                                                      │
│  - accept one RCPT, parse rampart-<id>/rampart-reply-<id>                │
│  - reject anything else (loop prevention)                      │
│  - mail-parser MIME; read Authentication-Results (topmost      │
│    with authserv-id matching our stalwart hostname)            │
│  - strip only our stalwart's AR header (misleading downstream) │
│  - leave inbound DKIM-Signature + ARC intact (forensic)        │
│                                                                │
│  FORWARD branch (rampart-<id>):                                     │
│    upsert reverse_contact, addheader/deleteheader From →       │
│    "Name <r.<token>@<alias-domain>>"                           │
│    submit SMTP AUTH to localhost:465 as rampart-notifier@<domain>   │
│    MAIL FROM = alias, RCPT TO = mailbox.email                  │
│                                                                │
│  REPLY branch (rampart-reply-<id>):                                 │
│    verify AR dmarc=pass + visible From aligns with mailbox     │
│    (strict mode: visible From equals mailbox.email exactly)    │
│    rewrite From: to alias address                              │
│    submit: MAIL FROM = alias, RCPT TO = rc.real_email          │
│                                                                │
│  LMTP 5xx on permanent failure (stalwart generates DSN)        │
│  LMTP 2xx only after outbound accepted (commits to logging)    │
└────────────────────────────────────────────────────────────────┘
      │ SMTP AUTH :465
      ▼
┌────────────────────────────────────────────────────────────────┐
│ Stalwart outbound                                              │
│  - authenticated relay allowed                                 │
│  - DKIM-signs for alias domain (existing signature rule)       │
│  - delivers to Gmail/Outlook/etc.                              │
└────────────────────────────────────────────────────────────────┘
```

## Why LMTP and not a pure-Sieve approach

Spike-verified answers, short form:

1. **Sieve `editheader` is a capability, not a verb.** Real commands
   are `addheader` / `deleteheader`, and they only execute at DATA
   because RCPT has no message body.
   Source: `sieve-rs-0.7.0/src/compiler/grammar/actions/action_editheader.rs:62`.

2. **Cross-stage state doesn't exist.** sieve-rs rejects arbitrary
   envelope variable names (only fixed fields — To, From, Notify,
   Orcpt, Ret, Envid per `sieve-rs/src/lib.rs:155`). Sieve globals
   don't persist across stage invocations. So you can't stash the
   original alias at RCPT and read it at DATA.

3. **`envelope.from` at RCPT is forgeable.** Unauthenticated inbound
   SMTP lets the sender declare any MAIL FROM. Real sender
   authentication requires DMARC results, which only exist at DATA
   (stalwart writes Authentication-Results there, per
   `stalwart/crates/smtp/src/inbound/data.rs:226`).

An LMTP worker sidesteps all three: the alias ID travels in the
LMTP envelope's synthetic address (`rampart-<alias_id>@internal.rampart.lmtp`),
the worker sees the full message AND the AR header, and the worker
runs Rust code with no stage-model constraints.

## Stalwart config — where each piece lives

| what | where | how configured |
|---|---|---|
| Sieve script at session.rcpt | `services.stalwart.settings.sieve.trusted.scripts.rampart_rcpt` | TOML settings; file-referenced script body |
| session.rcpt.relay expression | `services.stalwart.settings.session.rcpt.relay` | TOML settings; expression |
| MtaRoute (LMTP relay route) | **JMAP registry object** (ObjectType::MtaRoute) | admin API, NOT TOML |
| MtaOutboundStrategy (picks route per rcpt domain) | **JMAP registry object** | admin API |
| MtaDeliverySchedule, MtaVirtualQueue | **JMAP registry objects** | admin API |
| rampart-notifier principal + password | internal directory | JMAP account create (Account::User) |
| auth.dkim.sign (alias domain) | `services.stalwart.settings.auth.dkim.sign` | TOML settings |
| server.virtual for alias domain | `services.stalwart.settings.server.virtual` | TOML settings |

### The TOML-vs-registry split (and why it matters)

Stalwart 0.15.5 moved outbound queue configuration (routes, delivery
schedules, virtual queues, connection strategies) into its
registry/JMAP object model, rather than flat TOML settings. This is
observable in `tests/src/smtp/outbound/lmtp.rs`, which uses
`registry_create_object(MtaRoute::Relay(...))` rather than a
settings block.

For us, this means **the NixOS module can't just emit TOML** for the
LMTP route. We need a bootstrap step that:

1. Waits for stalwart to come up and its HTTP listener to be ready
2. Authenticates as `admin` (via the `fallback-admin` credential
   already in our module)
3. Creates / upserts:
   - `MtaRoute::Relay { name: "rampart-lmtp", address: "127.0.0.1",
     port: 8024, protocol: Lmtp }`
   - `MtaOutboundStrategy { route expression routing
     rcpt_domain=='internal.rampart.lmtp' to 'rampart-lmtp' }`
   - `MtaVirtualQueue { name: "default", ... }` (if not already)
   - `MtaDeliverySchedule` for retry policy
   - The `rampart-notifier@<primary-domain>` principal with an
     age-sourced password
4. Is idempotent (skip creation if the named object already exists)

Implemented as a new `rampart admin bootstrap-stalwart` subcommand. Uses
`reqwest` to POST JMAP requests (JSON envelope at `/jmap`). The
module's systemd unit runs this after stalwart.service + before
enabling rampart-worker.service, and re-runs safely on every stalwart
reload.

## Spike results

Verified during architectural review against Stalwart 0.15.5
(commit shipped in nixpkgs#stalwart):

- **Queue routing via `RoutingStrategy::Relay` + `protocol=Lmtp`**:
  confirmed by stalwart's own test
  (`tests/src/smtp/outbound/lmtp.rs:93-102` — creates
  `MtaRoute::Relay` with `protocol: MtaProtocol::Lmtp`, stalwart
  delivers to a real LMTP listener on 9924 over loopback).
- **`session.rcpt.relay` as the gate for unknown domains**:
  confirmed by `crates/smtp/src/inbound/rcpt.rs:233` — when
  `RcptResolution::UnknownDomain` is returned, stalwart evaluates
  the `rcpt.relay` expression; `true` means accept. Allowing
  `rcpt_domain == "internal.rampart.lmtp"` does exactly what we need.
- **Authentication-Results preservation in LMTP delivery**:
  confirmed by `crates/smtp/src/inbound/data.rs:226` (AR header
  built during DATA), line 389 (written into the headers buffer),
  and `crates/smtp/src/queue/spool.rs:350` (headers + raw_message
  queued as the blob that LMTP delivery then reads).
- **Outbound SMTP AUTH relay to external**: confirmed by
  `crates/smtp/src/scripts/event_loop.rs:269-276` (queue add is
  unconditional) and the existing mail/default.nix behavior on
  nunatak (authenticated submission via submissions port).

**Spike outcome: LMTP worker architecture is sound. Deployment
complication: registry bootstrap via JMAP, handled by
`rampart admin bootstrap-stalwart`.**

A throwaway local stalwart was also started (pid at
`/tmp/rampart-spike/stalwart.pid`, now stopped) to confirm its HTTP
admin serves the JMAP session endpoint at `/.well-known/jmap` —
the transport we'll use for bootstrap. Instance teardown: `rm -rf
/tmp/rampart-spike`.
