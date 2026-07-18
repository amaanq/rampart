//! Operator subcommands. Runs against the DB directly — no HTTP.

use anyhow::{Context, Result, bail};
use data_encoding::BASE64URL_NOPAD;
use hmac_sha256::Hash;
use rampart_codegen::queries::{
    aliases, api_keys, domains, gc as gc_q, mailboxes, sessions, tokens, users,
};
use std::path::PathBuf;
use time::{Duration, OffsetDateTime};

use crate::auth::hash_password;
use crate::db;

pub async fn render_sieve(url: &str, output: Option<PathBuf>) -> Result<()> {
    let mut client = db::connect_once(url).await?;
    match output {
        Some(path) => {
            crate::sieve::render_and_write_locked(&mut client, &path)
                .await
                .with_context(|| format!("writing sieve to {}", path.display()))?;
            tracing::info!(path = %path.display(), "rendered sieve");
        }
        None => {
            let rendered = crate::sieve::render(&client).await?;
            print!("{rendered}");
        }
    }
    Ok(())
}

pub async fn user_add(
    url: &str,
    email: String,
    display_name: Option<String>,
    is_admin: bool,
    password: String,
) -> Result<()> {
    if password.len() < 10 {
        bail!("password must be at least 10 characters");
    }
    let hash = hash_password(&password)?;
    let c = db::connect_once(url).await?;
    let id = users::create()
        .bind(&c, &email, &Some(hash), &display_name, &is_admin)
        .one()
        .await
        .context("inserting user")?;
    tracing::info!(id, email = %email, is_admin, "user added");
    Ok(())
}

