//! MTA singleton patches and bare-object upserts. Domain-marker logic
//! lives in `domains.rs`.

use anyhow::{Context, Result};
use serde_json::{Value, json};

use super::jmap::{JmapClient, build_expression, read_else, read_match_branches};
use super::{CONN_NAME, RELAY_GUARD, ROUTE_NAME, SCHEDULE_NAME, SINGLETON_ID, Stats, VQ_NAME};

pub(super) async fn patch_stage_rcpt(
    client: &JmapClient,
    stats: &mut Stats,
    dry_run: bool,
) -> Result<()> {
    // Writing just `{ "else": ... }` clobbers operator-defined match
    // branches; read-modify-write to preserve them.
    let cur = client.get_by_id("MtaStageRcpt", SINGLETON_ID).await?;
    let cur_match = read_match_branches(cur.as_ref().and_then(|o| o.get("allowRelaying")));
    let cur_else = read_else(cur.as_ref().and_then(|o| o.get("allowRelaying")), "false");

    let new_match = rebuild_with_canonical_guard(cur_match.clone(), "true");

    let unchanged = new_match == cur_match;
    if unchanged && cur.is_some() {
        stats.skipped += 1;
        tracing::info!("MtaStageRcpt: relay guard already canonical, skipping");
        return Ok(());
    }

    let patch = json!({
        "allowRelaying": build_expression(new_match, &cur_else),
    });
    if dry_run {
        println!("DRY RUN: MtaStageRcpt/set update singleton: {patch:#}");
        stats.skipped += 1;
        return Ok(());
    }
    if cur.is_some() {
        client
            .set_update("MtaStageRcpt", SINGLETON_ID, patch)
            .await?;
        stats.patched += 1;
    } else {
        // maxRecipients matches stalwart's seed default.
        let item = json!({
            "allowRelaying": patch.get("allowRelaying").unwrap(),
            "maxRecipients": { "else": "100" },
        });
        client.set_create("MtaStageRcpt", item).await?;
        stats.created += 1;
    }
    Ok(())
}

pub(super) async fn upsert_route(
    client: &JmapClient,
    stats: &mut Stats,
    address: &str,
    port: u16,
    dry_run: bool,
) -> Result<()> {
    let create = json!({
        "@type": "Relay",
        "name": ROUTE_NAME,
        "address": address,
        "port": port,
        "protocol": "lmtp",
        "implicitTls": false,
        "allowInvalidCerts": true,
    });

    if let Some(id) = client.query_by_name("MtaRoute", ROUTE_NAME).await?
        && let Some(cur) = client.get_by_id("MtaRoute", &id).await?
    {
        let cur_type = cur.get("@type").and_then(|v| v.as_str()).unwrap_or("");
        if cur_type != "Relay" {
            anyhow::bail!(
                "MtaRoute '{ROUTE_NAME}' (id {id}) has @type='{cur_type}', expected 'Relay'. \
                 Delete the route in stalwart and re-run bootstrap to recreate as Relay."
            );
        }
        let same = cur.get("address").and_then(|v| v.as_str()) == Some(address)
            && cur.get("port").and_then(|v| v.as_u64()) == Some(port as u64)
            && cur.get("protocol").and_then(|v| v.as_str()) == Some("lmtp")
            && cur.get("implicitTls").and_then(|v| v.as_bool()) == Some(false)
            && cur.get("allowInvalidCerts").and_then(|v| v.as_bool()) == Some(true);
        if same {
            stats.skipped += 1;
            return Ok(());
        }
        let patch = json!({
            "address": address,
            "port": port,
            "protocol": "lmtp",
            "implicitTls": false,
            "allowInvalidCerts": true,
        });
        if dry_run {
            println!("DRY RUN: MtaRoute/set update {id}: {patch:#}");
            stats.patched += 1;
            return Ok(());
        }
        client.set_update("MtaRoute", &id, patch).await?;
        stats.patched += 1;
        return Ok(());
    }

    if dry_run {
        println!("DRY RUN: MtaRoute/set create: {create:#}");
        return Ok(());
    }
    client.set_create("MtaRoute", create).await?;
    stats.created += 1;
    Ok(())
}

