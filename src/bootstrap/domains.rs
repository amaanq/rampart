//! Alias-domain marker logic — `Domain.description` distinguishes
//! rampart-managed Domains from user-managed ones, so reconcile never destroys
//! unrelated mail domains.

use anyhow::{
   Result,
   bail,
};
use serde_json::{
   Value,
   json,
};

use super::{
   Stats,
   jmap::JmapClient,
};

const RAMPART_DOMAIN_MARKER: &str = "[rampart-managed]";

pub(crate) async fn upsert_managed_alias_domain(
   client: &JmapClient,
   stats: &mut Stats,
   name: &str,
   dry_run: bool,
) -> Result<()> {
   if let Some(id) = client.query_by_name("Domain", name).await? {
      // Never stamp the marker onto an existing un-marked Domain —
      // reconcile would later destroy a Domain rampart never owned. When the
      // marker IS present, re-assert canonical state to heal partial drift.
      let body = client
         .get_by_id("Domain", &id)
         .await?
         .unwrap_or(Value::Null);
      let current = body
         .get("description")
         .and_then(|v| v.as_str())
         .unwrap_or("");
      if current == RAMPART_DOMAIN_MARKER {
         if dry_run {
            println!("DRY RUN: would heal marker-owned Domain/{id} ({name}) to canonical state");
            stats.skipped += 1;
            return Ok(());
         }
         if !is_canonical_managed(&body) {
            client
               .set_update("Domain", &id, canonical_managed_patch())
               .await?;
            stats.patched += 1;
            tracing::info!(
               domain = name,
               "marker-owned Domain re-asserted to canonical state"
            );
         } else {
            stats.skipped += 1;
            tracing::info!(
               domain = name,
               "marker-owned Domain already canonical, skipping"
            );
         }
         // Stalwart only schedules DKIM keygen on Manual→Automatic;
         // Automatic→Automatic is a no-op even with no keys present.
         // Force the transition if signatures are missing.
         let dkim_ids = client.query_dkim_signatures_for(&id).await?;
         if dkim_ids.is_empty() {
            tracing::warn!(
               domain = name,
               "marker-owned Domain has zero DkimSignatures; toggling DKIM to force key \
                regeneration"
            );
            client
               .set_update(
                  "Domain",
                  &id,
                  json!({ "dkimManagement": { "@type": "Manual" } }),
               )
               .await?;
            client
               .set_update(
                  "Domain",
                  &id,
                  json!({ "dkimManagement": canonical_dkim_management() }),
               )
               .await?;
            stats.patched += 1;
         }
      } else {
         stats.skipped += 1;
         tracing::info!(
            domain = name,
            description = current,
            "alias Domain object already exists (NOT rampart-managed); routing through it without \
             taking ownership"
         );
      }
      return Ok(());
   }
   let mut item = canonical_managed_patch();
   if let Some(obj) = item.as_object_mut() {
      obj.insert("name".into(), json!(name));
   }
   if dry_run {
      println!("DRY RUN: alias Domain/set create: {item:#}");
      return Ok(());
   }
   let _id = client.set_create("Domain", item).await?;
   stats.created += 1;
   tracing::info!(domain = name, "created alias Domain with auto-DKIM");
   Ok(())
}

/// Fields we own on a marker-stamped Domain — create body (`name` injected)
/// and heal-patch body. Mirror in `is_canonical_managed`.
fn canonical_managed_patch() -> Value {
   json!({
       "isEnabled": true,
       "description": RAMPART_DOMAIN_MARKER,
       "dkimManagement": canonical_dkim_management(),
       "certificateManagement": { "@type": "Manual" },
       "dnsManagement":         { "@type": "Manual" },
       "subAddressing":         { "@type": "Enabled" },
       "allowRelaying": false,
   })
}

