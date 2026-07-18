//! Per-message dispatch: forward vs reply, header rewrite, upsert of
//! reverse_contact, outbound submission. Everything happens here after
//! the LMTP layer has validated the envelope and assembled the body.

use anyhow::{Context, Result, bail};
use data_encoding::BASE64URL_NOPAD;
use rampart_codegen::queries::{aliases, contacts, email_log};
use rand::TryRngCore;

use crate::worker::WorkerState;
use crate::worker::auth_results::{extract_for, reply_policy_ok};
use crate::worker::loop_guard::Rcpt;
use crate::worker::resubmit::rewrite_headers;

/// The one RCPT we accepted, plus the raw message bytes stalwart delivered.
pub struct Delivery {
    pub rcpt: Rcpt,
    pub mail_from: String,
    pub raw: Vec<u8>,
}

#[derive(Debug)]
pub enum Verdict {
    Delivered,
    /// 5xx permanent rejection. `internal` is for tracing/logs only;
    /// `smtp` is what we send back to the peer. Anything that could
    /// reveal mailbox addresses, alias->mailbox bindings, or reverse-
    /// contact metadata MUST stay in `internal` — `smtp` is read by
    /// anyone who can talk to the LMTP listener.
    Perm {
        internal: String,
        smtp: &'static str,
    },
    /// 4xx temporary rejection. Same split as Perm.
    Temp {
        internal: String,
        smtp: &'static str,
    },
}

pub async fn process(state: &WorkerState, d: Delivery) -> Verdict {
    match do_process(state, d).await {
        Ok(()) => Verdict::Delivered,
        Err(e) => {
            tracing::error!(error = ?e, "pipeline error");
            let internal = e.to_string();
            if internal.contains("alias disabled") || internal.contains("alias not found") {
                Verdict::Perm {
                    internal,
                    smtp: "no such alias",
                }
            } else if internal.contains("reverse_contact not found") {
                Verdict::Perm {
                    internal,
                    smtp: "no such reverse alias",
                }
            } else if internal.contains("reply-policy") || internal.contains("block_reply") {
                Verdict::Perm {
                    internal,
                    smtp: "reply rejected",
                }
            } else {
                Verdict::Temp {
                    internal,
                    smtp: "transient processing error, try later",
                }
            }
        }
    }
}

async fn do_process(state: &WorkerState, d: Delivery) -> Result<()> {
    match d.rcpt {
        Rcpt::Forward(alias_id) => handle_forward(state, alias_id, &d.mail_from, &d.raw).await,
        Rcpt::Reply(rc_id) => handle_reply(state, rc_id, &d.mail_from, &d.raw).await,
        Rcpt::Bounce { payload } => handle_bounce(state, &payload, &d.mail_from, &d.raw).await,
    }
}