pub(super) async fn upsert_virtual_queue(
    client: &JmapClient,
    stats: &mut Stats,
    dry_run: bool,
) -> Result<String> {
    const THREADS: u64 = 4;
    let create = json!({ "name": VQ_NAME, "threadsPerNode": THREADS });

    if let Some(id) = client.query_by_name("MtaVirtualQueue", VQ_NAME).await? {
        if let Some(cur) = client.get_by_id("MtaVirtualQueue", &id).await? {
            let same = cur.get("threadsPerNode").and_then(|v| v.as_u64()) == Some(THREADS);
            if same {
                stats.skipped += 1;
                return Ok(id);
            }
            let patch = json!({ "threadsPerNode": THREADS });
            if dry_run {
                println!("DRY RUN: MtaVirtualQueue/set update {id}: {patch:#}");
                stats.patched += 1;
                return Ok(id);
            }
            client.set_update("MtaVirtualQueue", &id, patch).await?;
            stats.patched += 1;
            return Ok(id);
        }
        return Ok(id);
    }

    if dry_run {
        println!("DRY RUN: MtaVirtualQueue/set create: {create:#}");
        return Ok("dryrun-vq-id".into());
    }
    let id = client.set_create("MtaVirtualQueue", create).await?;
    stats.created += 1;
    Ok(id)
}

pub(super) async fn upsert_delivery_schedule(
    client: &JmapClient,
    stats: &mut Stats,
    queue_id: &str,
    dry_run: bool,
) -> Result<()> {
    // 60s × 3 then bounce (4 min total). Duration is millis-as-u64;
    // `intervals` is a stalwart `List<T>` (object keyed by stringified
    // positional indexes, NOT a JSON array).
    let retry = json!({
        "@type": "Custom",
        "intervals": {
            "0": { "duration": 60_000_u64 },
            "1": { "duration": 60_000_u64 },
            "2": { "duration": 60_000_u64 },
        },
    });
    let notify = json!({ "@type": "Default" });
    let expiry = json!({ "@type": "Ttl", "expire": 240_000_u64 });

    let create = json!({
        "name": SCHEDULE_NAME,
        "queueId": queue_id,
        "retry": retry,
        "notify": notify,
        "expiry": expiry,
    });

    if let Some(id) = client
        .query_by_name("MtaDeliverySchedule", SCHEDULE_NAME)
        .await?
        && let Some(cur) = client.get_by_id("MtaDeliverySchedule", &id).await?
    {
        let same = cur.get("queueId").and_then(|v| v.as_str()) == Some(queue_id)
            && cur.get("retry") == Some(&retry)
            && cur.get("notify") == Some(&notify)
            && cur.get("expiry") == Some(&expiry);
        if same {
            stats.skipped += 1;
            return Ok(());
        }
        let patch = json!({
            "queueId": queue_id,
            "retry": retry,
            "notify": notify,
            "expiry": expiry,
        });
        if dry_run {
            println!("DRY RUN: MtaDeliverySchedule/set update {id}: {patch:#}");
            stats.patched += 1;
            return Ok(());
        }
        client.set_update("MtaDeliverySchedule", &id, patch).await?;
        stats.patched += 1;
        return Ok(());
    }

    if dry_run {
        println!("DRY RUN: MtaDeliverySchedule/set create: {create:#}");
        return Ok(());
    }
    client.set_create("MtaDeliverySchedule", create).await?;
    stats.created += 1;
    Ok(())
}

