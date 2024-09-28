-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable pg_trgm extension
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Set similarity threshold (optional, adjust as needed)
SET pg_trgm.similarity_threshold = 0.3;

CREATE TABLE
    "users" (
        id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
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
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
);

-- Create trigram indexes
CREATE INDEX testees_first_name_trgm_idx ON testees USING gin (first_name gin_trgm_ops);
CREATE INDEX testees_last_name_trgm_idx ON testees USING gin (last_name gin_trgm_ops);
CREATE INDEX testees_email_trgm_idx ON testees USING gin (email gin_trgm_ops);


CREATE TABLE tests (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4())
);

CREATE TABLE test_metadata (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),       
    test_id UUID NOT NULL REFERENCES tests(id), 
    test_name VARCHAR NOT NULL,    
    minimum_percent REAL NOT NULL,   
    max_score INTEGER NOT NULL,
    achieved_score INTEGER NOT NULL,             
    testee_id UUID NOT NULL REFERENCES testees(id),              
    test_date TIMESTAMP NOT NULL,
    is_passing BOOLEAN NOT NULL,
    proctor_id UUID NOT NULL REFERENCES users(id),
    failure_explanation TEXT[]                 
);

-- TestTable (stores multiple tables associated with a test)
CREATE TABLE test_tables (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    test_id UUID NOT NULL REFERENCES tests(id),
    insert_counter SERIAL
);

-- TestSection (stores multiple sections under a table)
CREATE TABLE test_sections (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    table_id UUID REFERENCES test_tables(id), 
    name VARCHAR NOT NULL,
    insert_counter SERIAL
);

-- ScoringCategory (stores scoring categories for sections)
CREATE TABLE scoring_categories (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    section_id UUID REFERENCES test_sections(id), 
    name VARCHAR NOT NULL,
    values TEXT[] NOT NULL,
    insert_counter SERIAL
);

CREATE TABLE competencies (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    section_id UUID NOT NULL REFERENCES test_sections(id),  
    name VARCHAR NOT NULL,
    scores JSONB NOT NULL,                      -- Array of scores (2D array for Vec<Vec<i32>>)
    subtext TEXT,
    antithesis TEXT,
    achieved_scores JSONB NOT NULL,                   
    achieved_score_labels JSONB NOT NULL,                    
    failing_score_labels JSONB NOT NULL,   
    insert_counter SERIAL
);

CREATE TABLE bonus_items (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    test_id UUID NOT NULL REFERENCES tests(id), 
    name VARCHAR NOT NULL,
    score INTEGER NOT NULL,
    achieved BOOLEAN NOT NULL,                
    insert_counter SERIAL
);

CREATE TABLE queue (
    testee_id UUID NOT NULL REFERENCES testees(id),
    test_definition_index INTEGER NOT NULL,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (testee_id, test_definition_index)
);