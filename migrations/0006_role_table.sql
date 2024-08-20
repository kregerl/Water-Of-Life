CREATE TABLE IF NOT EXISTS roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    scope_id INTEGER NOT NULL
);
INSERT INTO roles(name, scope_id)
VALUES ('Admin', 1),
    ('Admin', 2);