pub(super) async fn upsert_connection_strategy(
    client: &JmapClient,
    stats: &mut Stats,
    dry_run: bool,
) -> Result<()> {
    // Duration (u64 millis), NOT Expression.
    const TIMEOUTS: [(&str, u64); 6] = [
        ("connectTimeout", 5_000),
        ("dataTimeout", 30_000),
        ("ehloTimeout", 5_000),
        ("greetingTimeout", 5_000),
        ("mailFromTimeout", 5_000),
        ("rcptToTimeout", 5_000),
    ];
    let mut create = serde_json::Map::new();
    create.insert("name".into(), json!(CONN_NAME));
    for (k, v) in TIMEOUTS {
        create.insert(k.into(), json!(v));
    }
    let create = Value::Object(create);

    if let Some(id) = client
        .query_by_name("MtaConnectionStrategy", CONN_NAME)
        .await?
        && let Some(cur) = client.get_by_id("MtaConnectionStrategy", &id).await?
    {
        let same = TIMEOUTS
            .iter()
            .all(|(k, v)| cur.get(*k).and_then(|x| x.as_u64()) == Some(*v));
        if same {
            stats.skipped += 1;
            return Ok(());
        }
        let mut patch = serde_json::Map::new();
        for (k, v) in TIMEOUTS {
            patch.insert(k.into(), json!(v));
        }
        let patch = Value::Object(patch);
        if dry_run {
            println!("DRY RUN: MtaConnectionStrategy/set update {id}: {patch:#}");
            stats.patched += 1;
            return Ok(());
        }
        client
            .set_update("MtaConnectionStrategy", &id, patch)
            .await?;
        stats.patched += 1;
        return Ok(());
    }

    if dry_run {
        println!("DRY RUN: MtaConnectionStrategy/set create: {create:#}");
        return Ok(());
    }
    client.set_create("MtaConnectionStrategy", create).await?;
    stats.created += 1;
    Ok(())
}

/// Force exactly one canonical `RELAY_GUARD` branch at the head, preserving
/// every non-guard branch in order. Stale then-values from prior runs get
/// overwritten.
pub(super) fn rebuild_with_canonical_guard(
    existing: Vec<(String, String)>,
    expected_then: &str,
) -> Vec<(String, String)> {
    let mut out = vec![(RELAY_GUARD.to_owned(), expected_then.to_owned())];
    out.extend(
        existing
            .into_iter()
            .filter(|(if_, _)| !if_.contains(RELAY_GUARD)),
    );
    out
}

pub(super) async fn patch_outbound_strategy(
    client: &JmapClient,
    stats: &mut Stats,
    dry_run: bool,
) -> Result<()> {
    let cur = client
        .get_by_id("MtaOutboundStrategy", SINGLETON_ID)
        .await?;

    // Preserve stalwart's default is_local_domain route and dsn/report schedule.
    let route_match = read_match_branches(cur.as_ref().and_then(|o| o.get("route")));
    let route_else = read_else(cur.as_ref().and_then(|o| o.get("route")), "'mx'");
    let schedule_match = read_match_branches(cur.as_ref().and_then(|o| o.get("schedule")));
    let schedule_else = read_else(cur.as_ref().and_then(|o| o.get("schedule")), "'remote'");
    let connection_match = read_match_branches(cur.as_ref().and_then(|o| o.get("connection")));
    let connection_else = read_else(cur.as_ref().and_then(|o| o.get("connection")), "'default'");

    let want_route_then = format!("'{ROUTE_NAME}'");
    let want_sched_then = format!("'{SCHEDULE_NAME}'");
    let want_conn_then = format!("'{CONN_NAME}'");
    let new_route = rebuild_with_canonical_guard(route_match.clone(), &want_route_then);
    let new_schedule = rebuild_with_canonical_guard(schedule_match.clone(), &want_sched_then);
    let new_connection = rebuild_with_canonical_guard(connection_match.clone(), &want_conn_then);

    let unchanged = new_route == route_match
        && new_schedule == schedule_match
        && new_connection == connection_match;
    if unchanged && cur.is_some() {
        stats.skipped += 1;
        tracing::info!(
            "MtaOutboundStrategy: canonical rampart branches already in place, skipping"
        );
        return Ok(());
    }

    let patch = json!({
        "route":      build_expression(new_route, &route_else),
        "schedule":   build_expression(new_schedule, &schedule_else),
        "connection": build_expression(new_connection, &connection_else),
    });

    if dry_run {
        println!("DRY RUN: MtaOutboundStrategy/set update singleton: {patch:#}");
        stats.skipped += 1;
        return Ok(());
    }
    if cur.is_some() {
        client
            .set_update("MtaOutboundStrategy", SINGLETON_ID, patch)
            .await?;
        stats.patched += 1;
    } else {
        // else-fallbacks above match stalwart's documented defaults.
        let item = json!({
            "route":      patch.get("route").unwrap(),
            "schedule":   patch.get("schedule").unwrap(),
            "connection": patch.get("connection").unwrap(),
        });
        client.set_create("MtaOutboundStrategy", item).await?;
        stats.created += 1;
    }
    Ok(())
}

