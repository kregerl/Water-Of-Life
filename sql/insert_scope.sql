INSERT INTO scopes(scope)
VALUES ($1) ON CONFLICT(scope) DO NOTHING;