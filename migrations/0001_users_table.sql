CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY NOT NULL,
    preferred_username TEXT NOT NULL,
    email TEXT NOT NULL,
    refresh_token_version INTEGER NOT NULL
);