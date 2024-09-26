-- Add down migration script here

DROP TABLE IF EXISTS queue;
DROP TABLE IF EXISTS bonus_items;
DROP TABLE IF EXISTS competencies;
DROP TABLE IF EXISTS scoring_categories;
DROP TABLE IF EXISTS test_sections;
DROP TABLE IF EXISTS test_tables;
DROP TABLE IF EXISTS test_metadata;
DROP TABLE IF EXISTS tests;
DROP INDEX IF EXISTS testees_email_trgm_idx;
DROP INDEX IF EXISTS testees_last_name_trgm_idx;
DROP INDEX IF EXISTS testees_first_name_trgm_idx;
DROP TABLE IF EXISTS testees;
DROP INDEX IF EXISTS users_email_idx;
DROP TABLE IF EXISTS users;
DROP EXTENSION IF EXISTS pg_trgm;
DROP EXTENSION IF EXISTS "uuid-ossp";
