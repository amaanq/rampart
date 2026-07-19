ALTER TABLE api_key
    ADD COLUMN scopes TEXT[] NOT NULL DEFAULT ARRAY['*']::TEXT[],
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'legacy',
    ADD COLUMN token_prefix TEXT,
    ADD COLUMN expires_at TIMESTAMPTZ,
    ADD CONSTRAINT api_key_scopes_nonempty CHECK (cardinality(scopes) > 0),
    ADD CONSTRAINT api_key_kind_valid CHECK (kind IN ('legacy', 'extension'));

CREATE INDEX ix_api_key_active_expiry
    ON api_key(expires_at)
    WHERE revoked_at IS NULL AND expires_at IS NOT NULL;

CREATE TABLE api_idempotency (
    api_key_id     BIGINT NOT NULL REFERENCES api_key(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL CHECK (length(idempotency_key) BETWEEN 1 AND 128),
    alias_id       BIGINT REFERENCES alias(id) ON DELETE CASCADE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (api_key_id, idempotency_key)
);