async fn handle_forward(
    state: &WorkerState,
    alias_id: i64,
    mail_from: &str,
    raw: &[u8],
) -> Result<()> {
    // Reverse-contact key is visible From, not envelope MAIL FROM —
    // mailing lists / SRS / VERP set MAIL FROM unrelated to the author
    // the user wants to reply to. Envelope is the fallback.
    let (from_address, from_display) = extract_from(raw);
    let real_email_key = from_address.unwrap_or_else(|| mail_from.to_owned());

    // Scope the DB client so it drops before the SMTP submit; otherwise
    // the pool connection is held across network I/O.
    let (alias_address, mailbox_email, alias_domain, user_id, token, email_log_id) = {
        let c = state.pool.get().await?;
        let Some(r) = aliases::forward_join().bind(&c, &alias_id).opt().await? else {
            bail!("alias not found ({alias_id})");
        };
        if !r.alias_enabled || !r.mailbox_enabled || !r.user_enabled {
            bail!("alias disabled");
        }
        let alias_address = r.alias_address;
        let mailbox_email = r.mailbox_email;
        let alias_domain = r.alias_domain;
        let user_id = r.user_id;
        let (token, contact_enabled) =
            upsert_reverse_contact(&c, alias_id, &real_email_key, &alias_domain).await?;
        if !contact_enabled {
            // Accept-and-drop, not 5xx — a 550 backscatters at every
            // upstream retry and confirms the alias exists.
            if let Err(e) = email_log::insert_block()
                .bind(&c, &alias_id, &Some(real_email_key.clone()))
                .await
            {
                tracing::error!(error = ?e, alias_id, "block log failed (ignored)");
            }
            if let Err(e) = aliases::bump_block_count().bind(&c, &alias_id).await {
                tracing::error!(error = ?e, alias_id, "block counter update failed (ignored)");
            }
            tracing::info!(
                alias_id,
                envelope_from = mail_from,
                visible_from = real_email_key,
                "forward dropped: contact disabled (accept-and-drop)"
            );
            return Ok(());
        }
        let email_log_id = email_log::insert_forward()
            .bind(&c, &alias_id, &Some(real_email_key.clone()))
            .one()
            .await
            .context("pre-submit email_log insert")?;
        (
            alias_address,
            mailbox_email,
            alias_domain,
            user_id,
            token,
            email_log_id,
        )
    };

    // `ra+` MUST match alias_domain.reply_prefix and the rendered
    // Sieve glob, or replies bounce with "Unknown reply handle".
    let display = match from_display.as_deref() {
        Some(name) if name != real_email_key => format!("{name} - {real_email_key}"),
        _ => real_email_key.clone(),
    };
    let new_from = format!(
        "{} <ra+{}@{}>",
        crate::worker::resubmit::rfc5322_quoted_display(&display),
        token,
        alias_domain
    );
    // To: rewrites to alias_address so Reply All can't leak to the
    // stripped recipients.
    let new_raw = rewrite_headers(
        raw,
        &new_from,
        &alias_address,
        &state.config.stalwart_hostname,
    );

    submit_and_track_status(
        state,
        crate::worker::loop_guard::BouncePhase::Forward,
        email_log_id,
        &alias_domain,
        &mailbox_email,
        &new_raw,
        "forwarding outbound submit",
    )
    .await?;

    // Post-submit bookkeeping is best-effort: returning Err here would
    // make stalwart retry and duplicate the mail.
    match state.pool.get().await {
        Ok(c) => {
            if let Err(e) = aliases::bump_forward_count().bind(&c, &alias_id).await {
                tracing::error!(error = ?e, alias_id, "counter update failed post-submit (ignored)");
            }
        }
        Err(e) => {
            tracing::error!(error = ?e, alias_id, "pool unavailable post-submit (ignored, mail delivered)");
        }
    }
    tracing::info!(
        user_id,
        alias_id,
        alias_address,
        mailbox_email,
        envelope_from = mail_from,
        visible_from = real_email_key,
        email_log_id,
        "forwarded"
    );
    Ok(())
}

async fn upsert_reverse_contact(
    c: &deadpool_postgres::Client,
    alias_id: i64,
    real_email: &str,
    alias_domain: &str,
) -> Result<(String, bool)> {
    let fresh_token = random_token();
    let fresh_reply_addr = format!("ra+{}@{}", fresh_token, alias_domain);
    let r = contacts::upsert_for_worker()
        .bind(c, &alias_id, &real_email, &fresh_token, &fresh_reply_addr)
        .one()
        .await?;
    Ok((r.token, r.enabled))
}

/// Submit with an HMAC-signed bounce VERP as MAIL FROM, then flip
/// the pre-INSERT email_log row to submitted/failed. Propagates the
/// submit error after the best-effort status flip.
async fn submit_and_track_status(
    state: &WorkerState,
    phase: crate::worker::loop_guard::BouncePhase,
    email_log_id: i64,
    alias_domain: &str,
    rcpt_to: &str,
    raw: &[u8],
    err_context: &'static str,
) -> Result<()> {
    let bounce_payload =
        crate::worker::verp::make_local_payload(&state.config.verp_key, phase, email_log_id);
    let bounce_from = format!("bnc+{bounce_payload}@{alias_domain}");
    let submit_result = state
        .submit
        .submit(&bounce_from, rcpt_to, raw)
        .await
        .context(err_context);
    if let Err(e) = &submit_result {
        if let Ok(c) = state.pool.get().await {
            let _ = email_log::flip_failed()
                .bind(&c, &Some(e.to_string()), &email_log_id)
                .await;
        }
    }
    submit_result?;
    if let Ok(c) = state.pool.get().await {
        if let Err(e) = email_log::flip_submitted().bind(&c, &email_log_id).await {
            tracing::error!(error = ?e, email_log_id, "status flip pending→submitted failed");
        }
    }
    Ok(())
}

fn random_token() -> String {
    let mut bytes = [0u8; 10];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .expect("OsRng must not fail");
    BASE64URL_NOPAD.encode(&bytes)
}

/// Extract `(address, display_name)` from the visible From: header.
/// Both halves are independently optional. Parsed via mail-parser so
/// folded, encoded-word, group-syntax, and commented headers all work.
fn extract_from(raw: &[u8]) -> (Option<String>, Option<String>) {
    use mail_parser::{HeaderValue, MessageParser};
    let Some(msg) = MessageParser::new().parse(raw) else {
        return (None, None);
    };
    let Some(from) = msg.header("From").or_else(|| msg.header("from")) else {
        return (None, None);
    };
    match from {
        HeaderValue::Address(addr) => {
            let addrs = addr.clone().into_list();
            for a in addrs {
                let address = a.address().map(|s| s.to_owned());
                let name = a.name().map(|s| s.to_owned());
                if address.is_some() || name.is_some() {
                    return (address, name);
                }
            }
            (None, None)
        }
        _ => (None, None),
    }
}

