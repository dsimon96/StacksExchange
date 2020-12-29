-- Your SQL goes here
CREATE TABLE balance (
    id SERIAL PRIMARY KEY,
    node_id INTEGER NOT NULL UNIQUE REFERENCES node(id) ON DELETE CASCADE,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    squad_id INTEGER NOT NULL REFERENCES squad(id) ON DELETE CASCADE,
    UNIQUE(person_id, squad_id)
);
CREATE INDEX ON balance ( node_id );
CREATE INDEX ON balance ( person_id, id, squad_id );
CREATE INDEX ON balance ( squad_id, id, person_id );
ALTER TYPE node_type ADD VALUE IF NOT EXISTS 'balance';
