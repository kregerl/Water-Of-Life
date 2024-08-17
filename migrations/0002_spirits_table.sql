CREATE TABLE IF NOT EXISTS spirits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    distiller TEXT NOT NULL,
    abv REAL NOT NULL,
    image_id uuid NOT NULL
);