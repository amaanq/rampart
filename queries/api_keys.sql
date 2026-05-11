-- Bearer api_key lookup + best-effort last_used bump. The cascade
-- revoke (admin disable user) lives here too because it's an api_key
-- mutation in spirit.

--! lookup_with_user
SELECT k.user_id, u.is_admin
FROM api_key k
JOIN "user" u ON u.id = k.user_id
WHERE k.key_hash = :key_hash AND k.revoked_at IS NULL AND u.enabled;

--! bump_last_used
UPDATE api_key SET last_used_at = now() WHERE key_hash = :key_hash;

--! revoke_all_for_user
UPDATE api_key SET revoked_at = now()
WHERE user_id = :user_id AND revoked_at IS NULL;
