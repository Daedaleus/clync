-- Backfill default values for string fields on user records that existed
-- before the SCHEMAFULL schema (0002) was applied.
-- SurrealDB enforces TYPE string on read, so NONE values cause 500 errors.
UPDATE user SET discord_visibility = 'public' WHERE discord_visibility = NONE;
UPDATE user SET steam_visibility = 'public' WHERE steam_visibility = NONE;
