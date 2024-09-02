-- Add down migration script here

DROP TABLE IF EXISTS queue;
DROP TABLE IF EXISTS bonus_points;
DROP TABLE IF EXISTS techniques;
DROP TABLE IF EXISTS patterns;
DROP TABLE IF EXISTS tests;
DROP INDEX IF EXISTS testees_email_trgm_idx;
DROP INDEX IF EXISTS testees_last_name_trgm_idx;
DROP INDEX IF EXISTS testees_first_name_trgm_idx;
DROP TABLE IF EXISTS testees;
DROP INDEX IF EXISTS users_email_idx;
DROP TABLE IF EXISTS users;
DROP EXTENSION IF EXISTS pg_trgm;
DROP EXTENSION IF EXISTS "uuid-ossp";
