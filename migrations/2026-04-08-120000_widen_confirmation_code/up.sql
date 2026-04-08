-- Widen confirmation_code column to support 32-byte hex tokens (64 chars)
-- Previous: VARCHAR(20) for 8-char alphanumeric codes
ALTER TABLE users ALTER COLUMN confirmation_code TYPE VARCHAR(64);
