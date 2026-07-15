-- Sieve render reads. The advisory-lock + idle-in-txn timeout calls
-- are kept as raw SQL in src/sieve.rs (Cornucopia rejects void-returning
-- queries — they're not "data" in the typed sense anyway).

--! all_alias_domain_names
SELECT domain::text AS domain FROM alias_domain ORDER BY domain;
