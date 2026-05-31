-- 0003 failed silently: SCHEMAFULL TYPE string rejects NONE even in WHERE clauses.
-- Solution: temporarily redefine the fields as option<string> (NONE becomes valid),
-- backfill, then restore strict TYPE string.

DEFINE FIELD discord_visibility ON TABLE user TYPE option<string> DEFAULT 'public';
DEFINE FIELD steam_visibility   ON TABLE user TYPE option<string> DEFAULT 'public';

UPDATE user SET discord_visibility = 'public' WHERE discord_visibility = NONE;
UPDATE user SET steam_visibility   = 'public' WHERE steam_visibility   = NONE;

DEFINE FIELD discord_visibility ON TABLE user TYPE string DEFAULT 'public';
DEFINE FIELD steam_visibility   ON TABLE user TYPE string DEFAULT 'public';
