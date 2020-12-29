-- Your SQL goes here
CREATE TABLE txn (
    id SERIAL PRIMARY KEY,
    node_id INTEGER NOT NULL UNIQUE REFERENCES node(id) ON DELETE CASCADE,
    squad_id INTEGER NOT NULL REFERENCES squad(id) ON DELETE CASCADE
);
CREATE INDEX ON txn ( node_id );
CREATE INDEX ON txn ( squad_id, id );
ALTER TYPE node_type ADD VALUE IF NOT EXISTS 'txn';

CREATE TABLE txn_part (
    id SERIAL PRIMARY KEY,
    txn_id INTEGER NOT NULL REFERENCES txn(id) ON DELETE CASCADE,
    balance_id INTEGER NOT NULL REFERENCES balance(id) ON DELETE CASCADE,
    balance_change_cents INTEGER NOT NULL,
    UNIQUE(txn_id, balance_id)
);
CREATE INDEX ON txn_part ( txn_id, balance_id, balance_change_cents );
CREATE INDEX ON txn_part ( balance_id, txn_id, balance_change_cents );
