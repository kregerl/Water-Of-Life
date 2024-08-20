CREATE TABLE IF NOT EXISTS scopes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scope TEXT NOT NULL
);
INSERT INTO scopes(scope)
VALUES ('a_spirit_add'),
    ('a_spirit_edit');