pub(super) async fn upsert_notifier(
    client: &JmapClient,
    stats: &mut Stats,
    address: &str,
    password: &str,
    dry_run: bool,
) -> Result<()> {
    let (local, domain_name) = address
        .split_once('@')
        .ok_or_else(|| anyhow::anyhow!("rampart_notifier_address must be local@domain"))?;
    let local = local.to_owned();
    let domain_name = domain_name.to_owned();

    let domain_id = ensure_domain_id(client, stats, &domain_name, dry_run).await?;

    // Account uniqueness is (name, domainId); disambiguate every name=local
    // hit by domainId. List<Credential> uses the same indexed-object shape as `match`.
    let credentials = json!({ "0": { "@type": "Password", "secret": password } });
    let candidates = client.query_ids_by_name("Account", &local).await?;
    for candidate_id in &candidates {
        if let Some(obj) = client.get_by_id("Account", candidate_id).await?
            && obj.get("domainId").and_then(|v| v.as_str()) == Some(domain_id.as_str())
        {
            // Re-push so rotated agenix secrets reach stalwart. Wholesale
            // replacement is intended — rampart-notifier has no AppPassword/ApiKey.
            let patch = json!({ "credentials": credentials });
            if dry_run {
                println!(
                    "DRY RUN: Account/set update {candidate_id} (password redacted): {{\"credentials\":[{{...}}]}}"
                );
                stats.patched += 1;
                return Ok(());
            }
            client.set_update("Account", candidate_id, patch).await?;
            stats.patched += 1;
            return Ok(());
        }
    }

    let item = json!({
        "@type": "User",
        "name": local,
        "domainId": domain_id,
        "credentials": credentials,
        "description": "rampart outbound submission account",
    });
    if dry_run {
        println!(
            "DRY RUN: Account/set create (password redacted): {{\"name\":\"{local}\",\"domainId\":\"{domain_id}\"}}"
        );
        return Ok(());
    }
    client.set_create("Account", item).await?;
    stats.created += 1;
    Ok(())
}

/// Bare Domain by name. Alias domains go through
/// `domains::upsert_managed_alias_domain` which applies the marker.
pub(super) async fn ensure_domain_id(
    client: &JmapClient,
    stats: &mut Stats,
    name: &str,
    dry_run: bool,
) -> Result<String> {
    if let Some(id) = client.query_by_name("Domain", name).await? {
        return Ok(id);
    }
    let item = json!({
        "name": name,
        "isEnabled": true,
    });
    if dry_run {
        println!("DRY RUN: Domain/set create: {item:#}");
        return Ok("dryrun-domain-id".into());
    }
    let id = client.set_create("Domain", item).await?;
    stats.created += 1;
    Ok(id)
}

/// libpq connection string → PostgreSQL LookupStore params. Stalwart
/// connects as the `stalwart-mail` role via unix-peer auth (V001 grants
/// it just enough EXECUTE+SELECT for the sieve `query()` call), so the
/// URL's user is deliberately dropped.
struct PgParams {
    host: String,
    database: String,
}

fn parse_pg_url(url: &str) -> Result<PgParams> {
    let mut host: Option<String> = None;
    let mut database: Option<String> = None;
    for kv in url.split_whitespace() {
        if let Some((k, v)) = kv.split_once('=') {
            match k {
                "host" => host = Some(v.to_owned()),
                "dbname" | "database" => database = Some(v.to_owned()),
                _ => {}
            }
        }
    }
    Ok(PgParams {
        host: host.unwrap_or_else(|| "/run/postgresql".into()),
        database: database
            .ok_or_else(|| anyhow::anyhow!("connection string missing dbname/database"))?,
    })
}

