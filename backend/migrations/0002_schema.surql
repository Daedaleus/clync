-- ── user ──────────────────────────────────────────────────────────────────────
-- Record ID is the keycloak_id: user:<keycloak_id>
DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS keycloak_id ON TABLE user TYPE string;
DEFINE FIELD IF NOT EXISTS username    ON TABLE user TYPE string;
DEFINE FIELD IF NOT EXISTS email       ON TABLE user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS games       ON TABLE user TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS friends     ON TABLE user TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS steam_handle      ON TABLE user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS steam_visibility  ON TABLE user TYPE string  DEFAULT 'public';
DEFINE FIELD IF NOT EXISTS discord_handle    ON TABLE user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS discord_visibility ON TABLE user TYPE string DEFAULT 'public';

-- ── game ──────────────────────────────────────────────────────────────────────
-- Record ID is the game name: game:<name>
DEFINE TABLE IF NOT EXISTS game SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name                  ON TABLE game TYPE string;
DEFINE FIELD IF NOT EXISTS description           ON TABLE game TYPE option<string>;
DEFINE FIELD IF NOT EXISTS genre                 ON TABLE game TYPE option<string>;
DEFINE FIELD IF NOT EXISTS thumbnail_b64         ON TABLE game TYPE option<string>;
DEFINE FIELD IF NOT EXISTS thumbnail_content_type ON TABLE game TYPE option<string>;

-- ── group ─────────────────────────────────────────────────────────────────────
DEFINE TABLE IF NOT EXISTS `group` SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name           ON TABLE `group` TYPE string;
DEFINE FIELD IF NOT EXISTS is_public      ON TABLE `group` TYPE bool   DEFAULT false;
DEFINE FIELD IF NOT EXISTS members        ON TABLE `group` TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS creator_id     ON TABLE `group` TYPE option<string>;
DEFINE FIELD IF NOT EXISTS discord_invite ON TABLE `group` TYPE option<string>;

-- ── session ───────────────────────────────────────────────────────────────────
DEFINE TABLE IF NOT EXISTS session SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user_id       ON TABLE session TYPE string;
DEFINE FIELD IF NOT EXISTS username      ON TABLE session TYPE string;
DEFINE FIELD IF NOT EXISTS game          ON TABLE session TYPE string;
DEFINE FIELD IF NOT EXISTS scheduled_at  ON TABLE session TYPE datetime;
DEFINE FIELD IF NOT EXISTS scope         ON TABLE session TYPE string;
DEFINE FIELD IF NOT EXISTS group_ids     ON TABLE session TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS notes         ON TABLE session TYPE option<string>;
DEFINE FIELD IF NOT EXISTS participants  ON TABLE session TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS start_notified ON TABLE session TYPE bool DEFAULT false;
-- rsvps is an array of {user_id, username, status} objects
DEFINE FIELD IF NOT EXISTS rsvps         ON TABLE session TYPE array DEFAULT [];
DEFINE FIELD IF NOT EXISTS rsvps[*]      ON TABLE session FLEXIBLE TYPE object;

-- ── invitation (group invitations) ────────────────────────────────────────────
DEFINE TABLE IF NOT EXISTS invitation SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS group_id         ON TABLE invitation TYPE string;
DEFINE FIELD IF NOT EXISTS group_name       ON TABLE invitation TYPE string;
DEFINE FIELD IF NOT EXISTS inviter_id       ON TABLE invitation TYPE string;
DEFINE FIELD IF NOT EXISTS inviter_username ON TABLE invitation TYPE string;
DEFINE FIELD IF NOT EXISTS invitee_id       ON TABLE invitation TYPE string;
DEFINE FIELD IF NOT EXISTS created_at       ON TABLE invitation TYPE datetime DEFAULT time::now();

-- ── session_invitation ────────────────────────────────────────────────────────
DEFINE TABLE IF NOT EXISTS session_invitation SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS session_id       ON TABLE session_invitation TYPE string;
DEFINE FIELD IF NOT EXISTS game             ON TABLE session_invitation TYPE string;
DEFINE FIELD IF NOT EXISTS scheduled_at     ON TABLE session_invitation TYPE string;
DEFINE FIELD IF NOT EXISTS inviter_id       ON TABLE session_invitation TYPE string;
DEFINE FIELD IF NOT EXISTS inviter_username ON TABLE session_invitation TYPE string;
DEFINE FIELD IF NOT EXISTS invitee_id       ON TABLE session_invitation TYPE string;
DEFINE FIELD IF NOT EXISTS created_at       ON TABLE session_invitation TYPE datetime DEFAULT time::now();

-- ── invite (join codes) ───────────────────────────────────────────────────────
DEFINE TABLE IF NOT EXISTS invite SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS code       ON TABLE invite TYPE string;
DEFINE FIELD IF NOT EXISTS created_by ON TABLE invite TYPE string;
DEFINE FIELD IF NOT EXISTS expires_at ON TABLE invite TYPE datetime;
DEFINE FIELD IF NOT EXISTS used       ON TABLE invite TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE invite TYPE datetime DEFAULT time::now();

-- ── friend_request ────────────────────────────────────────────────────────────
DEFINE TABLE IF NOT EXISTS friend_request SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS from_id       ON TABLE friend_request TYPE string;
DEFINE FIELD IF NOT EXISTS from_username ON TABLE friend_request TYPE string;
DEFINE FIELD IF NOT EXISTS to_id         ON TABLE friend_request TYPE string;

-- ── push_subscription ─────────────────────────────────────────────────────────
-- Record ID is the user keycloak_id: push_subscription:<keycloak_id>
DEFINE TABLE IF NOT EXISTS push_subscription SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user_id  ON TABLE push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS endpoint ON TABLE push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS p256dh   ON TABLE push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS auth     ON TABLE push_subscription TYPE string;
