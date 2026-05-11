-- webauthn_ceremony + webauthn_credential.

--! ceremony_insert_register (user_id?)
INSERT INTO webauthn_ceremony (id, user_id, kind, state_blob, expires_at)
VALUES (:id, :user_id, 'register', :state_blob, :expires_at);

--! ceremony_insert_auth (user_id?)
INSERT INTO webauthn_ceremony (id, user_id, kind, state_blob, expires_at)
VALUES (:id, :user_id, 'auth', :state_blob, :expires_at);

--! ceremony_consume_register
DELETE FROM webauthn_ceremony
WHERE id = :id AND user_id = :user_id AND kind = 'register'
  AND expires_at > now()
RETURNING state_blob;

--! ceremony_consume_auth
DELETE FROM webauthn_ceremony
WHERE id = :id AND kind = 'auth' AND expires_at > now()
RETURNING state_blob;

--! credentials_for_user
SELECT credential_blob
FROM webauthn_credential
WHERE user_id = :user_id;

--! credential_insert
INSERT INTO webauthn_credential (user_id, credential_id, credential_blob, name)
VALUES (:user_id, :credential_id, :credential_blob, :name);

--! credential_for_update
SELECT sign_count, credential_blob
FROM webauthn_credential
WHERE credential_id = :credential_id
FOR UPDATE;

--! credential_update_blob_and_count
UPDATE webauthn_credential
SET sign_count = GREATEST(sign_count, :sign_count::int),
    credential_blob = :credential_blob,
    last_used_at = now()
WHERE credential_id = :credential_id;

--! credential_update_count_only
UPDATE webauthn_credential
SET sign_count = GREATEST(sign_count, :sign_count::int),
    last_used_at = now()
WHERE credential_id = :credential_id;

--! credential_user_id
SELECT c.user_id
FROM webauthn_credential c JOIN "user" u ON u.id = c.user_id
WHERE c.credential_id = :credential_id AND u.enabled;

--! list_for_user : (last_used_at?)
SELECT id, name, created_at, last_used_at
FROM webauthn_credential
WHERE user_id = :user_id
ORDER BY id;

--! delete_for_user
DELETE FROM webauthn_credential
WHERE id = :credential_pk AND user_id = :user_id;
