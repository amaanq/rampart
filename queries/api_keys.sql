-- Bearer api_key lookup + best-effort last_used bump. The cascade
-- revoke (admin disable user) lives here too because it's an api_key
-- mutation in spirit.

--: ApiKeyRow(last_used_at?, revoked_at?, expires_at?, token_prefix?)

--! lookup_with_user
SELECT k.id AS api_key_id, k.user_id, u.is_admin, k.scopes
FROM api_key k
JOIN "user" u ON u.id = k.user_id
WHERE k.key_hash = :key_hash
  AND k.revoked_at IS NULL
  AND (k.expires_at IS NULL OR k.expires_at > now())
  AND u.enabled;

--! bump_last_used
UPDATE api_key SET last_used_at = now() WHERE key_hash = :key_hash;

--! revoke_all_for_user
UPDATE api_key SET revoked_at = now()
WHERE user_id = :user_id AND revoked_at IS NULL;

--! list_for_user : ApiKeyRow
SELECT id, name, scopes, kind, token_prefix, last_used_at, revoked_at,
       created_at, expires_at
FROM api_key
WHERE user_id = :user_id
ORDER BY revoked_at NULLS FIRST, id DESC;

--! create_extension (expires_at?, token_prefix?) : ApiKeyRow
INSERT INTO api_key (user_id, name, key_hash, scopes, kind, token_prefix, expires_at)
VALUES (:user_id, :name, :key_hash, :scopes, 'extension', :token_prefix, :expires_at)
RETURNING id, name, scopes, kind, token_prefix, last_used_at, revoked_at,
          created_at, expires_at;

--! revoke_for_user
UPDATE api_key SET revoked_at = now()
WHERE id = :api_key_id AND user_id = :user_id AND revoked_at IS NULL;

--! revoke_self
UPDATE api_key SET revoked_at = now()
WHERE id = :api_key_id AND user_id = :user_id AND revoked_at IS NULL;
