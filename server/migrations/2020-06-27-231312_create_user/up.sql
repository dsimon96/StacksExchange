-- Your SQL goes here
CREATE TABLE node (
    id SERIAL PRIMARY KEY,
    uid UUID NOT NULL UNIQUE
);
CREATE TABLE person (
    id SERIAL PRIMARY KEY,
    node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
    display_name VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(320) NOT NULL UNIQUE,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL
);
CREATE INDEX ON node ( id, uid );
CREATE INDEX ON node ( uid, id );
CREATE INDEX ON person ( node_id );
