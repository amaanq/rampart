-- Scheduled garbage collection. Counts return non-null COALESCE-d totals;
-- DELETEs return rowcount which the caller reads via .execute()-style.

--! count_invite_token_stale : (n)
SELECT count(*)::bigint AS n FROM invite_token
WHERE used_at IS NOT NULL OR expires_at < now();

--! delete_invite_token_stale
DELETE FROM invite_token
WHERE used_at IS NOT NULL OR expires_at < now();

--! count_password_reset_token_stale : (n)
SELECT count(*)::bigint AS n FROM password_reset_token
WHERE used_at IS NOT NULL OR expires_at < now();

--! delete_password_reset_token_stale
DELETE FROM password_reset_token
WHERE used_at IS NOT NULL OR expires_at < now();

--! count_email_change_token_stale : (n)
SELECT count(*)::bigint AS n FROM email_change_token
WHERE used_at IS NOT NULL OR expires_at < now();

--! delete_email_change_token_stale
DELETE FROM email_change_token
WHERE used_at IS NOT NULL OR expires_at < now();

--! count_mailbox_verify_token_stale : (n)
SELECT count(*)::bigint AS n FROM mailbox_verify_token
WHERE used_at IS NOT NULL OR expires_at < now();

--! delete_mailbox_verify_token_stale
DELETE FROM mailbox_verify_token
WHERE used_at IS NOT NULL OR expires_at < now();

--! count_webauthn_ceremony_stale : (n)
SELECT count(*)::bigint AS n FROM webauthn_ceremony WHERE expires_at < now();

--! delete_webauthn_ceremony_stale
DELETE FROM webauthn_ceremony WHERE expires_at < now();

--! count_session_stale : (n)
SELECT count(*)::bigint AS n FROM session WHERE expires_at < now();

--! delete_session_stale
DELETE FROM session WHERE expires_at < now();

--! count_rate_limit_bucket_stale : (n)
SELECT count(*)::bigint AS n FROM rate_limit_bucket
WHERE window_start < now() - interval '1 day';

--! delete_rate_limit_bucket_stale
DELETE FROM rate_limit_bucket
WHERE window_start < now() - interval '1 day';

--! count_email_log_old : (n)
SELECT count(*)::bigint AS n FROM email_log
WHERE created_at < now() - make_interval(days => :days::int);

--! delete_email_log_old
DELETE FROM email_log
WHERE created_at < now() - make_interval(days => :days::int);
