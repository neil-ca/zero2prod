-- We wrap the whole migration in a transaction to make sure
-- it succeeds or fails atomically.
BEGIN;
    -- Backfill `status` for historical entries
    UPDATE subscriptions 
        SET status = 'confirmed'
        WHERE status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;

