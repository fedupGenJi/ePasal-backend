CREATE TABLE IF NOT EXISTS khalti_temp_payments (
    id SERIAL PRIMARY KEY,
    pidx TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    laptop_id TEXT NOT NULL
);