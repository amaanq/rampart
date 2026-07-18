//! Thin async JMAP client over reqwest, scoped to the registry-object
//! method shape (`x:<T>/get|set|query`).

use std::time::Duration;

use anyhow::{
   Context as _,
   Result,
   bail,
};
use serde_json::{
   Value,
   json,
};
use tokio::time;

use crate::domain_setup::DkimRecord;

pub struct JmapClient {
   http:        reqwest::Client,
   base_url:    String,
   auth_header: String,
}

impl JmapClient {
   pub fn new(base_url: &str, user: &str, password: &str) -> Result<Self> {
      let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            // Localhost stalwart uses self-signed certs.
            .danger_accept_invalid_certs(true)
            .build()
            .context("building reqwest client")?;
      let creds = data_encoding::BASE64.encode(format!("{user}:{password}").as_bytes());
      let auth_header = format!("Basic {creds}");
      Ok(Self {
         http,
         base_url: base_url.trim_end_matches('/').to_owned(),
         auth_header,
      })
   }

   pub(super) async fn wait_until_ready(&self) -> Result<()> {
      let waits = [1, 2, 4, 8, 16, 16, 16];
      for (i, secs) in waits.iter().enumerate() {
         match self.session_object().await {
            Ok(_) => {
               tracing::info!(attempt = i + 1, "jmap ready");
               return Ok(());
            },
            Err(err) => {
               tracing::warn!(attempt = i + 1, error = ?err, "jmap not ready yet");
               time::sleep(Duration::from_secs(*secs)).await;
            },
         }
      }
      bail!("jmap never became ready after ~60s of retries");
   }

   async fn session_object(&self) -> Result<Value> {
      let url = format!("{}/.well-known/jmap", self.base_url);
      let resp = self
         .http
         .get(&url)
         .header("Authorization", &self.auth_header)
         .send()
         .await
         .context("GET jmap session")?;
      let status = resp.status();
      if !status.is_success() {
         bail!("jmap session: HTTP {status}");
      }
      resp.json::<Value>().await.context("decode session json")
   }

   /// Send a single methodCalls envelope. Returns `methodResponses`.
   pub(super) async fn call(&self, method_calls: Value) -> Result<Vec<Value>> {
      let body = json!({
          // x:Mta*/x:Account live under core+mail; unknown URN → notRequest.
          "using": [
              "urn:ietf:params:jmap:core",
              "urn:ietf:params:jmap:mail",
          ],
          "methodCalls": method_calls,
      });
      let url = format!("{}/jmap", self.base_url);
      let resp = self
         .http
         .post(&url)
         .header("Authorization", &self.auth_header)
         .json(&body)
         .send()
         .await
         .context("POST /jmap")?;
      let status = resp.status();
      if status.as_u16() == 401 || status.as_u16() == 403 {
         bail!("jmap auth: HTTP {status}");
      }
      if !status.is_success() {
         let text = resp.text().await.unwrap_or_default();
         bail!("jmap: HTTP {status}: {text}");
      }
      let value: Value = resp.json().await.context("decode jmap response")?;
      let mr = value
         .get("methodResponses")
         .and_then(|x| x.as_array())
         .cloned()
         .ok_or_else(|| anyhow::anyhow!("jmap: no methodResponses in {value}"))?;
      Ok(mr)
   }

   /// Body of the first method call, asserting it matches `expected`.
   /// Rejects `["error", {...}, "0"]` shapes.
   pub(super) fn first_body(responses: &[Value], expected: &str) -> Result<Value> {
      let inner = responses
         .first()
         .and_then(|value| value.as_array())
         .ok_or_else(|| anyhow::anyhow!("jmap: empty methodResponses"))?;
      let method = inner
         .first()
         .and_then(|value| value.as_str())
         .ok_or_else(|| anyhow::anyhow!("jmap: missing method name in response"))?;
      if method == "error" {
         let body = inner.get(1).cloned().unwrap_or(Value::Null);
         bail!("jmap method error: {body}");
      }
      if method != expected {
         bail!("jmap: expected {expected}, got {method}: {inner:?}");
      }
      inner
         .get(1)
         .cloned()
         .ok_or_else(|| anyhow::anyhow!("jmap: missing body for {method}"))
   }

   /// First matching id from `x:<T>/query name=...`.
   pub(super) async fn query_by_name(&self, object: &str, name: &str) -> Result<Option<String>> {
      let ids = self.query_ids_by_name(object, name).await?;
      Ok(ids.into_iter().next())
   }

   /// All matching ids in server order. Callers (e.g. Account, where the
   /// same local-part exists across domains) follow up with a get to
   /// disambiguate.
   pub(super) async fn query_ids_by_name(&self, object: &str, name: &str) -> Result<Vec<String>> {
      let calls = json!([[
          format!("x:{object}/query"),
          { "filter": { "name": name } },
          "0"
      ]]);
      let mr = self.call(calls).await?;
      let body = Self::first_body(&mr, &format!("x:{object}/query"))?;
      Ok(body
         .get("ids")
         .and_then(|value| value.as_array())
         .map(|arr| {
            arr.iter()
               .filter_map(|value| value.as_str().map(str::to_owned))
               .collect()
         })
         .unwrap_or_default())
   }

