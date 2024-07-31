-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

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

CREATE TABLE tests (
    id SERIAL PRIMARY KEY,
    testee_id INTEGER NOT NULL,
    FOREIGN KEY (testee_id) REFERENCES testees(id),
    role VARCHAR(10) CHECK (role IN ('leader', 'follower')) NOT NULL,
    test_date TIMESTAMP NOT NULL
);

CREATE TABLE patterns (
    id SERIAL PRIMARY KEY,
    test_id INTEGER NOT NULL,
    pattern TEXT NOT NULL,
    category TEXT NOT NULL,
    score INTEGER NOT NULL,
    FOREIGN KEY (test_id) REFERENCES tests(id)
);

CREATE TABLE techniques (
    id SERIAL PRIMARY KEY,
    test_id INTEGER NOT NULL,
    technique TEXT NOT NULL,
    score INTEGER NOT NULL,
    score_header TEXT NOT NULL,
    FOREIGN KEY (test_id) REFERENCES tests(id)
);

CREATE TABLE bonus_points (
    id SERIAL PRIMARY KEY,
    test_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    score INTEGER NOT NULL,
    FOREIGN KEY (test_id) REFERENCES tests(id)
);

