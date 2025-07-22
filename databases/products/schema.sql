CREATE TABLE IF NOT EXISTS laptop_details (
    id SERIAL PRIMARY KEY,
    brand_name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    model_year INTEGER,
    display_name TEXT,
    product_type TEXT,
    product_authetication TEXT,
    suitable_for TEXT,
    color TEXT,
    processor_generation TEXT,
    processor TEXT,
    processor_series TEXT,
    ram INTEGER,
    ram_type TEXT,
    storage INTEGER,
    storage_type TEXT,
    graphic TEXT,
    graphic_ram INTEGER,
    display TEXT,
    display_type TEXT,
    touchscreen BOOLEAN,
    power_supply TEXT,
    battery TEXT,
    warranty TEXT,
    cost_price NUMERIC(10, 2) NOT NULL,
    show_price NUMERIC(10, 2) GENERATED ALWAYS AS (cost_price + cost_price * 0.18) STORED,
    face_image_url TEXT, 
    quantity INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS laptop_side_images (
    id SERIAL PRIMARY KEY,
    laptop_id INTEGER REFERENCES laptop_details(id) ON DELETE CASCADE,
    image_url TEXT  
);