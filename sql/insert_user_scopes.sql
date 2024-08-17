INSERT INTO user_scopes(user_id, scope_id)
VALUES (?, ?) ON CONFLICT(user_id, scope_id) DO NOTHING;