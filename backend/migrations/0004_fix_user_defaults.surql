-- 0003 failed silently: SCHEMAFULL TYPE string/array rejects NONE even during WHERE
-- evaluation. Fix: temporarily widen all constrained fields to option<T>, backfill,
-- then restore strict types. Uses OVERWRITE to replace existing field definitions.

-- Step 1: allow NONE temporarily
DEFINE FIELD OVERWRITE discord_visibility ON TABLE user TYPE option<string>         DEFAULT 'public';
DEFINE FIELD OVERWRITE steam_visibility   ON TABLE user TYPE option<string>         DEFAULT 'public';
DEFINE FIELD OVERWRITE games              ON TABLE user TYPE option<array<string>>  DEFAULT [];
DEFINE FIELD OVERWRITE friends            ON TABLE user TYPE option<array<string>>  DEFAULT [];

-- Step 2: backfill missing values on existing records
UPDATE user SET discord_visibility = 'public' WHERE discord_visibility = NONE;
UPDATE user SET steam_visibility   = 'public' WHERE steam_visibility   = NONE;
UPDATE user SET games              = []        WHERE games              = NONE;
UPDATE user SET friends            = []        WHERE friends            = NONE;

-- Step 3: restore strict types
DEFINE FIELD OVERWRITE discord_visibility ON TABLE user TYPE string         DEFAULT 'public';
DEFINE FIELD OVERWRITE steam_visibility   ON TABLE user TYPE string         DEFAULT 'public';
DEFINE FIELD OVERWRITE games              ON TABLE user TYPE array<string>  DEFAULT [];
DEFINE FIELD OVERWRITE friends            ON TABLE user TYPE array<string>  DEFAULT [];
