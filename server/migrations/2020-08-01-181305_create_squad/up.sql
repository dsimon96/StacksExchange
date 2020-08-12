-- Your SQL goes here
CREATE TABLE squad (
    id SERIAL PRIMARY KEY,
    node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
    display_name VARCHAR(50) NOT NULL
);
CREATE TABLE person_squad_connection (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    squad_id INTEGER NOT NULL REFERENCES squad(id) ON DELETE CASCADE
);
CREATE INDEX ON squad ( node_id );
CREATE INDEX ON person_squad_connection ( person_id, id, squad_id );
CREATE INDEX ON person_squad_connection ( squad_id, id, person_id );
ALTER TYPE node_type ADD VALUE IF NOT EXISTS 'squad';