fn canonical_dkim_management() -> Value {
   json!({
       "@type": "Automatic",
       "selectorTemplate": "v{version}-{algorithm}-{date-%Y%m%d}",
       "algorithms": {
           "Dkim1Ed25519Sha256": true,
           "Dkim1RsaSha256": true,
       },
       "rotateAfter": 7_776_000_000_u64, // 90d
       "retireAfter":   604_800_000_u64, // 7d
       "deleteAfter": 2_592_000_000_u64, // 30d
   })
}

fn is_canonical_managed(body: &Value) -> bool {
   body.get("isEnabled").and_then(|v| v.as_bool()) == Some(true)
      && body.get("description").and_then(|v| v.as_str()) == Some(RAMPART_DOMAIN_MARKER)
      && body.get("allowRelaying").and_then(|v| v.as_bool()) == Some(false)
      && body
         .get("subAddressing")
         .and_then(|v| v.get("@type"))
         .and_then(|v| v.as_str())
         == Some("Enabled")
      && body
         .get("certificateManagement")
         .and_then(|v| v.get("@type"))
         .and_then(|v| v.as_str())
         == Some("Manual")
      && body
         .get("dnsManagement")
         .and_then(|v| v.get("@type"))
         .and_then(|v| v.as_str())
         == Some("Manual")
      && body
         .get("dkimManagement")
         .and_then(|v| v.get("@type"))
         .and_then(|v| v.as_str())
         == Some("Automatic")
}

/// Destroy rampart-managed Domains not in `keep`. DkimSignature children must
/// go first (stalwart returns `objectIsLinked` otherwise). Bails early if
/// `notifier_domain` is in the destroy set, since the Account → Domain FK
/// would fail mid-chain after we've already disabled the Domain.
pub(crate) async fn reconcile_alias_domains(
   client: &JmapClient,
   stats: &mut Stats,
   keep: &[String],
   notifier_domain: &str,
   dry_run: bool,
) -> Result<()> {
   let all = client
      .get_all("Domain", &["id", "name", "description"])
      .await?;
   let mut to_destroy = Vec::<(String, String)>::new();
   for item in &all {
      let id = item.get("id").and_then(|v| v.as_str()).unwrap_or_default();
      let name = item
         .get("name")
         .and_then(|v| v.as_str())
         .unwrap_or_default();
      let desc = item
         .get("description")
         .and_then(|v| v.as_str())
         .unwrap_or_default();
      if id.is_empty() || name.is_empty() || desc != RAMPART_DOMAIN_MARKER {
         continue;
      }
      if keep.iter().any(|k| k.eq_ignore_ascii_case(name)) {
         continue;
      }
      if name.eq_ignore_ascii_case(notifier_domain) {
         bail!(
            "refusing to reconcile: alias-domain '{name}' hosts the rampart-notifier \
             ({notifier_domain}); stalwart will reject the Domain destroy because Account has an \
             FK and we'd already have disabled the Domain and destroyed its DKIM keys. Move the \
             notifier to a different domain or re-add '{name}' to rampart's alias_domain table."
         );
      }
      to_destroy.push((id.to_owned(), name.to_owned()));
   }
   for (id, name) in to_destroy {
      if dry_run {
         println!("DRY RUN: would destroy stale Domain/{id} ({name})");
         continue;
      }
      // Disable BEFORE wiping DKIM keys — mid-chain failure leaves a
      // disabled Domain instead of an enabled-but-keyless one that
      // would silently break outbound signing for queued mail.
      client
         .set_update("Domain", &id, json!({ "isEnabled": false }))
         .await?;
      let dkim_ids = client.query_dkim_signatures_for(&id).await?;
      if !dkim_ids.is_empty() {
         client.set_destroy("DkimSignature", &dkim_ids).await?;
      }
      client.set_destroy("Domain", &[id.clone()]).await?;
      stats.patched += 1;
      tracing::info!(
         domain = name,
         id,
         dkim_destroyed = dkim_ids.len(),
         "destroyed stale rampart-managed Domain"
      );
   }
   Ok(())
}
