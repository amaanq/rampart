-- Cookie-session lifecycle.

--! lookup_with_user
SELECT s.user_id, s.expires_at, u.is_admin, u.enabled
FROM session s
JOIN "user" u ON u.id = s.user_id
WHERE s.id = :session_id;

--! delete_by_id
DELETE FROM session WHERE id = :session_id;

--! delete_by_user
DELETE FROM session WHERE user_id = :user_id;

--! bump_last_seen
UPDATE session SET last_seen_at = now(), expires_at = :expires_at WHERE id = :session_id;

--! create (user_agent?)
INSERT INTO session (id, user_id, expires_at, user_agent)
VALUES (:session_id, :user_id, :expires_at, :user_agent);
