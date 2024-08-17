CREATE VIEW user_to_scopes AS
SELECT us.user_id,
    s.id as scope_id,
    s.scope
FROM user_scopes us
    JOIN scopes s ON us.scope_id = s.id;