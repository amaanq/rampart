-- Extension mutation idempotency.

--! claim
INSERT INTO api_idempotency (api_key_id, idempotency_key)
VALUES (:api_key_id, :idempotency_key)
ON CONFLICT DO NOTHING
RETURNING api_key_id;

--! alias_id : (alias_id?)
SELECT alias_id
FROM api_idempotency
WHERE api_key_id = :api_key_id AND idempotency_key = :idempotency_key;

--! finish
UPDATE api_idempotency SET alias_id = :alias_id
WHERE api_key_id = :api_key_id AND idempotency_key = :idempotency_key;
