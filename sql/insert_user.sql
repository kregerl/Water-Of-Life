INSERT INTO users (
        user_id,
        preferred_username,
        email,
        refresh_token_version
    )
VALUES (?, ?, ?, ?) ON CONFLICT(user_id) DO NOTHING;