-- Your SQL goes here
CREATE TABLE squad (
    id SERIAL PRIMARY KEY,
    node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
    display_name VARCHAR(50) NOT NULL
);
CREATE INDEX ON squad ( node_id );
ALTER TYPE node_type ADD VALUE IF NOT EXISTS 'squad';
