//! Typed error-code constants — every value must match a key in the frontend i18n files.
//! Use these instead of string literals to get compile-time typo checking.

pub mod game {
    pub const NAME_REQUIRED: &str = "error.game.name_required";
    pub const RAWG_NOT_CONFIGURED: &str = "error.game.rawg_not_configured";
    pub const NOT_FOUND: &str = "error.game.not_found";
    pub const THUMBNAIL_TOO_LARGE: &str = "error.game.thumbnail_too_large";
    pub const IN_WISHLIST: &str = "error.game.in_wishlist";
}

pub mod session {
    pub const DATE_REQUIRED: &str = "error.session.date_required";
    pub const INVALID_DATE: &str = "error.session.invalid_date";
    pub const DATE_IN_PAST: &str = "error.session.date_in_past";
    pub const GROUPS_REQUIRED: &str = "error.session.groups_required";
    pub const NOTES_TOO_LONG: &str = "error.session.notes_too_long";
    pub const NOT_FOUND: &str = "error.session.not_found";
    pub const NO_LONGER_AVAILABLE: &str = "error.session.no_longer_available";
}

pub mod group {
    pub const NAME_REQUIRED: &str = "error.group.name_required";
    pub const NOT_FOUND: &str = "error.group.not_found";
    pub const INVALID_DISCORD_URL: &str = "error.group.invalid_discord_url";
    pub const DISCORD_URL_TOO_LONG: &str = "error.group.discord_url_too_long";
    pub const DISCORD_URL_UNAUTHORIZED: &str = "error.group.discord_url_unauthorized";
    pub const DELETE_UNAUTHORIZED: &str = "error.group.delete_unauthorized";
}

pub mod invitation {
    pub const ONLY_MEMBERS_CAN_INVITE: &str = "error.invitation.only_members_can_invite";
    pub const ALREADY_MEMBER: &str = "error.invitation.already_member";
    pub const NOT_MUTUAL_FRIENDS: &str = "error.invitation.not_mutual_friends";
    pub const ALREADY_SENT: &str = "error.invitation.already_sent";
    pub const NOT_FOUND: &str = "error.invitation.not_found";
    pub const ONLY_MEMBERS_SEE_LIST: &str = "error.invitation.only_members_see_list";
}

pub mod session_invitation {
    pub const ONLY_PARTICIPANTS_CAN_INVITE: &str =
        "error.session_invitation.only_participants_can_invite";
    pub const ALREADY_PARTICIPANT: &str = "error.session_invitation.already_participant";
    pub const NOT_MUTUAL_FRIENDS: &str = "error.session_invitation.not_mutual_friends";
    pub const ONLY_PARTICIPANTS_SEE_LIST: &str =
        "error.session_invitation.only_participants_see_list";
}

pub mod friend_request {
    pub const SELF_REQUEST: &str = "error.friend_request.self_request";
    pub const ALREADY_FRIENDS: &str = "error.friend_request.already_friends";
    pub const ALREADY_PENDING: &str = "error.friend_request.already_pending";
    pub const NOT_FOUND: &str = "error.friend_request.not_found";
}

pub mod invite {
    pub const INVALID_OR_EXPIRED: &str = "error.invite.invalid_or_expired";
}

pub mod registration {
    pub const INVALID_CREDENTIALS: &str = "error.registration.invalid_credentials";
    pub const USERNAME_TAKEN: &str = "error.registration.username_taken";
}