/// StoreLookup `sql` → rampart's postgres. Stalwart's sieve
/// `query('sql', '...', [...])` resolves through this.
pub(super) async fn upsert_store_lookup(
    client: &JmapClient,
    stats: &mut Stats,
    database_url: &str,
    dry_run: bool,
) -> Result<()> {
    let pg = parse_pg_url(database_url)?;
    if let Some(id) = query_store_lookup_by_namespace(client, "sql").await? {
        let item = json!({
            "store": {
                "@type": "PostgreSql",
                "host": pg.host,
                "port": 5432_u32,
                "database": pg.database,
                "authUsername": "stalwart-mail",
                "authSecret": { "@type": "None" },
                "timeout": 5000_u64,
                "poolMaxConnections": 4_u32,
            }
        });
        if dry_run {
            println!("DRY RUN: StoreLookup/update {id}: {item:#}");
            return Ok(());
        }
        client.set_update("StoreLookup", &id, item).await?;
        stats.patched += 1;
        return Ok(());
    }
    let item = json!({
        "namespace": "sql",
        "store": {
            "@type": "PostgreSql",
            "host": pg.host,
            "port": 5432_u32,
            "database": pg.database,
            "authUsername": "stalwart-mail",
            "authSecret": { "@type": "None" },
            "timeout": 5000_u64,
            "poolMaxConnections": 4_u32,
        }
    });
    if dry_run {
        println!("DRY RUN: StoreLookup/set create: {item:#}");
        return Ok(());
    }
    client.set_create("StoreLookup", item).await?;
    stats.created += 1;
    Ok(())
}

