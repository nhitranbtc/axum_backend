-- Revert confirmation_code column to VARCHAR(20)
-- Warning: existing 64-char codes will be truncated
ALTER TABLE users ALTER COLUMN confirmation_code TYPE VARCHAR(20);
