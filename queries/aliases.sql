-- Alias CRUD + the joined view rows.
--: AliasJoinedRow(note?, last_email_at?)

--! list_for_user_filtered (query?, pinned?) : AliasJoinedRow
SELECT a.id, a.address::text AS address, a.enabled, a.note, a.pinned,
       a.nb_forward, a.nb_block, a.nb_reply, a.mailbox_id,
       m.email::text AS mailbox_email, d.domain::text AS domain,
       a.created_at, a.last_email_at
FROM alias a
JOIN mailbox m ON m.id = a.mailbox_id
JOIN alias_domain d ON d.id = a.domain_id
WHERE a.user_id = :user_id
  AND (:query::text IS NULL OR a.address::text ILIKE :query OR a.note ILIKE :query)
  AND (:pinned::bool IS NULL OR a.pinned = :pinned)
ORDER BY a.pinned DESC, a.last_email_at DESC NULLS LAST, a.id DESC
LIMIT :lim OFFSET :off;

--! by_id_user : AliasJoinedRow
SELECT a.id, a.address::text AS address, a.enabled, a.note, a.pinned,
       a.nb_forward, a.nb_block, a.nb_reply, a.mailbox_id,
       m.email::text AS mailbox_email, d.domain::text AS domain,
       a.created_at, a.last_email_at
FROM alias a
JOIN mailbox m ON m.id = a.mailbox_id
JOIN alias_domain d ON d.id = a.domain_id
WHERE a.id = :alias_id AND a.user_id = :user_id;

--! by_id : AliasJoinedRow
SELECT a.id, a.address::text AS address, a.enabled, a.note, a.pinned,
       a.nb_forward, a.nb_block, a.nb_reply, a.mailbox_id,
       m.email::text AS mailbox_email, d.domain::text AS domain,
       a.created_at, a.last_email_at
FROM alias a
JOIN mailbox m ON m.id = a.mailbox_id
JOIN alias_domain d ON d.id = a.domain_id
WHERE a.id = :alias_id;

--! list_for_dashboard : (note?, last_email_at?)
SELECT a.id, a.address::text AS address, a.enabled, a.note, a.pinned,
       a.nb_forward, a.nb_block, a.nb_reply, a.mailbox_id,
       m.email::text AS mailbox_email, d.domain::text AS domain,
       a.last_email_at
FROM alias a
JOIN mailbox m ON m.id = a.mailbox_id
JOIN alias_domain d ON d.id = a.domain_id
WHERE a.user_id = :user_id
ORDER BY a.pinned DESC, a.last_email_at DESC NULLS LAST, a.id DESC
LIMIT 200;

--! address_for_user
SELECT a.address::text AS address
FROM alias a
WHERE a.id = :alias_id AND a.user_id = :user_id;

--! exists_for_user
SELECT 1 AS one FROM alias WHERE id = :alias_id AND user_id = :user_id;

--! set_note (note?)
UPDATE alias SET note = :note WHERE id = :alias_id AND user_id = :user_id;

--! set_pinned
UPDATE alias SET pinned = :pinned WHERE id = :alias_id AND user_id = :user_id;

--! set_mailbox
UPDATE alias SET mailbox_id = :mailbox_id WHERE id = :alias_id AND user_id = :user_id;

--! toggle_enabled
UPDATE alias SET enabled = NOT enabled WHERE id = :alias_id AND user_id = :user_id;

--! delete
DELETE FROM alias WHERE id = :alias_id AND user_id = :user_id;

--! disable_all_for_user
UPDATE alias SET enabled = FALSE WHERE user_id = :user_id;

--! create (note?)
INSERT INTO alias (user_id, address, domain_id, mailbox_id, note, auto_created)
VALUES (:user_id, :address, :domain_id, :mailbox_id, :note, :auto_created)
RETURNING id;

--! create_with_flags (note?)
INSERT INTO alias (user_id, address, domain_id, mailbox_id, enabled, pinned, note)
VALUES (:user_id, :address, :domain_id, :mailbox_id, :enabled, :pinned, :note);

--! bump_forward_count
UPDATE alias SET nb_forward = nb_forward + 1, last_email_at = now() WHERE id = :alias_id;

--! bump_block_count
UPDATE alias SET nb_block = nb_block + 1, last_email_at = now() WHERE id = :alias_id;

--: AliasExportRow(note?)

--! export : AliasExportRow
SELECT a.address::text AS address, a.note, a.enabled, a.pinned,
       m.email::text AS mailbox, u.email::text AS user_email
FROM alias a
JOIN mailbox m ON m.id = a.mailbox_id
JOIN "user" u ON u.id = a.user_id
ORDER BY u.id, a.id;

--! export_for_user : AliasExportRow
SELECT a.address::text AS address, a.note, a.enabled, a.pinned,
       m.email::text AS mailbox, u.email::text AS user_email
FROM alias a
JOIN mailbox m ON m.id = a.mailbox_id
JOIN "user" u ON u.id = a.user_id
WHERE u.email = :user_email
ORDER BY a.id;

--! forward_join
SELECT a.address::text AS alias_address, a.enabled AS alias_enabled,
       m.email::text AS mailbox_email, m.enabled AS mailbox_enabled,
       u.enabled AS user_enabled,
       d.domain::text AS alias_domain, a.user_id
FROM alias a
JOIN mailbox m ON m.id = a.mailbox_id
JOIN "user" u ON u.id = a.user_id
JOIN alias_domain d ON d.id = a.domain_id
WHERE a.id = :alias_id;