async fn query_store_lookup_by_namespace(
    client: &JmapClient,
    namespace: &str,
) -> Result<Option<String>> {
    // `namespace` is a Field, not a queryable Filter — server-side filtering
    // returns 0 hits and re-create races into primaryKeyViolation. List all,
    // scan client-side.
    let resp = client
        .call(json!([
            ["x:StoreLookup/query", {}, "a"],
            ["x:StoreLookup/get",
             {"#ids": {"resultOf": "a", "name": "x:StoreLookup/query", "path": "/ids"},
              "properties": ["namespace"]},
             "b"]
        ]))
        .await?;
    let list = resp
        .get(1)
        .and_then(|v| v.get(1))
        .and_then(|v| v.get("list"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for item in list {
        let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let ns = item.get("namespace").and_then(|v| v.as_str()).unwrap_or("");
        if ns == namespace {
            return Ok(Some(id.to_owned()));
        }
    }
    Ok(None)
}

/// Push rendered Sieve into `SieveSystemScript` named `rampart_rcpt`.
/// Idempotent — creates if absent, replaces contents if drifted.
pub(crate) async fn upsert_sieve_script(
    client: &JmapClient,
    stats: &mut Stats,
    contents: &str,
    dry_run: bool,
) -> Result<()> {
    if let Some(id) = client
        .query_by_name("SieveSystemScript", "rampart_rcpt")
        .await?
    {
        let resp = client
            .call(json!([[
                "x:SieveSystemScript/get",
                { "ids": [id.clone()], "properties": ["contents"] },
                "a",
            ]]))
            .await?;
        let stored = resp
            .first()
            .and_then(|v| v.get(1))
            .and_then(|v| v.get("list"))
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .and_then(|v| v.get("contents"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if stored == contents {
            stats.skipped += 1;
            tracing::info!("SieveSystemScript rampart_rcpt already canonical, skipping");
            return Ok(());
        }
        if dry_run {
            println!(
                "DRY RUN: SieveSystemScript/update {id}: <{} bytes>",
                contents.len()
            );
            return Ok(());
        }
        client
            .set_update("SieveSystemScript", &id, json!({ "contents": contents }))
            .await
            .context("update SieveSystemScript")?;
        stats.patched += 1;
        return Ok(());
    }
    let item = json!({
        "name": "rampart_rcpt",
        "isActive": true,
        "contents": contents,
        "description": "rampart forward+reply router (regenerated by rampart-render-sieve from alias_domain rows)",
    });
    if dry_run {
        println!("DRY RUN: SieveSystemScript/set create rampart_rcpt");
        return Ok(());
    }
    client.set_create("SieveSystemScript", item).await?;
    stats.created += 1;
    Ok(())
}

/// One `rcpt_domain == '<alias>' → 'rampart_rcpt'` branch per managed alias
/// domain. Replaces all existing `'rampart_rcpt'`-then branches; preserves
/// operator-added branches alongside.
pub(crate) async fn patch_stage_rcpt_script(
    client: &JmapClient,
    stats: &mut Stats,
    alias_domains: &[String],
    dry_run: bool,
) -> Result<()> {
    let resp = client
        .call(json!([
            ["x:MtaStageRcpt/get", {"ids": [SINGLETON_ID]}, "a"]
        ]))
        .await?;
    let current = resp
        .first()
        .and_then(|v| v.get(1))
        .and_then(|v| v.get("list"))
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("MtaStageRcpt singleton not found"))?;
    let existing = current.get("script");
    let mut branches: Vec<(String, String)> = read_match_branches(existing)
        .into_iter()
        .filter(|(_, then)| then != "'rampart_rcpt'")
        .collect();
    // Internal LMTP domain must run the Sieve too — `allowRelaying`
    // accepts `rcpt_domain == 'internal.rampart.lmtp'` so the worker can
    // receive sieve-rewritten envelopes, but a public sender on port 25
    // can use that same predicate to bypass session.rcpt validation and
    // reach `rampart-<id>@internal.rampart.lmtp` directly — the worker
    // then forwards by alias id, weaponizing the host as an open
    // forwarder. Routing the internal domain through `rampart_rcpt`
    // triggers the template's `internal.rampart.lmtp` reject guard
    // (line 6-8 of rampart_rcpt.sieve.tmpl), which 5xx's the external
    // RCPT before queueing. Sieve runs once at session.rcpt; internal
    // relay dispatch via MtaOutboundStrategy doesn't re-invoke it, so
    // legitimate sieve-rewritten queue traffic still flows.
    let mut canonical: Vec<(String, String)> = std::iter::once((
        "rcpt_domain == 'internal.rampart.lmtp'".to_owned(),
        "'rampart_rcpt'".to_owned(),
    ))
    .chain(
        alias_domains
            .iter()
            .map(|d| (format!("rcpt_domain == '{d}'"), "'rampart_rcpt'".to_owned())),
    )
    .collect();
    canonical.append(&mut branches);
    let else_expr = read_else(existing, "false");
    let new_expr = build_expression(canonical.clone(), &else_expr);
    if existing == Some(&new_expr) {
        stats.skipped += 1;
        tracing::info!("MtaStageRcpt.script branches already canonical, skipping");
        return Ok(());
    }
    if dry_run {
        println!("DRY RUN: MtaStageRcpt/update script: {new_expr:#}");
        return Ok(());
    }
    client
        .set_update("MtaStageRcpt", SINGLETON_ID, json!({ "script": new_expr }))
        .await?;
    stats.patched += 1;
    Ok(())
}

/// Relax mustMatchSender for rampart-notifier so the worker can submit as
/// arbitrary alias addresses; all other authed submissions stay strict.
pub(super) async fn patch_must_match_sender(
    client: &JmapClient,
    stats: &mut Stats,
    rampart_notifier_address: &str,
    dry_run: bool,
) -> Result<()> {
    let resp = client
        .call(json!([
            ["x:MtaStageAuth/get", {"ids": [SINGLETON_ID]}, "a"]
        ]))
        .await?;
    let current = resp
        .first()
        .and_then(|v| v.get(1))
        .and_then(|v| v.get("list"))
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("MtaStageAuth singleton not found"))?;
    let existing = current.get("mustMatchSender");
    // Strip any `authenticated_as == '<addr>' → 'false'` exemption — those
    // are the shape rampart owns. Renaming the notifier address would otherwise
    // leave the old exemption alongside the new one.
    let mut branches: Vec<(String, String)> = read_match_branches(existing)
        .into_iter()
        .filter(|(if_, then)| {
            !(then == "false" && if_.starts_with("authenticated_as == '") && if_.ends_with('\''))
        })
        .collect();
    let canonical_branch = (
        format!("authenticated_as == '{rampart_notifier_address}'"),
        "false".to_owned(),
    );
    let mut canonical = vec![canonical_branch];
    canonical.append(&mut branches);
    let else_expr = read_else(existing, "true");
    let new_expr = build_expression(canonical.clone(), &else_expr);
    if existing == Some(&new_expr) {
        stats.skipped += 1;
        tracing::info!("MtaStageAuth.mustMatchSender already canonical, skipping");
        return Ok(());
    }
    if dry_run {
        println!("DRY RUN: MtaStageAuth/update mustMatchSender: {new_expr:#}");
        return Ok(());
    }
    client
        .set_update(
            "MtaStageAuth",
            SINGLETON_ID,
            json!({ "mustMatchSender": new_expr }),
        )
        .await?;
    stats.patched += 1;
    Ok(())
}
