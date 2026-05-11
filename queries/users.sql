-- User row CRUD + password ops + admin counts.
-- Nullability annotations: clorinde defaults to non-null for every
-- column, so nullable columns are explicitly listed via `(col?)`.

--! by_id_with_pwhash : (password_hash?)
SELECT password_hash
FROM "user"
WHERE id = :user_id;

--! by_email_for_basic_auth : (password_hash?, display_name?, max_aliases?, max_domains?)
SELECT id, email::text AS email, password_hash, enabled, is_admin,
       display_name, max_aliases, max_domains, created_at, updated_at
FROM "user"
WHERE email = :email AND enabled;

--! by_email_id
SELECT id
FROM "user"
WHERE email = :email AND enabled;

--! by_email_id_unfiltered
SELECT id
FROM "user"
WHERE email = :email;

--! email_exists_for_other
SELECT 1 AS one
FROM "user"
WHERE email = :email AND id <> :exclude_id;

--! info : (alias_count, mailbox_count, domain_count)
SELECT u.email::text AS email, u.is_admin,
       (SELECT COUNT(*) FROM alias WHERE user_id = u.id)         AS alias_count,
       (SELECT COUNT(*) FROM mailbox WHERE user_id = u.id)       AS mailbox_count,
       (SELECT COUNT(*) FROM alias_domain WHERE owner_id = u.id) AS domain_count
FROM "user" u
WHERE u.id = :user_id;

--! email_by_id
SELECT email::text AS email
FROM "user"
WHERE id = :user_id;

--! display_for_webauthn : (display_name?)
SELECT email::text AS email, display_name
FROM "user"
WHERE id = :user_id;

--! list_admin : (display_name?, nb_aliases, nb_mailboxes, nb_domains)
SELECT u.id, u.email::text AS email, u.enabled, u.is_admin, u.display_name, u.created_at,
       (SELECT COUNT(*) FROM alias WHERE user_id = u.id)         AS nb_aliases,
       (SELECT COUNT(*) FROM mailbox WHERE user_id = u.id)       AS nb_mailboxes,
       (SELECT COUNT(*) FROM alias_domain WHERE owner_id = u.id) AS nb_domains
FROM "user" u
ORDER BY u.id;

--! list_admin_compact : (nb_aliases, nb_domains)
SELECT u.id, u.email::text AS email, u.enabled, u.is_admin, u.created_at,
       (SELECT COUNT(*) FROM alias WHERE user_id = u.id)         AS nb_aliases,
       (SELECT COUNT(*) FROM alias_domain WHERE owner_id = u.id) AS nb_domains
FROM "user" u
ORDER BY u.id;

--! create (display_name?, password_hash?)
INSERT INTO "user" (email, password_hash, display_name, is_admin)
VALUES (:email, :password_hash, :display_name, :is_admin)
RETURNING id;

-- Returns 0 (no user yet) or 1 (at least one) — the GET /setup route
-- branches on this to decide whether to render the form or 404.
--! any_exists
SELECT EXISTS(SELECT 1 FROM "user") AS exists;

-- Atomic "create the first admin or do nothing" — the SELECT body
-- evaluates per-row from the synthetic VALUES row, and the WHERE
-- clause filters the row out if any user already exists. Returns
-- Some(id) on success, None when another request beat us to it.
-- Caller MUST hold the FIRST_ADMIN_LOCK_KEY advisory lock — under
-- READ COMMITTED the subquery snapshot is per-statement, so two
-- concurrent INSERT-WHERE-NOT-EXISTS can both see an empty user
-- table and both insert. The advisory lock serializes /setup POSTs.
--! create_first_admin (display_name?)
INSERT INTO "user" (email, password_hash, display_name, is_admin)
SELECT :email, :password_hash, :display_name, TRUE
WHERE NOT EXISTS (SELECT 1 FROM "user")
RETURNING id;

--! create_via_invite (display_name?)
INSERT INTO "user" (email, password_hash, display_name)
VALUES (:email, :password_hash, :display_name)
RETURNING id, is_admin;

--! set_password (password_hash?)
UPDATE "user" SET password_hash = :password_hash WHERE id = :user_id;

--! set_email
UPDATE "user" SET email = :email WHERE id = :user_id;

--! enable
UPDATE "user" SET enabled = TRUE WHERE id = :user_id;

--! disable
UPDATE "user" SET enabled = FALSE WHERE id = :user_id;

--! list_cli : (display_name?)
SELECT id, email::text AS email, is_admin, enabled, display_name
FROM "user"
ORDER BY id;

--! cap_and_count_aliases : (cap, current)
SELECT COALESCE(u.max_aliases::bigint, :default_cap) AS cap,
       (SELECT COUNT(*) FROM alias WHERE user_id = u.id) AS current
FROM "user" u
WHERE u.id = :user_id;

--! cap_and_count_domains : (cap, current)
SELECT COALESCE(u.max_domains::bigint, :default_cap) AS cap,
       (SELECT COUNT(*) FROM alias_domain WHERE owner_id = u.id) AS current
FROM "user" u
WHERE u.id = :user_id;
