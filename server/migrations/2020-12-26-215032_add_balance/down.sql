-- This file should undo anything in `up.sql`
ALTER TABLE person_squad_connection
DROP COLUMN balance_cents;