async fn handle_reply(state: &WorkerState, rc_id: i64, mail_from: &str, raw: &[u8]) -> Result<()> {
    let (real_email, alias_address, alias_id, alias_domain, mailbox_email) = {
        let c = state.pool.get().await?;
        let Some(r) = contacts::reply_join().bind(&c, &rc_id).opt().await? else {
            bail!("reverse_contact not found ({rc_id})");
        };
        // Reply addresses outlive later account, alias, and mailbox disables.
        if !r.rc_enabled
            || r.block_reply
            || !r.alias_enabled
            || !r.mailbox_enabled
            || !r.user_enabled
        {
            bail!("block_reply or disabled");
        }
        (
            r.real_email,
            r.alias_address,
            r.alias_id,
            r.alias_domain,
            r.mailbox_email,
        )
    };

    let visible_from = extract_from_address(raw)
        .ok_or_else(|| anyhow::anyhow!("reply-policy: no visible From"))?;

    // Two acceptance paths, gated on whether stalwart wrote an AR header.
    //
    //  External (MX, port 25): stalwart adds AR only on local_port==25.
    //    Require dmarc=pass + visible-From-domain alignment + visible-From
    //    == mailbox exact match.
    //
    //  Local submission (465/587): no AR (stalwart can't DKIM-verify its
    //    own outbound). Trust the mustMatchSender + auth invariants:
    //    MAIL FROM == visible From == mailbox.
    //
    // External spoofers can't slip into the local-submission branch —
    // their mail enters via port 25 which always carries an AR.
    let ar_lines = extract_authentication_results(raw);
    if let Some(ar) = extract_for(&ar_lines, &state.config.stalwart_hostname) {
        reply_policy_ok(&ar, &mailbox_email, &visible_from)
            .map_err(|e| anyhow::anyhow!("reply-policy: {e}"))?;
    } else {
        if !mail_from.eq_ignore_ascii_case(&mailbox_email)
            || !visible_from.eq_ignore_ascii_case(&mailbox_email)
        {
            bail!(
                "reply-policy: local-submission requires mail_from={mail_from} == \
                 visible_from={visible_from} == mailbox={mailbox_email}"
            );
        }
        tracing::info!(
            mailbox_email,
            mail_from,
            "reply via trusted local submission"
        );
    }

    // From: → alias, To: → real recipient so MUA Reply All targets
    // the real recipient instead of the reverse alias.
    let new_raw = rewrite_headers(
        raw,
        &alias_address,
        &real_email,
        &state.config.stalwart_hostname,
    );

    // Pre-INSERT so the bounce VERP has an id. Tempfail on DB failure
    // is fine; duplicate delivery is not.
    let email_log_id: i64 = {
        let c = state.pool.get().await?;
        c.query_one(
            "INSERT INTO email_log (alias_id, reverse_contact_id, action, from_address) \
             VALUES ($1, $2, 'reply', $3) RETURNING id",
            &[&alias_id, &rc_id, &mail_from],
        )
        .await
        .context("pre-submit email_log insert (reply)")?
        .get("id")
    };

    submit_and_track_status(
        state,
        crate::worker::loop_guard::BouncePhase::Reply,
        email_log_id,
        &alias_domain,
        &real_email,
        &new_raw,
        "reply outbound submit",
    )
    .await?;

    // Best-effort: returning Err here would retry the LMTP inbound
    // and duplicate the outbound.
    match state.pool.get().await {
        Ok(c) => {
            if let Err(e) = c
                .execute(
                    "UPDATE alias SET nb_reply = nb_reply + 1 WHERE address = $1",
                    &[&alias_address],
                )
                .await
            {
                tracing::error!(error = ?e, "reply counter update failed post-submit (ignored)");
            }
            if let Err(e) = c
                .execute(
                    "UPDATE reverse_contact SET last_seen_at = now() WHERE id = $1",
                    &[&rc_id],
                )
                .await
            {
                tracing::error!(error = ?e, "reverse_contact last_seen update failed (ignored)");
            }
        }
        Err(e) => {
            tracing::error!(error = ?e, rc_id, "pool unavailable post-submit (ignored, mail delivered)");
        }
    }
    tracing::info!(
        alias_address,
        real_email,
        from = mail_from,
        email_log_id,
        "reply delivered"
    );
    Ok(())
}

