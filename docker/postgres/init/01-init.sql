-- =========================================================
-- 1. Create application user
-- =========================================================
CREATE USER axum WITH ENCRYPTED PASSWORD 'axum123';

-- =========================================================
-- 2. Create database and make axum the OWNER
--    (THIS IS THE KEY FIX)
-- =========================================================
CREATE DATABASE axum_backend OWNER axum;

-- =========================================================
-- 3. Connect to the new database
-- =========================================================
\c axum_backend

-- =========================================================
-- 4. Ensure public schema ownership
-- =========================================================
ALTER SCHEMA public OWNER TO axum;

-- =========================================================
-- 5. Grant full schema privileges (safe & explicit)
-- =========================================================
GRANT USAGE, CREATE ON SCHEMA public TO axum;

-- Existing objects (in case extensions create them)
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO axum;

GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO axum;

-- Future objects (CRITICAL for sqlx migrations)
ALTER DEFAULT PRIVILEGES IN SCHEMA public
GRANT ALL ON TABLES TO axum;

ALTER DEFAULT PRIVILEGES IN SCHEMA public
GRANT ALL ON SEQUENCES TO axum;