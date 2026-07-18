-- rampart V001 — initial schema (multi-user + phase-2 ready).
--
-- All tables created up front. Rust code fills them in three chunks:
--   1. Multi-user foundation uses: user, session, api_key, invite_token,
--      and the user_id / owner_id / shared columns on existing tables.
--   2. Self-service + passkeys uses: webauthn_credential, webauthn_ceremony,
--      password_reset_token, email_change_token, mailbox_verify_token,
--      rate_limit_bucket.
--   3. Reply-via-alias uses: reverse_contact.
-- No migration chain yet because nothing is deployed.

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;

--------------------------------------------------------------------------------
-- user
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS "user" (
    id              BIGSERIAL PRIMARY KEY,
    email           CITEXT NOT NULL UNIQUE,
    -- argon2id PHC; NULL means passkey-only (we don't allow that at signup,
    -- but admin-created users may start without a password and set one via
    -- invite flow).
    password_hash   TEXT,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    is_admin        BOOLEAN NOT NULL DEFAULT FALSE,
    display_name    TEXT,
    -- Abuse control caps. NULL = use global default from config.
    max_aliases     INTEGER,
    max_domains     INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

--------------------------------------------------------------------------------
-- session  (browser cookies)
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS session (
    id              BYTEA PRIMARY KEY,          -- 32 random bytes
    user_id         BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    user_agent      TEXT,
    ip              INET
);
CREATE INDEX IF NOT EXISTS ix_session_user    ON session(user_id);
CREATE INDEX IF NOT EXISTS ix_session_expires ON session(expires_at);

--------------------------------------------------------------------------------
-- api_key  (per-user bearer tokens, multiple allowed)
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS api_key (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    key_hash        BYTEA NOT NULL UNIQUE,      -- sha256 of the token
    last_used_at    TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_api_key_user ON api_key(user_id);

--------------------------------------------------------------------------------
-- invite_token
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS invite_token (
    token_hash      BYTEA PRIMARY KEY,          -- sha256 of link token
    created_by      BIGINT REFERENCES "user"(id) ON DELETE SET NULL,
    preset_email    CITEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ,
    used_by         BIGINT REFERENCES "user"(id) ON DELETE SET NULL
);

--------------------------------------------------------------------------------
-- alias_domain
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS alias_domain (
    id                  BIGSERIAL PRIMARY KEY,
    domain              CITEXT NOT NULL UNIQUE,
    -- NULL owner = admin-managed "global" domain (rare, used for retired/disabled).
    -- Non-NULL owner + shared=FALSE = private user domain.
    -- Non-NULL owner + shared=TRUE  = shared pool domain (admin-promoted).
    owner_id            BIGINT REFERENCES "user"(id) ON DELETE SET NULL,
    shared              BOOLEAN NOT NULL DEFAULT FALSE,
    catch_all           BOOLEAN NOT NULL DEFAULT FALSE,
    -- Spam-amplification guard: if NOT NULL, rampart_resolve_or_create refuses
    -- to auto-create more than this many auto_created aliases for the
    -- domain. MUST be set when the domain's MX is reachable from the
    -- public internet, else anyone can mint unbounded aliases by sending
    -- mail to fresh local-parts.
    max_auto_created    INTEGER,
    random_prefix       TEXT NOT NULL DEFAULT ''
        -- random_local_part appends 10 hex chars, so prefix is capped
        -- at 54 to fit the 64-byte alias-local-part cap. Reserves `ra+`
        -- and `bnc+` (Sieve reply / bounce routes) and the trigger's
        -- dot rules so direct-SQL inserts can't produce a value the
        -- alias_validate trigger would then reject.
        CHECK (
            random_prefix = ''
            OR (
                length(random_prefix) BETWEEN 1 AND 54
                AND random_prefix ~ '^[A-Za-z0-9._+-]+$'
                AND lower(random_prefix) NOT LIKE 'ra+%'
                AND lower(random_prefix) NOT LIKE 'bnc+%'
                AND left(random_prefix, 1) <> '.'
                AND right(random_prefix, 1) <> '.'
                AND position('..' in random_prefix) = 0
            )
        ),
    -- Pinned to 'ra+': the rendered Sieve hardcodes the `ra+*` localpart
    -- glob and worker/pipeline.rs hardcodes `ra+<token>@<domain>` reply
    -- addresses. Relaxing this requires threading a per-domain prefix
    -- through the Sieve template and the worker's reverse-contact upsert.
    reply_prefix        TEXT NOT NULL DEFAULT 'ra+' CHECK (reply_prefix = 'ra+'),
    default_mailbox_id  BIGINT,                 -- FK added after mailbox created
    -- Cached Stalwart DKIM public records and the most recent DNS observations
    -- power the domain setup page without querying either service on list views.
    dkim_records        JSONB NOT NULL DEFAULT '[]'::jsonb
        CONSTRAINT alias_domain_dkim_records_array
        CHECK (jsonb_typeof(dkim_records) = 'array'),
    dns_status          JSONB NOT NULL DEFAULT '{}'::jsonb
        CONSTRAINT alias_domain_dns_status_object
        CHECK (jsonb_typeof(dns_status) = 'object'),
    dns_checked_at      TIMESTAMPTZ,
    dns_verified_at     TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Domain shape: lowercase ASCII labels (a-z 0-9 -), each up to 63
    -- chars, dot-separated, total length capped. The rendered Sieve
    -- embeds the domain inside `envelope :domain :is "to" "<domain>"`
    -- with no escaping, so this regex doubles as Sieve-injection guard.
    CONSTRAINT domain_shape CHECK (
        length(domain::text) BETWEEN 3 AND 253
        AND domain::text ~ '^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)+$'
    ),
    -- catch_all without a cap turns a public-MX domain into an
    -- unbounded alias minter; force operators to set max_auto_created
    -- explicitly.
    CONSTRAINT catch_all_requires_cap
        CHECK (NOT catch_all OR max_auto_created IS NOT NULL)
);
CREATE INDEX IF NOT EXISTS ix_alias_domain_owner ON alias_domain(owner_id);

--------------------------------------------------------------------------------
-- mailbox
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS mailbox (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    email           CITEXT NOT NULL,
    display_name    TEXT,
    verified        BOOLEAN NOT NULL DEFAULT FALSE,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, email)
);
CREATE INDEX IF NOT EXISTS ix_mailbox_user ON mailbox(user_id);

ALTER TABLE alias_domain DROP CONSTRAINT IF EXISTS alias_domain_default_mailbox_fk;
ALTER TABLE alias_domain
    ADD CONSTRAINT alias_domain_default_mailbox_fk
    FOREIGN KEY (default_mailbox_id) REFERENCES mailbox(id) ON DELETE SET NULL;

--------------------------------------------------------------------------------
-- alias
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS alias (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    address         CITEXT NOT NULL UNIQUE,          -- globally unique
    domain_id       BIGINT NOT NULL REFERENCES alias_domain(id) ON DELETE RESTRICT,
    mailbox_id      BIGINT NOT NULL REFERENCES mailbox(id) ON DELETE RESTRICT,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    note            TEXT,
    auto_created    BOOLEAN NOT NULL DEFAULT FALSE,
    pinned          BOOLEAN NOT NULL DEFAULT FALSE,
    nb_forward      BIGINT NOT NULL DEFAULT 0,
    nb_block        BIGINT NOT NULL DEFAULT 0,
    nb_reply        BIGINT NOT NULL DEFAULT 0,
    last_email_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_alias_user         ON alias (user_id);
CREATE INDEX IF NOT EXISTS ix_alias_domain_id    ON alias (domain_id);
CREATE INDEX IF NOT EXISTS ix_alias_mailbox_id   ON alias (mailbox_id);
CREATE INDEX IF NOT EXISTS ix_alias_enabled      ON alias (enabled) WHERE enabled = TRUE;
CREATE INDEX IF NOT EXISTS ix_alias_last_email   ON alias (last_email_at DESC NULLS LAST);

-- Full respecification of alias_validate:
--   1. alias.domain_id exists
--   2. alias.address domain suffix matches alias_domain.domain
--   3. local-part does not start with alias_domain.reply_prefix (reserved)
--   4. domain is accessible to user: shared=true, or user owns domain, or
--      user is admin
--   5. alias.mailbox_id belongs to the same user
CREATE OR REPLACE FUNCTION alias_validate() RETURNS trigger AS $$
DECLARE
    dom_row      alias_domain%ROWTYPE;
    mbox_user_id BIGINT;
    addr_domain  TEXT;
    local_part   TEXT;
    user_admin   BOOLEAN;
BEGIN
    -- FOR UPDATE serializes us against `admin_domain_set_shared`'s
    -- transactional unshare check — without it, a concurrent alias
    -- INSERT can read shared=TRUE while admin's COUNT+UPDATE flips it,
    -- stranding the new alias on a now-unshared domain. Locking the
    -- domain row here means admin's unshare waits until our INSERT
    -- commits, then sees us in its COUNT and refuses.
    SELECT * INTO dom_row FROM alias_domain WHERE id = NEW.domain_id FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'alias.domain_id=% does not reference alias_domain', NEW.domain_id;
    END IF;

    addr_domain := split_part(NEW.address::text, '@', 2);
    IF addr_domain = '' THEN
        RAISE EXCEPTION 'alias.address=% must contain @domain', NEW.address;
    END IF;
    IF addr_domain::CITEXT <> dom_row.domain THEN
        RAISE EXCEPTION 'alias.address=% domain part does not match alias_domain (expected %)',
            NEW.address, dom_row.domain;
    END IF;

    local_part := split_part(NEW.address::text, '@', 1);
    IF position(lower(dom_row.reply_prefix) in lower(local_part)) = 1 THEN
        RAISE EXCEPTION 'alias.address=% local-part is reserved by reply_prefix=%',
            NEW.address, dom_row.reply_prefix;
    END IF;
    -- Bounce VERP namespace: the Sieve routes `bnc+f+*` and `bnc+r+*`
    -- directly into handle_bounce, so a user-minted `bnc+` alias could
    -- siphon forwards into the bounce handler or frame an existing
    -- email_log row as bounced.
    IF position('bnc+' in lower(local_part)) = 1 THEN
        RAISE EXCEPTION 'alias.address=% local-part starts with reserved prefix bnc+', NEW.address;
    END IF;

    -- Trigger mirrors api::validate_local_part_fragment for direct-SQL
    -- inserts. ASCII-only — SMTPUTF8 local-parts need an explicit policy.
    IF length(local_part) = 0 OR length(local_part) > 64 THEN
        RAISE EXCEPTION 'alias.address=% local-part length must be 1..64 bytes', NEW.address;
    END IF;
    IF local_part !~ '^[A-Za-z0-9._+-]+$' THEN
        RAISE EXCEPTION 'alias.address=% local-part must be ASCII alphanumeric or [._+-]', NEW.address;
    END IF;
    IF left(local_part, 1) = '.' OR right(local_part, 1) = '.' OR position('..' in local_part) > 0 THEN
        RAISE EXCEPTION 'alias.address=% local-part must not start/end with dot or contain consecutive dots', NEW.address;
    END IF;

    SELECT is_admin INTO user_admin FROM "user" WHERE id = NEW.user_id;
    IF user_admin IS NULL THEN
        RAISE EXCEPTION 'alias.user_id=% does not exist', NEW.user_id;
    END IF;

    IF NOT (dom_row.shared OR dom_row.owner_id = NEW.user_id OR user_admin) THEN
        RAISE EXCEPTION 'domain % is not accessible to user %', dom_row.domain, NEW.user_id;
    END IF;

    SELECT user_id INTO mbox_user_id FROM mailbox WHERE id = NEW.mailbox_id;
    IF mbox_user_id IS NULL THEN
        RAISE EXCEPTION 'alias.mailbox_id=% does not exist', NEW.mailbox_id;
    END IF;
    IF mbox_user_id <> NEW.user_id THEN
        RAISE EXCEPTION 'mailbox % does not belong to user %', NEW.mailbox_id, NEW.user_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS alias_validate_bi ON alias;
CREATE TRIGGER alias_validate_bi BEFORE INSERT OR UPDATE ON alias
    FOR EACH ROW EXECUTE FUNCTION alias_validate();

-- alias_domain's default_mailbox_id must belong to the domain's owner,
-- preventing an admin shared-domain catch-all from auto-forwarding into
-- another user's mailbox.
CREATE OR REPLACE FUNCTION alias_domain_validate() RETURNS trigger AS $$
DECLARE
    mbox_user_id  BIGINT;
    mbox_is_admin BOOLEAN;
    mbox_verified BOOLEAN;
BEGIN
    IF NEW.default_mailbox_id IS NOT NULL THEN
        SELECT user_id, verified INTO mbox_user_id, mbox_verified
          FROM mailbox WHERE id = NEW.default_mailbox_id;
        IF mbox_user_id IS NULL THEN
            RAISE EXCEPTION 'alias_domain.default_mailbox_id=% does not exist', NEW.default_mailbox_id;
        END IF;
        IF NEW.owner_id IS NOT NULL THEN
            -- Private or shared-with-owner domain: mailbox must belong to owner
            IF mbox_user_id <> NEW.owner_id THEN
                RAISE EXCEPTION 'default_mailbox does not belong to domain owner (%)', NEW.owner_id;
            END IF;
        ELSE
            -- Global/admin-only domain (owner_id IS NULL): default_mailbox
            -- must belong to an admin so a user's mailbox can't be made
            -- a catch-all sink for other users' rejected mail.
            SELECT is_admin INTO mbox_is_admin FROM "user" WHERE id = mbox_user_id;
            IF NOT COALESCE(mbox_is_admin, FALSE) THEN
                RAISE EXCEPTION 'global domain default_mailbox must belong to an admin user';
            END IF;
        END IF;
        -- Catch-all target must be verified, otherwise auto-forward
        -- could chain to an unproven destination.
        IF NOT COALESCE(mbox_verified, FALSE) THEN
            RAISE EXCEPTION 'default_mailbox must be verified to serve as catch-all target';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS alias_domain_validate_bi ON alias_domain;
CREATE TRIGGER alias_domain_validate_bi BEFORE INSERT OR UPDATE ON alias_domain
    FOR EACH ROW EXECUTE FUNCTION alias_domain_validate();

--------------------------------------------------------------------------------
-- email_log  (no user_id; derive via join through alias to prevent drift)
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS email_log (
    id                  BIGSERIAL PRIMARY KEY,
    alias_id            BIGINT NOT NULL REFERENCES alias(id) ON DELETE CASCADE,
    reverse_contact_id  BIGINT,   -- FK set below; NULL unless a reply
    -- Operation taxonomy: forward / block / reply / bounce.
    action              TEXT NOT NULL CHECK (action IN ('forward','block','reply','bounce')),
    -- Lifecycle: pre-INSERT 'pending' → post-submit 'submitted' /
    -- 'failed', VERP DSN handler → 'bounced'. Distinguishes "submit
    -- failed locally" from "delivered then DSN'd". Block rows skip
    -- pending and land as 'submitted' (self-contained accept-and-drop).
    status              TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','submitted','failed','bounced')),
    from_address        TEXT,
    message_id          TEXT,
    reason              TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_email_log_alias_created ON email_log (alias_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_email_log_created       ON email_log (created_at DESC);

--------------------------------------------------------------------------------
-- reverse_contact  (phase 2)
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS reverse_contact (
    id              BIGSERIAL PRIMARY KEY,
    alias_id        BIGINT NOT NULL REFERENCES alias(id) ON DELETE CASCADE,
    real_email      CITEXT NOT NULL,
    token           TEXT NOT NULL UNIQUE,
    reply_address   CITEXT NOT NULL UNIQUE,
    display_name    TEXT,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    block_reply     BOOLEAN NOT NULL DEFAULT FALSE,
    last_seen_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (alias_id, real_email)
);

CREATE INDEX IF NOT EXISTS ix_reverse_contact_alias ON reverse_contact (alias_id);
CREATE INDEX IF NOT EXISTS ix_reverse_contact_reply ON reverse_contact (reply_address);

ALTER TABLE email_log DROP CONSTRAINT IF EXISTS email_log_reverse_contact_fk;
ALTER TABLE email_log
    ADD CONSTRAINT email_log_reverse_contact_fk
    FOREIGN KEY (reverse_contact_id) REFERENCES reverse_contact(id) ON DELETE SET NULL;

CREATE OR REPLACE FUNCTION reverse_contact_validate() RETURNS trigger AS $$
DECLARE
    expected_prefix TEXT;
    local_part      TEXT;
    addr_domain     TEXT;
    dom_id          BIGINT;
BEGIN
    addr_domain := split_part(NEW.reply_address::text, '@', 2);
    IF addr_domain = '' THEN
        RAISE EXCEPTION 'reverse_contact.reply_address=% must contain @domain', NEW.reply_address;
    END IF;

    SELECT ad.id, ad.reply_prefix
        INTO dom_id, expected_prefix
    FROM alias_domain ad WHERE ad.domain = addr_domain::CITEXT;
    IF dom_id IS NULL THEN
        RAISE EXCEPTION 'reverse_contact.reply_address domain % is not a managed alias_domain', addr_domain;
    END IF;

    local_part := split_part(NEW.reply_address::text, '@', 1);
    IF position(lower(expected_prefix) in lower(local_part)) <> 1 THEN
        RAISE EXCEPTION 'reverse_contact.reply_address=% must start with reply_prefix=% for domain %',
            NEW.reply_address, expected_prefix, addr_domain;
    END IF;

    IF EXISTS (SELECT 1 FROM alias WHERE address = NEW.reply_address) THEN
        RAISE EXCEPTION 'reverse_contact.reply_address=% collides with an existing alias',
            NEW.reply_address;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS reverse_contact_validate_bi ON reverse_contact;
CREATE TRIGGER reverse_contact_validate_bi BEFORE INSERT OR UPDATE ON reverse_contact
    FOR EACH ROW EXECUTE FUNCTION reverse_contact_validate();

--------------------------------------------------------------------------------
-- Phase-2 chunk-2 tables: webauthn, token flows, rate limits
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS webauthn_credential (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    credential_id   BYTEA NOT NULL UNIQUE,          -- webauthn credential ID
    credential_blob BYTEA NOT NULL,                 -- serialized Passkey
    sign_count      INTEGER NOT NULL DEFAULT 0,
    name            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at    TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS ix_webauthn_credential_user ON webauthn_credential(user_id);

CREATE TABLE IF NOT EXISTS webauthn_ceremony (
    id              BYTEA PRIMARY KEY,              -- random challenge id
    user_id         BIGINT REFERENCES "user"(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL CHECK (kind IN ('register','auth')),
    state_blob      BYTEA NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS ix_webauthn_ceremony_expires ON webauthn_ceremony(expires_at);

CREATE TABLE IF NOT EXISTS password_reset_token (
    token_hash      BYTEA PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS email_change_token (
    token_hash      BYTEA PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    new_email       CITEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS mailbox_verify_token (
    token_hash      BYTEA PRIMARY KEY,
    mailbox_id      BIGINT NOT NULL REFERENCES mailbox(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS rate_limit_bucket (
    key             TEXT PRIMARY KEY,               -- e.g. "reset:user:42"
    count           INTEGER NOT NULL DEFAULT 0,
    window_start    TIMESTAMPTZ NOT NULL DEFAULT now()
);

--------------------------------------------------------------------------------
-- Views consumed by the Sieve scripts
--
-- rampart_sieve_lookup: phase-1 forward-path lookup. Sieve calls:
--   SELECT id FROM rampart_sieve_lookup WHERE address = $1 AND enabled
-- The lookup is user-agnostic — any user's alias is a match because alias
-- addresses are globally unique. User attribution happens in the LMTP worker
-- via alias_id → alias.user_id join.
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW rampart_sieve_lookup AS
    SELECT
        a.id                      AS alias_id,
        a.address,
        (a.enabled AND m.enabled AND u.enabled) AS enabled,
        m.email                   AS forward_to,
        a.user_id
    FROM alias a
    JOIN mailbox m ON m.id = a.mailbox_id
    JOIN "user"   u ON u.id = a.user_id;

--------------------------------------------------------------------------------
-- rampart_resolve_or_create — called by the session.rcpt Sieve:
--   SELECT rampart_resolve_or_create(?)
-- Returns the alias id for an existing enabled alias, or auto-creates an
-- alias on a catch-all domain (respecting max_auto_created, mailbox/user
-- enabled state, and mailbox verification). Returns NULL when no valid
-- target exists; the Sieve then rejects 550.
--
-- Race safety: per-domain advisory lock serializes concurrent catch-all
-- inserts so max_auto_created is a hard cap, not advisory. The 2-arg
-- advisory-lock form lives in a separate lock space from api.rs's 1-arg
-- form — no collision possible.
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION rampart_resolve_or_create(addr CITEXT)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE
    existing_id    BIGINT;
    d              alias_domain%ROWTYPE;
    dmbox_uid      BIGINT;
    dmbox_enabled  BOOLEAN;
    dmbox_verified BOOLEAN;
    owner_enabled  BOOLEAN;
    auto_count     BIGINT;
    user_max       BIGINT;
    user_count     BIGINT;
    new_id         BIGINT;
BEGIN
    -- Fast path: existing enabled alias (view rolls up alias/mailbox/user enabled).
    SELECT alias_id INTO existing_id FROM rampart_sieve_lookup
     WHERE address = addr AND enabled LIMIT 1;
    IF existing_id IS NOT NULL THEN
        RETURN existing_id;
    END IF;

    -- Slow path: catch-all on the target domain? FOR UPDATE so a
    -- concurrent admin patch (disable catch_all, lower max_auto_created,
    -- rotate default_mailbox_id) serializes against the resolver — we
    -- don't want to auto-create from a snapshot of values the admin
    -- already committed away.
    SELECT * INTO d FROM alias_domain
     WHERE domain = split_part(addr::text, '@', 2)::CITEXT
       AND catch_all AND default_mailbox_id IS NOT NULL
     FOR UPDATE;
    IF NOT FOUND THEN RETURN NULL; END IF;

    -- Serialize concurrent catch-all inserts on THIS domain so
    -- max_auto_created is a hard cap. 2-arg form; class=3 doesn't
    -- collide with api.rs's 1-arg lock space.
    PERFORM pg_advisory_xact_lock(3::int, hashint8(d.id));

    SELECT user_id, enabled, verified
      INTO dmbox_uid, dmbox_enabled, dmbox_verified
      FROM mailbox WHERE id = d.default_mailbox_id;
    IF dmbox_uid IS NULL OR NOT dmbox_enabled OR NOT dmbox_verified THEN
        RETURN NULL;
    END IF;
    SELECT enabled INTO owner_enabled FROM "user" WHERE id = dmbox_uid;
    IF NOT COALESCE(owner_enabled, FALSE) THEN RETURN NULL; END IF;

    -- Per-user alias cap. Same default as quota.rs::DEFAULT_MAX_ALIASES = 200 —
    -- keep the two in sync. Without this, a user with catch-all on a domain
    -- whose max_auto_created is permissive (or NULL) could blow past their
    -- per-user cap via fresh local-parts. Take the API's per-user lock
    -- (1-arg form, class 1 per quota.rs::lock_id) so concurrent API alias
    -- creation for the same user serializes against us. Lock order:
    -- per-domain (above) then per-user (here); API only takes per-user,
    -- so no opposite-order paths exist → no deadlock.
    PERFORM pg_advisory_xact_lock(
        (1::bigint << 56) | (dmbox_uid & ((1::bigint << 56) - 1))
    );
    SELECT COALESCE(max_aliases::bigint, 200) INTO user_max
      FROM "user" WHERE id = dmbox_uid;
    SELECT count(*) INTO user_count FROM alias WHERE user_id = dmbox_uid;
    IF user_count >= user_max THEN
        RETURN NULL;
    END IF;

    IF d.max_auto_created IS NOT NULL THEN
        SELECT count(*) INTO auto_count FROM alias
         WHERE domain_id = d.id AND auto_created;
        IF auto_count >= d.max_auto_created THEN
            RETURN NULL;
        END IF;
    END IF;

    -- alias_validate trigger enforces ownership / domain-access rules.
    INSERT INTO alias (user_id, address, domain_id, mailbox_id, auto_created)
    VALUES (dmbox_uid, addr, d.id, d.default_mailbox_id, TRUE)
    RETURNING id INTO new_id;
    RETURN new_id;
EXCEPTION WHEN unique_violation THEN
    -- Lost the race to a user-initiated insert (the advisory lock
    -- excludes concurrent catch-all inserts on the same domain). Re-read
    -- through the view; if absent the Sieve rejects, which is fine.
    SELECT alias_id INTO existing_id FROM rampart_sieve_lookup
     WHERE address = addr AND enabled LIMIT 1;
    RETURN existing_id;
END;
$$;

-- Stalwart's Sieve binds the recipient address as text; Postgres won't
-- auto-coerce text→citext for function dispatch, so a citext-only
-- function fails to resolve and the Sieve 5xx's every legitimate
-- forward. This text overload re-dispatches to the citext implementation.
-- Both are SECURITY DEFINER so the stalwart-mail role (which has no
-- direct table privileges) can run them with the migration owner's
-- access to alias / mailbox / "user" / alias_domain.
CREATE OR REPLACE FUNCTION rampart_resolve_or_create(addr text) RETURNS bigint
LANGUAGE sql SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
    SELECT rampart_resolve_or_create($1::citext);
$$;

ALTER FUNCTION rampart_resolve_or_create(citext)
    SECURITY DEFINER
    SET search_path = public, pg_temp;

-- rampart_resolve_reply: minimum-privilege Sieve dispatch for the reply path.
-- Granting bare SELECT on reverse_contact would expose real_email and
-- token to the stalwart-mail role across every row; SECURITY DEFINER
-- exposes only the single id the Sieve actually needs.
CREATE OR REPLACE FUNCTION rampart_resolve_reply(addr text) RETURNS bigint
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
    SELECT id FROM reverse_contact
    WHERE reply_address = $1::citext
      AND enabled
      AND NOT block_reply
    LIMIT 1;
$$;

-- Postgres grants EXECUTE to PUBLIC by default. Combined with
-- SECURITY DEFINER, that lets any cluster role run our resolvers with
-- the migration owner's privileges. REVOKE PUBLIC unconditionally and
-- grant only to stalwart-mail.
REVOKE ALL ON FUNCTION rampart_resolve_or_create(citext) FROM PUBLIC;
REVOKE ALL ON FUNCTION rampart_resolve_or_create(text)   FROM PUBLIC;
REVOKE ALL ON FUNCTION rampart_resolve_reply(text)       FROM PUBLIC;

-- Grants for the stalwart-mail postgres role used by the Sieve hook.
-- Gated so a host without that role can still apply the migration.
DO $grant$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'stalwart-mail') THEN
        EXECUTE 'GRANT USAGE   ON SCHEMA public                            TO "stalwart-mail"';
        EXECUTE 'GRANT EXECUTE ON FUNCTION rampart_resolve_or_create(citext)    TO "stalwart-mail"';
        EXECUTE 'GRANT EXECUTE ON FUNCTION rampart_resolve_or_create(text)      TO "stalwart-mail"';
        EXECUTE 'GRANT EXECUTE ON FUNCTION rampart_resolve_reply(text)          TO "stalwart-mail"';
    END IF;
END
$grant$;

--------------------------------------------------------------------------------
-- updated_at auto-touch triggers
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION touch_updated_at() RETURNS trigger AS $$
BEGIN
    NEW.updated_at := now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS user_touch ON "user";
CREATE TRIGGER user_touch            BEFORE UPDATE ON "user"
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
DROP TRIGGER IF EXISTS alias_domain_touch ON alias_domain;
CREATE TRIGGER alias_domain_touch    BEFORE UPDATE ON alias_domain
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
DROP TRIGGER IF EXISTS mailbox_touch ON mailbox;
CREATE TRIGGER mailbox_touch         BEFORE UPDATE ON mailbox
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
DROP TRIGGER IF EXISTS alias_touch ON alias;
CREATE TRIGGER alias_touch           BEFORE UPDATE ON alias
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
DROP TRIGGER IF EXISTS reverse_contact_touch ON reverse_contact;
CREATE TRIGGER reverse_contact_touch BEFORE UPDATE ON reverse_contact
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