/// DSN delivered to an HMAC-signed bounce VERP. Verify, flip
/// `email_log.status` to 'bounced', do NOT re-forward (that loops if
/// the mailbox itself bounces). Returns Ok so we 250 either way —
/// bouncing-the-bounce is worse than swallowing.
async fn handle_bounce(
    state: &WorkerState,
    payload: &str,
    mail_from: &str,
    raw: &[u8],
) -> Result<()> {
    let Some((phase, email_log_id)) =
        crate::worker::verp::verify_payload(&state.config.verp_key, payload)
    else {
        // Forged VERP — anyone can RCPT TO a guessed bnc+; HMAC is
        // what authenticates the mutation path.
        tracing::warn!(
            envelope_from = mail_from,
            payload,
            "bounce VERP HMAC verification failed; not mutating any row"
        );
        return Ok(());
    };
    let reason = extract_dsn_reason(raw);
    let c = state.pool.get().await?;

    // bnc+f → action='forward', bnc+r → 'reply'. The status filter
    // makes this idempotent.
    let want_action = match phase {
        crate::worker::loop_guard::BouncePhase::Forward => "forward",
        crate::worker::loop_guard::BouncePhase::Reply => "reply",
    };
    // Transient DB error tempfails (stalwart retries); swallowing
    // would lose a valid DSN. 0 rows = already-bounced / deleted /
    // phase mismatch — log and move on.
    let rows = c
        .execute(
            "UPDATE email_log SET status = 'bounced', \
                                  reason = COALESCE($2, reason) \
             WHERE id = $1 \
               AND action = $3 \
               AND status IN ('pending','submitted')",
            &[&email_log_id, &reason.as_deref(), &want_action],
        )
        .await
        .context("bounce email_log UPDATE")?;
    if rows == 0 {
        tracing::warn!(
            email_log_id,
            ?phase,
            envelope_from = mail_from,
            "bounce VERP verified but no eligible row (already bounced, deleted, or phase mismatch)"
        );
    } else {
        tracing::warn!(
            email_log_id,
            ?phase,
            envelope_from = mail_from,
            reason = reason.as_deref().unwrap_or("<no DSN body>"),
            "DSN bounce received; not forwarded"
        );
    }
    Ok(())
}

/// Best-effort short reason from an RFC 3464 DSN body. Prefers
/// `Diagnostic-Code:` over the first non-empty line; capped at 500
/// bytes. For tracing only — sanitize before showing to users.
fn extract_dsn_reason(raw: &[u8]) -> Option<String> {
    use mail_parser::MessageParser;
    let msg = MessageParser::new().parse(raw)?;
    for part in msg.parts.iter() {
        // Skip non-text parts without short-circuiting — later text/*
        // parts may still match.
        let Some(text) = part.text_contents() else {
            continue;
        };
        for line in text.lines() {
            let lower = line.to_ascii_lowercase();
            if let Some(rest) = lower.strip_prefix("diagnostic-code:") {
                let _ = rest; // we want the original-case content
                let v = line[16..].trim();
                if !v.is_empty() {
                    return Some(truncate(v, 500));
                }
            }
        }
    }
    // Fallback: first non-empty trimmed line of body text.
    for part in msg.parts.iter() {
        if let Some(text) = part.text_contents() {
            for line in text.lines() {
                let t = line.trim();
                if !t.is_empty() {
                    return Some(truncate(t, 500));
                }
            }
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    // Reserve room for the ellipsis before slicing so the final string
    // fits inside `max` bytes.
    const ELLIPSIS: &str = "…";
    let budget = max.saturating_sub(ELLIPSIS.len());
    let mut cut = budget.min(s.len());
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    let mut out = String::with_capacity(cut + ELLIPSIS.len());
    out.push_str(&s[..cut]);
    out.push_str(ELLIPSIS);
    out
}

fn extract_authentication_results(raw: &[u8]) -> Vec<String> {
    let hdr_end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap_or(raw.len());
    let headers = std::str::from_utf8(&raw[..hdr_end]).unwrap_or("");
    let mut lines = Vec::new();
    let mut current: Option<String> = None;
    for line in headers.split("\r\n") {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(c) = current.as_mut() {
                c.push(' ');
                c.push_str(line.trim());
            }
            continue;
        }
        if let Some(prev) = current.take() {
            lines.push(prev);
        }
        if let Some(rest) = line
            .strip_prefix("Authentication-Results:")
            .or_else(|| line.strip_prefix("authentication-results:"))
        {
            current = Some(rest.trim().to_owned());
        }
    }
    if let Some(prev) = current {
        lines.push(prev);
    }
    lines
}

fn extract_from_address(raw: &[u8]) -> Option<String> {
    extract_from(raw).0
}
