SELECT uuid AS 'uuid: String',
    name AS 'name: String',
    distiller AS 'distiller: String',
    bottler AS 'bottler: String',
    type AS 'typ: String'
FROM spirits_fts
WHERE name MATCH $1
ORDER BY name DESC
LIMIT 20;