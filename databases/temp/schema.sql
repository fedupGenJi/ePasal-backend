CREATE TABLE IF NOT EXISTS temp_users (
    temp_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    number TEXT NOT NULL,
    gmail TEXT NOT NULL,
    password TEXT NOT NULL,
    code TEXT NOT NULL
);