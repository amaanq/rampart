-- Mailbox CRUD with nb_alias subselect.
-- Shared row type so the api layer can reuse one Rust struct
-- across list_for_user / by_id / by_id_user.
--: MailboxRow(display_name?, nb_alias)

--! list_for_user : MailboxRow
SELECT m.id, m.email::text AS email, m.display_name, m.verified, m.enabled,
       m.created_at,
       (SELECT COUNT(*) FROM alias a WHERE a.mailbox_id = m.id) AS nb_alias
FROM mailbox m
WHERE m.user_id = :user_id
ORDER BY m.id;

--! by_id : MailboxRow
SELECT m.id, m.email::text AS email, m.display_name, m.verified, m.enabled,
       m.created_at,
       (SELECT COUNT(*) FROM alias a WHERE a.mailbox_id = m.id) AS nb_alias
FROM mailbox m
WHERE m.id = :mailbox_id;

--! by_id_user : MailboxRow
SELECT m.id, m.email::text AS email, m.display_name, m.verified, m.enabled,
       m.created_at,
       (SELECT COUNT(*) FROM alias a WHERE a.mailbox_id = m.id) AS nb_alias
FROM mailbox m
WHERE m.id = :mailbox_id AND m.user_id = :user_id;

--! exists_verified
SELECT 1 AS one FROM mailbox
WHERE id = :mailbox_id AND user_id = :user_id AND enabled AND verified;

--! id_if_verified
SELECT id FROM mailbox
WHERE id = :mailbox_id AND user_id = :user_id AND enabled AND verified;

--! first_verified_for_user
SELECT id FROM mailbox
WHERE user_id = :user_id AND enabled AND verified
ORDER BY id LIMIT 1;

--! id_for_user_email
SELECT id FROM mailbox
WHERE user_id = :user_id AND email = :email AND enabled AND verified;

--! email_and_verified
SELECT email::text AS email, verified
FROM mailbox
WHERE id = :mailbox_id;

--! verified_for_user
SELECT verified FROM mailbox WHERE id = :mailbox_id AND user_id = :user_id;

--! create (display_name?)
INSERT INTO mailbox (user_id, email, display_name, verified)
VALUES (:user_id, :email, :display_name, FALSE)
RETURNING id;

--! create_verified (display_name?)
INSERT INTO mailbox (user_id, email, display_name, verified)
VALUES (:user_id, :email, :display_name, TRUE)
RETURNING id;

--! set_display_name (display_name?)
UPDATE mailbox SET display_name = :display_name
WHERE id = :mailbox_id AND user_id = :user_id;

--! set_enabled
UPDATE mailbox SET enabled = :enabled
WHERE id = :mailbox_id AND user_id = :user_id;

--! set_verified
UPDATE mailbox SET verified = TRUE WHERE id = :mailbox_id;

--! delete
DELETE FROM mailbox WHERE id = :mailbox_id AND user_id = :user_id;

--: MailboxAdminRow(display_name?)

--! list_admin : MailboxAdminRow
SELECT m.id, u.email::text AS user_email, m.email::text AS email,
       m.display_name, m.verified, m.enabled
FROM mailbox m JOIN "user" u ON u.id = m.user_id
ORDER BY m.user_id, m.id;

--! list_admin_for_user : MailboxAdminRow
SELECT m.id, u.email::text AS user_email, m.email::text AS email,
       m.display_name, m.verified, m.enabled
FROM mailbox m JOIN "user" u ON u.id = m.user_id
WHERE m.user_id = :user_id
ORDER BY m.id;