   /// First object from `x:<T>/get id=...`.
   pub(super) async fn get_by_id(&self, object: &str, id: &str) -> Result<Option<Value>> {
      let calls = json!([[
          format!("x:{object}/get"),
          { "ids": [id] },
          "0"
      ]]);
      let mr = self.call(calls).await?;
      let body = Self::first_body(&mr, &format!("x:{object}/get"))?;
      let list = body.get("list").and_then(|value| value.as_array());
      Ok(list.and_then(|arr| arr.first()).cloned())
   }

   /// DNS records for every active signing key on a named Domain.
   pub(crate) async fn dkim_dns_records_for_domain(
      &self,
      domain_name: &str,
   ) -> Result<Vec<DkimRecord>> {
      let Some(domain_id) = self.query_by_name("Domain", domain_name).await? else {
         return Ok(Vec::new());
      };
      let ids = self.query_dkim_signatures_for(&domain_id).await?;
      let mut records = Vec::with_capacity(ids.len());
      for id in ids {
         let calls = json!([[
             "x:DkimSignature/get",
             {
                 "ids": [id],
                 "properties": ["@type", "selector", "stage", "publicKey"],
             },
             "0"
         ]]);
         let responses = self.call(calls).await?;
         let body = Self::first_body(&responses, "x:DkimSignature/get")?;
         let Some(item) = body
            .get("list")
            .and_then(Value::as_array)
            .and_then(|list| list.first())
         else {
            continue;
         };
         if !item
            .get("stage")
            .and_then(Value::as_str)
            .is_some_and(|stage| stage.eq_ignore_ascii_case("active"))
         {
            continue;
         }
         let algorithm = match item.get("@type").and_then(Value::as_str) {
            Some("Dkim1RsaSha256") => "rsa",
            Some("Dkim1Ed25519Sha256") => "ed25519",
            _ => continue,
         };
         let Some(selector) = item.get("selector").and_then(Value::as_str) else {
            continue;
         };
         let Some(public_key) = item.get("publicKey").and_then(Value::as_str) else {
            continue;
         };
         records.push(DkimRecord {
            algorithm: algorithm.to_owned(),
            selector:  selector.to_owned(),
            value:     format!("v=DKIM1; k={algorithm}; h=sha256; p={public_key}"),
         });
      }
      records.sort_by(|lhs, rhs| lhs.algorithm.cmp(&rhs.algorithm));
      Ok(records)
   }

   /// Create one item under temp id "i0"; returns the server-assigned id.
   pub(super) async fn set_create(&self, object: &str, item: Value) -> Result<String> {
      let calls = json!([[
          format!("x:{object}/set"),
          { "create": { "i0": item } },
          "0"
      ]]);
      let mr = self.call(calls).await?;
      let body = Self::first_body(&mr, &format!("x:{object}/set"))?;
      if let Some(not_created) = body.get("notCreated").and_then(|value| value.as_object())
         && !not_created.is_empty()
      {
         bail!("jmap create failed: {not_created:?}");
      }
      let id = body
         .get("created")
         .and_then(|created| created.get("i0"))
         .and_then(|value| value.get("id"))
         .and_then(|value| value.as_str())
         .ok_or_else(|| anyhow::anyhow!("jmap: created.i0.id missing in {body}"))?
         .to_owned();
      Ok(id)
   }

   /// Reload settings so registry mutations land at the SMTP/queue layer.
   pub(crate) async fn reload_settings(&self) -> Result<()> {
      let calls = json!([[
          "x:Action/set",
          { "create": { "i0": { "@type": "ReloadSettings" } } },
          "0"
      ]]);
      let mr = self.call(calls).await?;
      Self::first_body(&mr, "x:Action/set")?;
      Ok(())
   }

   /// Partial update for one id.
   pub(super) async fn set_update(&self, object: &str, id: &str, patch: Value) -> Result<()> {
      let calls = json!([[
          format!("x:{object}/set"),
          { "update": { id: patch } },
          "0"
      ]]);
      let mr = self.call(calls).await?;
      let body = Self::first_body(&mr, &format!("x:{object}/set"))?;
      if let Some(not_updated) = body.get("notUpdated").and_then(|value| value.as_object())
         && !not_updated.is_empty()
      {
         bail!("jmap update failed: {not_updated:?}");
      }
      Ok(())
   }

   /// Destroy ids of the same type, chunked under stalwart's setMaxObjects
   /// (500).
   pub(super) async fn set_destroy(&self, object: &str, ids: &[String]) -> Result<()> {
      const SET_CHUNK: usize = 256;
      if ids.is_empty() {
         return Ok(());
      }
      for chunk in ids.chunks(SET_CHUNK) {
         let calls = json!([[
             format!("x:{object}/set"),
             { "destroy": chunk },
             "0"
         ]]);
         let mr = self.call(calls).await?;
         let body = Self::first_body(&mr, &format!("x:{object}/set"))?;
         if let Some(not_destroyed) = body.get("notDestroyed").and_then(|value| value.as_object())
            && !not_destroyed.is_empty()
         {
            bail!("jmap destroy failed: {not_destroyed:?}");
         }
      }
      Ok(())
   }

