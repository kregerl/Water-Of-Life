INSERT INTO user_scopes(user_id, scope_id)
VALUES ($1, $2) ON CONFLICT(user_id, scope_id) DO NOTHING;