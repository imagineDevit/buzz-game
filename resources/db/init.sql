CREATE TABLE  IF NOT EXISTS players
(
    id  VARCHAR(255) PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL UNIQUE,
    score OID NOT NULL
)