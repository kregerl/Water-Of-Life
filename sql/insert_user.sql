INSERT INTO users (
        user_id,
        preferred_username,
        email,
        refresh_token_version,
        role
    )
VALUES ($1, $2, $3, $4, $5) ON CONFLICT(user_id) DO NOTHING;