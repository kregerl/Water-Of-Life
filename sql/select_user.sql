SELECT user_id,
    preferred_username,
    email,
    refresh_token_version,
    role
FROM users
WHERE user_id = ?;