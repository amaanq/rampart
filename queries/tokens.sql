-- One-shot token tables.

--! invite_create (preset_email?)
INSERT INTO invite_token (token_hash, preset_email, expires_at)
VALUES (:token_hash, :preset_email, :expires_at);

--! invite_claim
UPDATE invite_token SET used_at = now()
WHERE token_hash = :token_hash
  AND used_at IS NULL
  AND expires_at > now()
  AND (preset_email IS NULL OR preset_email = :email::CITEXT)
RETURNING token_hash;

--! invite_set_used_by
UPDATE invite_token SET used_by = :user_id WHERE token_hash = :token_hash;

--! password_reset_create
INSERT INTO password_reset_token (token_hash, user_id, expires_at)
VALUES (:token_hash, :user_id, :expires_at);

--! password_reset_claim
UPDATE password_reset_token SET used_at = now()
WHERE token_hash = :token_hash AND used_at IS NULL AND expires_at > now()
RETURNING user_id;

--! email_change_create
INSERT INTO email_change_token (token_hash, user_id, new_email, expires_at)
VALUES (:token_hash, :user_id, :new_email, :expires_at);

--! email_change_claim
UPDATE email_change_token SET used_at = now()
WHERE token_hash = :token_hash AND used_at IS NULL AND expires_at > now()
RETURNING user_id, new_email::text AS new_email;

--! mailbox_verify_create
INSERT INTO mailbox_verify_token (token_hash, mailbox_id, expires_at)
VALUES (:token_hash, :mailbox_id, :expires_at);

--! mailbox_verify_claim
UPDATE mailbox_verify_token SET used_at = now()
WHERE token_hash = :token_hash AND used_at IS NULL AND expires_at > now()
RETURNING mailbox_id;
