-- abuse: sliding-window rate limiting via rate_limit_bucket.

--! check
INSERT INTO rate_limit_bucket (key, count, window_start)
VALUES (:key, 1, :now)
ON CONFLICT (key) DO UPDATE SET
   count = CASE WHEN rate_limit_bucket.window_start < :window_start_min THEN 1
                ELSE rate_limit_bucket.count + 1 END,
   window_start = CASE WHEN rate_limit_bucket.window_start < :window_start_min THEN :now
                       ELSE rate_limit_bucket.window_start END
RETURNING count;

--! clear
DELETE FROM rate_limit_bucket WHERE key = :key;
