-- email_log: reads + worker writes.

--! activity_for_alias : (from_address?, reason?)
SELECT action, status, from_address, reason, created_at
FROM email_log
WHERE alias_id = :alias_id
ORDER BY created_at DESC
LIMIT :lim OFFSET :off;

--! activity_for_alias_api : (reverse_contact_id?, from_address?, message_id?, reason?)
SELECT id, alias_id, reverse_contact_id, action, status, from_address, message_id,
       reason, created_at
FROM email_log
WHERE alias_id = :alias_id
ORDER BY created_at DESC
LIMIT :lim OFFSET :off;

--! insert_block (from_address?)
INSERT INTO email_log (alias_id, action, status, from_address)
VALUES (:alias_id, 'block', 'submitted', :from_address);

--! insert_forward (from_address?)
INSERT INTO email_log (alias_id, action, from_address)
VALUES (:alias_id, 'forward', :from_address)
RETURNING id;

--! flip_failed (reason?)
UPDATE email_log SET status = 'failed', reason = :reason
WHERE id = :email_log_id AND status = 'pending';

--! flip_submitted
UPDATE email_log SET status = 'submitted'
WHERE id = :email_log_id AND status = 'pending';
