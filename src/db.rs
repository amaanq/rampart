//! Postgres connection pool and the typed row structs we actually use.
//! Pool via deadpool-postgres; dedicated client for migrations.
//! Phase-1 deployment uses local Unix-socket postgres with NoTls.

use anyhow::{Context, Result};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

pub fn build_pool(url: &str) -> Result<Pool> {
    let mut cfg = Config::new();
    cfg.url = Some(url.to_owned());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .context("failed to build deadpool-postgres pool")?;
    Ok(pool)
}

pub async fn connect_once(url: &str) -> Result<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(url, NoTls)
        .await
        .with_context(|| format!("connecting to postgres: {url}"))?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!(error = ?e, "postgres connection task ended with error");
        }
    });
    Ok(client)
}
