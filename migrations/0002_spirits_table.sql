CREATE TABLE IF NOT EXISTS spirits (
    uuid TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    distiller TEXT NOT NULL,
    bottler TEXT NOT NULL,
    type TEXT NOT NULL,
    abv REAL NOT NULL,
    age TEXT NOT NULL
);
CREATE VIRTUAL TABLE spirits_fts USING fts5(uuid, name, distiller, bottler, type);
ATTACH DATABASE './spirit_database/bottleraiders/whiskey.db' AS whiskeys;
INSERT INTO spirits (
        uuid,
        name,
        description,
        distiller,
        bottler,
        type,
        abv,
        age
    )
SELECT image_uuid,
    title,
    '',
    distiller,
    bottler,
    type,
    abv,
    age
FROM whiskeys.whiskey;
INSERT INTO spirits_fts (uuid, name, distiller, bottler, type)
SELECT image_uuid,
    title,
    distiller,
    bottler,
    type
FROM whiskeys.whiskey;