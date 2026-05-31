-- Session: frequently queried by user_id and scheduled_at
DEFINE INDEX IF NOT EXISTS idx_session_user_id ON TABLE session COLUMNS user_id;
DEFINE INDEX IF NOT EXISTS idx_session_scheduled_at ON TABLE session COLUMNS scheduled_at;

-- Group invitation: queried by invitee_id and group_id
DEFINE INDEX IF NOT EXISTS idx_invitation_invitee_id ON TABLE invitation COLUMNS invitee_id;
DEFINE INDEX IF NOT EXISTS idx_invitation_group_id ON TABLE invitation COLUMNS group_id;

-- Session invitation: queried by invitee_id and session_id
DEFINE INDEX IF NOT EXISTS idx_session_invitation_invitee_id ON TABLE session_invitation COLUMNS invitee_id;
DEFINE INDEX IF NOT EXISTS idx_session_invitation_session_id ON TABLE session_invitation COLUMNS session_id;

-- Friend request: queried in both directions
DEFINE INDEX IF NOT EXISTS idx_friend_request_from_id ON TABLE friend_request COLUMNS from_id;
DEFINE INDEX IF NOT EXISTS idx_friend_request_to_id ON TABLE friend_request COLUMNS to_id;

-- Invite: code lookups must be unique
DEFINE INDEX IF NOT EXISTS idx_invite_code ON TABLE invite COLUMNS code UNIQUE;
