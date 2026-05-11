-- alias_domain CRUD.
--: DomainRow(owner_id?, default_mailbox_id?, nb_alias)
--: AliasDomainRow(owner_id?, default_mailbox_id?)

--! list_for_user : DomainRow
SELECT d.id, d.domain::text AS domain, d.owner_id, d.shared, d.catch_all,
       d.random_prefix, d.reply_prefix, d.default_mailbox_id,
       (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias
FROM alias_domain d
WHERE d.shared OR d.owner_id = :user_id OR :is_admin
ORDER BY d.id;

--! by_id : DomainRow
SELECT d.id, d.domain::text AS domain, d.owner_id, d.shared, d.catch_all,
       d.random_prefix, d.reply_prefix, d.default_mailbox_id,
       (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias
FROM alias_domain d
WHERE d.id = :domain_id;

--! list_for_dashboard : (owner_id?, nb_alias)
SELECT d.id, d.domain::text AS domain, d.shared, d.owner_id,
       d.random_prefix, d.reply_prefix,
       (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias
FROM alias_domain d
WHERE d.shared OR d.owner_id = :user_id OR :is_admin
ORDER BY d.shared DESC, d.id;

--! list_admin : (owner_email?, nb_alias)
SELECT d.id, d.domain::text AS domain, d.shared,
       u.email::text AS owner_email,
       (SELECT COUNT(*) FROM alias a WHERE a.domain_id = d.id) AS nb_alias
FROM alias_domain d LEFT JOIN "user" u ON u.id = d.owner_id
ORDER BY d.id;

--! exists_managable
SELECT 1 AS one FROM alias_domain
WHERE id = :domain_id AND (owner_id = :user_id OR :is_admin);

--! catch_all_and_cap : (max_auto_created?)
SELECT catch_all, max_auto_created
FROM alias_domain
WHERE id = :domain_id;

--! set_catch_all_and_cap (max_auto_created?)
UPDATE alias_domain SET catch_all = :catch_all, max_auto_created = :max_auto_created
WHERE id = :domain_id;

--! set_random_prefix
UPDATE alias_domain SET random_prefix = :random_prefix WHERE id = :domain_id;

--! set_default_mailbox (default_mailbox_id?)
UPDATE alias_domain SET default_mailbox_id = :default_mailbox_id WHERE id = :domain_id;

--! set_shared
UPDATE alias_domain SET shared = :shared WHERE id = :domain_id;

--! delete
DELETE FROM alias_domain WHERE id = :domain_id AND (owner_id = :user_id OR :is_admin);

--! create (owner_id?, random_prefix?)
INSERT INTO alias_domain (domain, owner_id, shared, random_prefix)
VALUES (:domain, :owner_id, FALSE, COALESCE(:random_prefix, ''))
RETURNING id;

--! by_domain_for_user : AliasDomainRow
SELECT id, domain::text AS domain, owner_id, shared, catch_all,
       random_prefix, reply_prefix, default_mailbox_id, created_at, updated_at
FROM alias_domain
WHERE domain = :domain AND (shared OR owner_id = :user_id OR :is_admin);

--! first_accessible_for_user : AliasDomainRow
SELECT id, domain::text AS domain, owner_id, shared, catch_all,
       random_prefix, reply_prefix, default_mailbox_id, created_at, updated_at
FROM alias_domain
WHERE shared OR owner_id = :user_id OR :is_admin
ORDER BY shared DESC, id LIMIT 1;

--! id_by_domain
SELECT id FROM alias_domain WHERE domain = :domain;

--! all_domain_names
SELECT domain::text AS domain FROM alias_domain ORDER BY domain;

--! set_default_mailbox_by_owner_email
UPDATE alias_domain d SET default_mailbox_id = (
    SELECT m.id FROM mailbox m
    WHERE m.email = :mailbox_email AND m.user_id = d.owner_id
      AND m.enabled AND m.verified
)
WHERE d.domain = :domain;

--! default_mailbox_is_null : (is_null?)
SELECT default_mailbox_id IS NULL AS is_null
FROM alias_domain
WHERE domain = :domain;
