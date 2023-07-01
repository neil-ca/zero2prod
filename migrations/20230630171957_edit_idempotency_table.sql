-- Add migration script here
ALTER TABLE idempotency
ADD COLUMN response_body BYTEA NOT NULL;
