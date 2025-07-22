CREATE TABLE laptop_details (
    id SERIAL PRIMARY KEY,
    brand_name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    model_year INTEGER,
    product_type TEXT,
    cost_price NUMERIC(10, 2) NOT NULL,
    show_price NUMERIC(10, 2) GENERATED ALWAYS AS (cost_price + cost_price * 0.18) STORED,
    ram INTEGER,
    ram_type TEXT,
    storage INTEGER,
    storage_type TEXT,
    processor TEXT,
    processor_series TEXT,
    graphic_ram INTEGER,
    graphic TEXT,
    warranty TEXT,
    display TEXT,
    display_type TEXT,
    face_image BYTEA,
    quantity INTEGER DEFAULT 0,
    touchscreen BOOLEAN
);

CREATE TABLE laptop_side_images (
    id SERIAL PRIMARY KEY,
    laptop_id INTEGER REFERENCES laptop_details(id) ON DELETE CASCADE,
    image BYTEA
);