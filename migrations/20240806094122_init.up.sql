-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable pg_trgm extension
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Set similarity threshold (optional, adjust as needed)
SET pg_trgm.similarity_threshold = 0.3;

CREATE TABLE
    "users" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        first_name VARCHAR(100) NOT NULL,
        last_name VARCHAR(100) NOT NULL,
        email VARCHAR(255) NOT NULL UNIQUE,
        password VARCHAR(100) NOT NULL,
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );

CREATE INDEX users_email_idx ON users (email);

CREATE TABLE testees (
    id SERIAL PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
);

-- Create trigram indexes
CREATE INDEX testees_first_name_trgm_idx ON testees USING gin (first_name gin_trgm_ops);
CREATE INDEX testees_last_name_trgm_idx ON testees USING gin (last_name gin_trgm_ops);
CREATE INDEX testees_email_trgm_idx ON testees USING gin (email gin_trgm_ops);


CREATE TABLE tests (
    id SERIAL PRIMARY KEY,
    testee_id INTEGER NOT NULL,
    FOREIGN KEY (testee_id) REFERENCES testees(id),
    role VARCHAR(10) CHECK (role IN ('leader', 'follower')) NOT NULL,
    test_date TIMESTAMP NOT NULL,
    score INTEGER NOT NULL,
    max_score INTEGER NOT NULL,
    passing_score INTEGER NOT NULL
);

CREATE TABLE patterns (
    id SERIAL PRIMARY KEY,
    test_id INTEGER NOT NULL,
    pattern TEXT NOT NULL,
    category TEXT NOT NULL,
    score INTEGER NOT NULL,
    max_score INTEGER NOT NULL,
    FOREIGN KEY (test_id) REFERENCES tests(id)
);

CREATE TABLE techniques (
    id SERIAL PRIMARY KEY,
    test_id INTEGER NOT NULL,
    technique TEXT NOT NULL,
    score INTEGER NOT NULL,
    score_header TEXT NOT NULL,
    max_score INTEGER NOT NULL,
    FOREIGN KEY (test_id) REFERENCES tests(id)
);

CREATE TABLE bonus_points (
    id SERIAL PRIMARY KEY,
    test_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    score INTEGER NOT NULL,
    FOREIGN KEY (test_id) REFERENCES tests(id)
);

CREATE TABLE queue (
    testee_id INT NOT NULL,
    role VARCHAR(10) CHECK (role IN ('leader', 'follower')) NOT NULL,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (testee_id, role),
    FOREIGN KEY (testee_id) REFERENCES testees(id)
);
