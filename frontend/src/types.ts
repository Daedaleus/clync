export type RsvpStatus = 'accepted' | 'maybe' | 'declined';

export interface Session {
  id: string;
  user_id: string;
  username: string;
  game: string;
  scheduled_at: string;
  scope: string;
  participant_count: number;
  is_mine: boolean;
  is_participant: boolean;
  my_rsvp: RsvpStatus | null;
  group_ids: string[];
  group_names: string[];
  thumbnail_url?: string | null;
  notes?: string | null;
}

export interface GroupSummary {
  id: string;
  name: string;
  is_public: boolean;
}

export interface Friend {
  keycloak_id: string;
  username: string;
}

export interface Game {
  name: string;
  description?: string | null;
  genre?: string | null;
  thumbnail_url?: string | null;
}

export interface Invitation {
  id: string;
  group_id: string;
  group_name: string;
  inviter_id: string;
  inviter_username: string;
}

export interface SessionInvitation {
  id: string;
  session_id: string;
  game: string;
  scheduled_at: string;
  inviter_id: string;
  inviter_username: string;
}