pub async fn user_list(url: &str) -> Result<()> {
    let c = db::connect_once(url).await?;
    let rows = users::list_cli().bind(&c).all().await?;
    println!("{:>3}  {:<32}  admin  enabled  display_name", "id", "email");
    for u in rows {
        println!(
            "{:>3}  {:<32}  {:<5}  {:<7}  {}",
            u.id,
            u.email,
            u.is_admin,
            u.enabled,
            u.display_name.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

pub async fn user_disable(url: &str, email: String) -> Result<()> {
    let mut c = db::connect_once(url).await?;
    let Some(id) = users::by_email_id_unfiltered()
        .bind(&c, &email)
        .opt()
        .await?
    else {
        bail!("no user with email '{email}'")
    };
    let txn = c.transaction().await?;
    users::disable().bind(&txn, &id).await?;
    sessions::delete_by_user().bind(&txn, &id).await?;
    api_keys::revoke_all_for_user().bind(&txn, &id).await?;
    aliases::disable_all_for_user().bind(&txn, &id).await?;
    txn.commit().await?;
    tracing::info!(id, email = %email, "user disabled (sessions cleared, api_keys revoked, aliases disabled)");
    Ok(())
}

pub async fn reset_password(url: &str, email: String) -> Result<()> {
    let c = db::connect_once(url).await?;
    let id = users::by_email_id_unfiltered()
        .bind(&c, &email)
        .opt()
        .await?
        .with_context(|| format!("no user with email '{email}'"))?;
    let new_password = random_password();
    let hash = hash_password(&new_password)?;
    users::set_password().bind(&c, &Some(hash), &id).await?;
    sessions::delete_by_user().bind(&c, &id).await?;
    println!("password reset for {email}");
    println!("new password: {new_password}");
    println!("(all existing sessions invalidated)");
    Ok(())
}

pub async fn invite(url: &str, preset_email: Option<String>) -> Result<()> {
    let c = db::connect_once(url).await?;
    let token = random_token(24);
    let hash = Hash::hash(token.as_bytes()).to_vec();
    let expires = OffsetDateTime::now_utc() + Duration::hours(24 * 7);
    tokens::invite_create()
        .bind(&c, &hash, &preset_email, &expires)
        .await?;
    let ts = expires
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    println!("invite token: {token}");
    println!("expires:      {ts}");
    println!("give this to the friend. They visit /signup/{token}");
    if let Some(email) = preset_email {
        println!("preset email: {email} (they must sign up with exactly this)");
    }
    Ok(())
}

pub async fn add_mailbox(
    url: &str,
    user_email: String,
    email: String,
    display_name: Option<String>,
) -> Result<()> {
    // Reject garbage before it lands as a verified mailbox — a typo like
    // alice@@example survives the schema's CITEXT column and only fails
    // later inside submit() at first forward, tempfailing inbound mail.
    use std::str::FromStr;
    lettre::Address::from_str(&email)
        .with_context(|| format!("invalid email address '{email}'"))?;
    let c = db::connect_once(url).await?;
    let user_id = users::by_email_id_unfiltered()
        .bind(&c, &user_email)
        .opt()
        .await?
        .with_context(|| format!("no user '{user_email}'"))?;
    let id = mailboxes::create_verified()
        .bind(&c, &user_id, &email, &display_name)
        .one()
        .await
        .context("inserting mailbox")?;
    tracing::info!(id, user_email = %user_email, email = %email, "mailbox added (verified=true)");
    Ok(())
}

pub async fn list_mailboxes(url: &str, user_email: Option<String>) -> Result<()> {
    let c = db::connect_once(url).await?;
    let rows = if let Some(ue) = user_email {
        let uid = users::by_email_id_unfiltered()
            .bind(&c, &ue)
            .opt()
            .await?
            .with_context(|| format!("no user '{ue}'"))?;
        mailboxes::list_admin_for_user()
            .bind(&c, &uid)
            .all()
            .await?
    } else {
        mailboxes::list_admin().bind(&c).all().await?
    };
    println!(
        "{:>3}  {:<24}  {:<32}  verified  enabled  display_name",
        "id", "user", "email"
    );
    for m in rows {
        println!(
            "{:>3}  {:<24}  {:<32}  {:<8}  {:<7}  {}",
            m.id,
            m.user_email,
            m.email,
            m.verified,
            m.enabled,
            m.display_name.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

pub async fn export_aliases(url: &str, user_email: Option<String>) -> Result<()> {
    let c = db::connect_once(url).await?;
    let rows = if let Some(ue) = user_email {
        aliases::export_for_user().bind(&c, &ue).all().await?
    } else {
        aliases::export().bind(&c).all().await?
    };
    println!("user,address,mailbox,enabled,pinned,note");
    for a in rows {
        println!(
            "{},{},{},{},{},{}",
            csv_field(&a.user_email),
            csv_field(&a.address),
            csv_field(&a.mailbox),
            a.enabled,
            a.pinned,
            csv_field(a.note.as_deref().unwrap_or(""))
        );
    }
    Ok(())
}

pub async fn import_aliases(url: &str, path: PathBuf) -> Result<()> {
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut c = db::connect_once(url).await?;
    let mut records = parse_csv(&content).into_iter();
    let _header = records.next();
    let mut imported = 0usize;
    let mut skipped_cap = 0usize;
    for record in records {
        if record.iter().all(|s| s.is_empty()) {
            continue;
        }
        if record.len() < 6 {
            tracing::warn!(?record, "skipping malformed record");
            continue;
        }
        let user_email = record[0].clone();
        let address = record[1].clone();
        let mailbox_email = record[2].clone();
        let enabled: bool = record[3].trim().parse().unwrap_or(true);
        let pinned: bool = record[4].trim().parse().unwrap_or(false);
        let note = record[5].clone();

        let Some(user_id) = users::by_email_id_unfiltered()
            .bind(&c, &user_email)
            .opt()
            .await?
        else {
            tracing::warn!(user_email, "user not found, skipping");
            continue;
        };

        let Some(mailbox_id) = mailboxes::id_for_user_email()
            .bind(&c, &user_id, &mailbox_email)
            .opt()
            .await?
        else {
            tracing::warn!(
                user_email,
                mailbox_email,
                "mailbox not found, disabled, or unverified — skipping"
            );
            continue;
        };

        let Some((_, domain)) = address.split_once('@') else {
            tracing::warn!(address, "bad address, skipping");
            continue;
        };
        let Some(domain_id) = domains::id_by_domain().bind(&c, &domain).opt().await? else {
            tracing::warn!(domain, "alias_domain not found, skipping");
            continue;
        };

        let txn = c.transaction().await?;
        if let Err(e) = txn
            .execute(
                "SELECT pg_advisory_xact_lock($1)",
                &[&crate::quota::lock_id(
                    crate::quota::LOCK_CLASS_ALIAS_CAP,
                    user_id,
                )],
            )
            .await
        {
            tracing::warn!(error = %e, "advisory lock failed, skipping");
            continue;
        }
        let cap_row = users::cap_and_count_aliases()
            .bind(&txn, &crate::quota::DEFAULT_MAX_ALIASES, &user_id)
            .one()
            .await?;
        if cap_row.current >= cap_row.cap {
            txn.rollback().await.ok();
            skipped_cap += 1;
            tracing::warn!(user_email, "alias cap reached, skipping");
            continue;
        }
        let note_opt: Option<String> = if note.is_empty() { None } else { Some(note) };
        match aliases::create_with_flags()
            .bind(
                &txn,
                &user_id,
                &address,
                &domain_id,
                &mailbox_id,
                &enabled,
                &pinned,
                &note_opt,
            )
            .await
        {
            Ok(_) => match txn.commit().await {
                Ok(_) => imported += 1,
                Err(e) => tracing::warn!(address, error = %e, "commit failed, skipping"),
            },
            Err(e) => {
                txn.rollback().await.ok();
                tracing::warn!(address, error = %e, "insert failed, skipping");
            }
        }
    }
    println!("imported {imported} aliases");
    if skipped_cap > 0 {
        println!("skipped {skipped_cap} aliases due to per-user cap");
    }
    Ok(())
}

fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

/// RFC 4180-ish CSV parser: records separated by LF or CRLF; fields by
/// comma. A field may be quoted, in which case it can contain commas,
/// newlines, and `""` for a literal `"`. Round-trips with `csv_field`.
fn parse_csv(input: &str) -> Vec<Vec<String>> {
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(ch);
            }
        } else {
            match ch {
                '"' if field.is_empty() => in_quotes = true,
                ',' => {
                    record.push(std::mem::take(&mut field));
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                }
                '\n' => {
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                }
                _ => field.push(ch),
            }
        }
    }
    // Trailing field/record without a final newline.
    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    records
}

#[cfg(test)]
mod csv_tests {
    use super::{csv_field, parse_csv};

    #[test]
    fn roundtrip_plain() {
        let row = vec!["alice@example.com", "alias@foo.org", "note here"];
        let line: Vec<_> = row.iter().map(|s| csv_field(s)).collect();
        let serialized = line.join(",") + "\n";
        let parsed = parse_csv(&serialized);
        assert_eq!(
            parsed,
            vec![row.iter().map(|s| s.to_string()).collect::<Vec<_>>()]
        );
    }

    #[test]
    fn roundtrip_comma_in_note() {
        let row = vec!["u@x", "a@y", "hello, world"];
        let line: Vec<_> = row.iter().map(|s| csv_field(s)).collect();
        let parsed = parse_csv(&(line.join(",") + "\n"));
        assert_eq!(parsed[0][2], "hello, world");
    }

    #[test]
    fn roundtrip_quote_and_newline() {
        let row = vec!["u@x", "a@y", "she said \"hi\"\nthen left"];
        let line: Vec<_> = row.iter().map(|s| csv_field(s)).collect();
        let parsed = parse_csv(&(line.join(",") + "\n"));
        assert_eq!(parsed[0][2], "she said \"hi\"\nthen left");
    }

    #[test]
    fn multi_row() {
        let serialized = "a,b,c\nd,e,f\n";
        let parsed = parse_csv(serialized);
        assert_eq!(parsed, vec![vec!["a", "b", "c"], vec!["d", "e", "f"]]);
    }
}

pub async fn set_default_mailbox(url: &str, domain: String, mailbox_email: String) -> Result<()> {
    let c = db::connect_once(url).await?;
    let n = domains::set_default_mailbox_by_owner_email()
        .bind(&c, &mailbox_email, &domain)
        .await
        .map_err(|e| {
            if let Some(db) = e.as_db_error() {
                anyhow::anyhow!("{}", db.message())
            } else {
                anyhow::anyhow!(e)
            }
        })?;
    if n == 0 {
        bail!("no alias_domain row matched domain='{domain}'");
    }
    let is_null = domains::default_mailbox_is_null()
        .bind(&c, &domain)
        .one()
        .await?
        .unwrap_or(true);
    if is_null {
        bail!(
            "domain '{domain}' has no enabled+verified mailbox '{mailbox_email}' \
             belonging to its owner"
        );
    }
    tracing::info!(domain = %domain, mailbox = %mailbox_email, "default mailbox set");
    Ok(())
}

fn random_token(bytes: usize) -> String {
    use rand::TryRngCore;
    let mut buf = vec![0u8; bytes];
    rand::rngs::OsRng
        .try_fill_bytes(&mut buf)
        .expect("OsRng must not fail");
    BASE64URL_NOPAD.encode(&buf)
}

fn random_password() -> String {
    random_token(12)
}

#[derive(Debug, Default)]
pub struct GcStats {
    pub invite_token: u64,
    pub password_reset_token: u64,
    pub email_change_token: u64,
    pub mailbox_verify_token: u64,
    pub webauthn_ceremony: u64,
    pub session: u64,
    pub rate_limit_bucket: u64,
    pub email_log: u64,
}

impl GcStats {
    pub fn print(&self, dry_run: bool) {
        let label = if dry_run { "would-remove" } else { "removed" };
        println!("invite_token         {label}={}", self.invite_token);
        println!("password_reset_token {label}={}", self.password_reset_token);
        println!("email_change_token   {label}={}", self.email_change_token);
        println!("mailbox_verify_token {label}={}", self.mailbox_verify_token);
        println!("webauthn_ceremony    {label}={}", self.webauthn_ceremony);
        println!("session              {label}={}", self.session);
        println!("rate_limit_bucket    {label}={}", self.rate_limit_bucket);
        println!("email_log            {label}={}", self.email_log);
    }
}

pub async fn dev_seed(url: &str) -> Result<()> {
    let localish = url.contains("localhost")
        || url.contains("127.0.0.1")
        || url.contains("/tmp")
        || url.contains("::1");
    if !localish && std::env::var("RAMPART_DEV_SEED_ALLOW").is_err() {
        bail!(
            "dev_seed refused: database URL does not appear to be local.\n\
             Set RAMPART_DEV_SEED_ALLOW=1 to override."
        );
    }
    if !localish {
        tracing::warn!("RAMPART_DEV_SEED_ALLOW set, proceeding against non-local database");
    }

    let c = db::connect_once(url).await?;

    if users::by_email_id_unfiltered()
        .bind(&c, &"dev@localhost")
        .opt()
        .await?
        .is_some()
    {
        println!("dev seed already applied, nothing to do");
        return Ok(());
    }

    let password_hash = hash_password("devpassword")?;
    let user_id = users::create()
        .bind(
            &c,
            &"dev@localhost".to_string(),
            &Some(password_hash),
            &Some("Developer".to_string()),
            &true,
        )
        .one()
        .await
        .context("creating dev user")?;
    println!("  user:  dev@localhost (admin)");

    let domain_id = domains::create()
        .bind(
            &c,
            &"dev.local".to_string(),
            &Some(user_id),
            &None::<String>,
        )
        .one()
        .await
        .context("creating dev domain")?;
    println!("  domain: dev.local");

    let mailbox_id = mailboxes::create_verified()
        .bind(&c, &user_id, &"dev@dev.local".to_string(), &None::<String>)
        .one()
        .await
        .context("creating dev mailbox")?;

    let aliases_data: &[(&str, bool, bool, Option<&str>)] = &[
        (
            "github@dev.local",
            true,
            false,
            Some("GitHub notifications"),
        ),
        ("shopping@dev.local", true, false, Some("Online shopping")),
        (
            "newsletter@dev.local",
            true,
            true,
            Some("Monthly newsletter"),
        ),
        ("social@dev.local", true, false, None),
        ("work@dev.local", true, false, Some("Work-related emails")),
        (
            "spamcatcher@dev.local",
            false,
            false,
            Some("Caught too much spam"),
        ),
    ];

    for (address, enabled, pinned, note) in aliases_data {
        let note_opt: Option<String> = note.map(|s| s.to_string());
        aliases::create_with_flags()
            .bind(
                &c,
                &user_id,
                &address.to_string(),
                &domain_id,
                &mailbox_id,
                enabled,
                pinned,
                &note_opt,
            )
            .await
            .with_context(|| format!("creating alias {address}"))?;
    }
    println!(
        "  aliases: {} (mixed enabled/disabled, pinned)",
        aliases_data.len()
    );

    println!();
    println!("  Login:   dev@localhost / devpassword");
    println!("  URL:     http://localhost:8090");

    Ok(())
}

pub const DEFAULT_EMAIL_LOG_DAYS: i32 = 90;

pub async fn gc(url: &str, email_log_days: i32, dry_run: bool) -> Result<GcStats> {
    if email_log_days < 0 {
        bail!("email_log_days must be >= 0");
    }
    let mut c = db::connect_once(url).await?;
    let mut stats = GcStats::default();

    let txn = c.transaction().await?;
    if dry_run {
        stats.invite_token = gc_q::count_invite_token_stale().bind(&txn).one().await? as u64;
        stats.password_reset_token = gc_q::count_password_reset_token_stale()
            .bind(&txn)
            .one()
            .await? as u64;
        stats.email_change_token = gc_q::count_email_change_token_stale()
            .bind(&txn)
            .one()
            .await? as u64;
        stats.mailbox_verify_token = gc_q::count_mailbox_verify_token_stale()
            .bind(&txn)
            .one()
            .await? as u64;
        stats.webauthn_ceremony = gc_q::count_webauthn_ceremony_stale()
            .bind(&txn)
            .one()
            .await? as u64;
        stats.session = gc_q::count_session_stale().bind(&txn).one().await? as u64;
        stats.rate_limit_bucket = gc_q::count_rate_limit_bucket_stale()
            .bind(&txn)
            .one()
            .await? as u64;
        txn.rollback().await?;
    } else {
        stats.invite_token = gc_q::delete_invite_token_stale().bind(&txn).await?;
        stats.password_reset_token = gc_q::delete_password_reset_token_stale().bind(&txn).await?;
        stats.email_change_token = gc_q::delete_email_change_token_stale().bind(&txn).await?;
        stats.mailbox_verify_token = gc_q::delete_mailbox_verify_token_stale().bind(&txn).await?;
        stats.webauthn_ceremony = gc_q::delete_webauthn_ceremony_stale().bind(&txn).await?;
        stats.session = gc_q::delete_session_stale().bind(&txn).await?;
        stats.rate_limit_bucket = gc_q::delete_rate_limit_bucket_stale().bind(&txn).await?;
        txn.commit().await?;
    }

    let txn2 = c.transaction().await?;
    stats.email_log = if dry_run {
        let n = gc_q::count_email_log_old()
            .bind(&txn2, &email_log_days)
            .one()
            .await?;
        txn2.rollback().await?;
        n as u64
    } else {
        let n = gc_q::delete_email_log_old()
            .bind(&txn2, &email_log_days)
            .await?;
        txn2.commit().await?;
        n
    };

    let total = stats.invite_token
        + stats.password_reset_token
        + stats.email_change_token
        + stats.mailbox_verify_token
        + stats.webauthn_ceremony
        + stats.session
        + stats.rate_limit_bucket
        + stats.email_log;
    tracing::info!(total, dry_run, "gc complete");
    Ok(stats)
}
