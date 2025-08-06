CREATE TABLE IF NOT EXISTS laptops_sold (
    sale_id SERIAL PRIMARY KEY,        
    laptop_id INTEGER NOT NULL,            
    sold_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    quantity INTEGER NOT NULL DEFAULT 1,
    price_at_sale NUMERIC(10, 2) NOT NULL,
    
    CONSTRAINT fk_laptop
        FOREIGN KEY (laptop_id)
        REFERENCES laptop_details(id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);