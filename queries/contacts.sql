-- reverse_contact CRUD + the worker upsert.

--! list_for_alias : (display_name?, last_seen_at?)
SELECT id, real_email::text AS real_email, reply_address::text AS reply_address,
       display_name, enabled, block_reply, last_seen_at, created_at
FROM reverse_contact
WHERE alias_id = :alias_id
ORDER BY id DESC;

--! exists_for_user
SELECT 1 AS one
FROM reverse_contact rc JOIN alias a ON a.id = rc.alias_id
WHERE rc.id = :contact_id AND a.user_id = :user_id;

--! set_enabled
UPDATE reverse_contact SET enabled = :enabled WHERE id = :contact_id;

--! set_block_reply
UPDATE reverse_contact SET block_reply = :block_reply WHERE id = :contact_id;

--! set_display_name (display_name?)
UPDATE reverse_contact SET display_name = :display_name WHERE id = :contact_id;

--! delete_for_user
DELETE FROM reverse_contact rc USING alias a
WHERE rc.id = :contact_id AND rc.alias_id = a.id AND a.user_id = :user_id;

--! upsert_for_worker
WITH ins AS (
    INSERT INTO reverse_contact (alias_id, real_email, token, reply_address)
    VALUES (:alias_id, :real_email, :token, :reply_address)
    ON CONFLICT (alias_id, real_email) DO UPDATE SET last_seen_at = now()
    RETURNING token, enabled
)
SELECT token, enabled FROM ins;

--! reply_join
SELECT rc.real_email::text AS real_email, rc.enabled AS rc_enabled,
       rc.block_reply,
       a.id AS alias_id, a.address::text AS alias_address,
       a.enabled AS alias_enabled,
       d.domain::text AS alias_domain,
       m.email::text AS mailbox_email, m.enabled AS mailbox_enabled,
       u.enabled AS user_enabled
FROM reverse_contact rc
JOIN alias a ON a.id = rc.alias_id
JOIN alias_domain d ON d.id = a.domain_id
JOIN mailbox m ON m.id = a.mailbox_id
JOIN "user" u ON u.id = a.user_id
WHERE rc.id = :rc_id;
