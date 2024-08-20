SELECT uts.scope
FROM user_to_scopes uts
WHERE uts.user_id = $1;