   /// Page `DkimSignature` ids by `domainId` past queryMaxResults.
   pub(super) async fn query_dkim_signatures_for(&self, domain_id: &str) -> Result<Vec<String>> {
      const QUERY_LIMIT: u64 = 256;
      const MAX_PAGES: u32 = 1024;
      let mut out: Vec<String> = Vec::new();
      let mut position: u64 = 0;
      for _ in 0..MAX_PAGES {
         let calls = json!([[
             "x:DkimSignature/query",
             {
                 "filter": { "domainId": domain_id },
                 "position": position,
                 "limit": QUERY_LIMIT,
             },
             "0"
         ]]);
         let mr = self.call(calls).await?;
         let body = Self::first_body(&mr, "x:DkimSignature/query")?;
         let ids: Vec<String> = body
            .get("ids")
            .and_then(|value| value.as_array())
            .map(|arr| {
               arr.iter()
                  .filter_map(|value| value.as_str().map(str::to_owned))
                  .collect()
            })
            .unwrap_or_default();
         if ids.is_empty() {
            return Ok(out);
         }
         let n = ids.len() as u64;
         position += n;
         out.extend(ids);
         if n < QUERY_LIMIT {
            return Ok(out);
         }
      }
      bail!("query_dkim_signatures_for exceeded MAX_PAGES; check stalwart pagination");
   }

   /// Page every `T` past queryMaxResults / getMaxResults; a single
   /// unfiltered query→get silently drops the tail.
   pub(super) async fn get_all(&self, object: &str, properties: &[&str]) -> Result<Vec<Value>> {
      const QUERY_LIMIT: u64 = 256;
      const MAX_PAGES: u32 = 1024;
      let mut out = Vec::<Value>::new();
      let mut position: u64 = 0;
      let query_method = format!("x:{object}/query");
      let get_method = format!("x:{object}/get");
      for _ in 0..MAX_PAGES {
         let calls = json!([
             [
                 query_method,
                 { "position": position, "limit": QUERY_LIMIT },
                 "q"
             ],
             [
                 get_method,
                 {
                     "#ids": {
                         "resultOf": "q",
                         "name": query_method,
                         "path": "/ids"
                     },
                     "properties": properties,
                 },
                 "g"
             ]
         ]);
         let mr = self.call(calls).await?;
         // get's response lives at responses[1], so find by method name
         // (first_body only checks responses[0]).
         let body = mr
            .iter()
            .find(|value| {
               value
                  .as_array()
                  .and_then(|arr| arr.first())
                  .and_then(|method| method.as_str())
                  == Some(get_method.as_str())
            })
            .and_then(|value| value.as_array())
            .and_then(|arr| arr.get(1))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("jmap: missing {get_method} in chained response"))?;
         let list: Vec<Value> = body
            .get("list")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
         if list.is_empty() {
            return Ok(out);
         }
         let n = list.len() as u64;
         position += n;
         out.extend(list);
         if n < QUERY_LIMIT {
            return Ok(out);
         }
      }
      bail!("get_all({object}) exceeded MAX_PAGES; check stalwart pagination")
   }
}

/// `Expression` value: `{if, then}` branches plus `else`. `match` is an
/// OBJECT keyed by stringified positional indexes; arrays silently drop
/// branches.
pub(super) fn build_expression(branches: Vec<(String, String)>, else_expr: &str) -> Value {
   let mut map = serde_json::Map::new();
   for (i, (if_, then)) in branches.into_iter().enumerate() {
      map.insert(i.to_string(), json!({ "if": if_, "then": then }));
   }
   json!({ "match": Value::Object(map), "else": else_expr })
}

/// Existing `match` branches. Accepts object and array forms.
pub(super) fn read_match_branches(expr: Option<&Value>) -> Vec<(String, String)> {
   fn extract(branch: &Value) -> Option<(String, String)> {
      let if_ = branch
         .get("if")
         .and_then(|value| value.as_str())?
         .to_owned();
      let then = branch
         .get("then")
         .and_then(|value| value.as_str())?
         .to_owned();
      Some((if_, then))
   }
   let Some(match_obj) = expr.and_then(|value| value.get("match")) else {
      return Vec::new();
   };
   match_obj.as_object().map_or_else(
      || {
         match_obj
            .as_array()
            .map_or_else(Vec::new, |arr| arr.iter().filter_map(extract).collect())
      },
      |obj| {
         // Numeric sort — string sort puts "10" before "2".
         let mut entries: Vec<(usize, &Value)> = obj
            .iter()
            .filter_map(|(key, value)| key.parse::<usize>().ok().map(|i| (i, value)))
            .collect();
         entries.sort_by_key(|&(i, _)| i);
         entries
            .into_iter()
            .filter_map(|(_, value)| extract(value))
            .collect()
      },
   )
}

pub(super) fn read_else(expr: Option<&Value>, fallback: &str) -> String {
   expr
      .and_then(|value| value.get("else"))
      .and_then(|value| value.as_str())
      .unwrap_or(fallback)
      .to_owned()
}
