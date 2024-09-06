INSERT INTO spirits(uuid, name, description, distiller, abv)
VALUES ($1, $2, $3, $4, $5) ON CONFLICT(uuid) DO NOTHING;