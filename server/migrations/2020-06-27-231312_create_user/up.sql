-- Your SQL goes here
CREATE TABLE account (
    id UUID PRIMARY KEY,
    display_name VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(320) NOT NULL UNIQUE,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL
);