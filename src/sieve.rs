//! Render the session.rcpt Sieve script from current alias_domain rows.
//! Used by `rampart admin render-sieve` and the domain CRUD handlers.

use anyhow::{Context, Result};
use askama::Template;
use rampart_codegen::queries::sieve;
use std::path::Path;

#[derive(Template)]
#[template(path = "rampart_rcpt.sieve.tmpl", escape = "none")]
struct SieveRcpt<'a> {
    domains: &'a [String],
}

pub async fn render_for_domains(domains: &[String]) -> Result<String> {
    Ok(SieveRcpt { domains }.render()?)
}

pub async fn render(client: &tokio_postgres::Client) -> Result<String> {
    let domains = sieve::all_alias_domain_names().bind(client).all().await?;
    render_for_domains(&domains).await
}

/// Atomically write `bytes` to `path` via temp+rename. Stalwart loads
/// the file with `%{file:...}%` at startup; a half-written file would
/// be visible. Temp name is unique per call so concurrent renders
/// can't truncate each other's bytes. Pair with the locked renderer
/// when DB snapshot order matters.
pub fn atomic_write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};
    let dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("sieve path has no parent: {}", path.display()))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("sieve path has no filename: {}", path.display()))?
        .to_string_lossy();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let suffix: u64 = rand::random();
    let tmp = dir.join(format!(
        ".{file_name}.tmp.{}.{nanos}.{suffix:x}",
        std::process::id()
    ));
    let written = (|| -> Result<()> {
        let mut f = std::fs::File::create(&tmp)
            .with_context(|| format!("create temp {}", tmp.display()))?;
        f.write_all(bytes)
            .with_context(|| format!("write temp {}", tmp.display()))?;
        f.sync_all()
            .with_context(|| format!("fsync temp {}", tmp.display()))?;
        Ok(())
    })();
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(anyhow::anyhow!(
            "rename {} -> {}: {e}",
            tmp.display(),
            path.display()
        ));
    }
    Ok(())
}

/// Truncated FNV-1a of `"rampart:sieve_render"`. Any value works; this
/// just needs to be the same across all renderers.
pub const SIEVE_RENDER_LOCK_KEY: i64 = 0x534C5F5345565F31;

/// Render and atomically write under a transaction-scoped advisory
/// lock. Lock releases on commit, rollback, or dropped connection.
pub async fn render_and_write_locked(
    client: &mut tokio_postgres::Client,
    path: &Path,
) -> Result<()> {
    render_write_and_sync_locked(client, path, |_, _| async { Ok(()) }).await
}

/// Like `render_and_write_locked` but also runs `sync` while holding
/// the lock, so the JMAP push and the file write share a snapshot of
/// alias_domain.
pub async fn render_write_and_sync_locked<F, Fut>(
    client: &mut tokio_postgres::Client,
    path: &Path,
    sync: F,
) -> Result<()>
where
    F: FnOnce(Vec<String>, String) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let txn = client
        .transaction()
        .await
        .context("begin txn for sieve render")?;
    // Cluster-side idle_in_transaction_session_timeout could otherwise
    // kill us mid-JMAP and release the lock.
    txn.execute("SET LOCAL idle_in_transaction_session_timeout = 0", &[])
        .await
        .context("disable idle-in-txn timeout for sieve lock")?;
    txn.execute(
        "SELECT pg_advisory_xact_lock($1::bigint)",
        &[&SIEVE_RENDER_LOCK_KEY],
    )
    .await
    .context("pg_advisory_xact_lock for sieve render")?;
    let domains = sieve::all_alias_domain_names().bind(&txn).all().await?;
    let rendered = render_for_domains(&domains).await?;
    atomic_write_file(path, rendered.as_bytes())?;
    sync(domains, rendered).await?;
    txn.commit().await.context("commit sieve render txn")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::render_for_domains;

    // Empty domain set must not emit `if anyof ()` — Sieve parses it
    // but the shape is brittle; the internal-LMTP loop guard must
    // still be present.
    #[tokio::test]
    async fn empty_domains_no_anyof_block() {
        let s = render_for_domains(&[]).await.expect("render");
        assert!(s.contains("internal.rampart.lmtp"), "loop guard missing");
        assert!(
            !s.contains("if anyof ("),
            "empty domain set must not emit `if anyof (`"
        );
        assert!(
            !s.contains("rampart_resolve_or_create"),
            "no managed block, no resolver call"
        );
    }

    #[tokio::test]
    async fn non_empty_domains_emit_anyof() {
        let domains = [
            "addy.example.com".to_owned(),
            "alias.example.org".to_owned(),
        ];
        let s = render_for_domains(&domains).await.expect("render");
        assert!(s.contains("if anyof ("));
        assert!(s.contains("addy.example.com"));
        assert!(s.contains("alias.example.org"));
        assert!(s.contains("rampart_resolve_or_create"));
    }
}
