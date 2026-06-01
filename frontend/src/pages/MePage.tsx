import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import Badge from '../components/atoms/Badge';
import Button from '../components/atoms/Button';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import GameCard from '../components/molecules/GameCard';
import SessionList from '../components/organisms/SessionList';
import PageLayout from '../components/templates/PageLayout';
import type { Game, GroupSummary, Invitation, RsvpStatus, Session, SessionInvitation } from '../types';
import { fmtDateTime } from '../utils/date';
import { api } from '../services/api';
import { isAbortError } from '../utils/abort';
import { isPast, isLateJoinable } from '../utils/date';


interface FriendRequest {
  id: string;
  from_id: string;
  from_username: string;
}

interface MeData {
  username: string;
  games: string[];
  groups: GroupSummary[];
}

export default function MePage() {
  const { t } = useTranslation();
  const [me, setMe] = useState<MeData | null>(null);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [invitations, setInvitations] = useState<Invitation[]>([]);
  const [sessionInvitations, setSessionInvitations] = useState<SessionInvitation[]>([]);
  const [friendRequests, setFriendRequests] = useState<FriendRequest[]>([]);
  const [library, setLibrary] = useState<Map<string, Game>>(new Map());
  const [error, setError] = useState<string | null>(null);

  function greeting(name: string): string {
    const h = new Date().getHours();
    if (h < 5) return t('me.greeting_night', { name });
    if (h < 12) return t('me.greeting_morning', { name });
    if (h < 18) return t('me.greeting_day', { name });
    return t('me.greeting_evening', { name });
  }

  useEffect(() => {
    const controller = new AbortController();
    api.get<MeData>('/api/v1/me', controller.signal)
      .then(setMe)
      .catch((err: unknown) => { if (!isAbortError(err)) setError(t('me.load_error')); });
    api.get<Session[]>('/api/v1/sessions/mine', controller.signal)
      .then(setSessions)
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    api.get<Invitation[]>('/api/v1/invitations', controller.signal)
      .then(setInvitations)
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    api.get<SessionInvitation[]>('/api/v1/session-invitations', controller.signal)
      .then(setSessionInvitations)
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    api.get<FriendRequest[]>('/api/v1/friends/requests', controller.signal)
      .then(setFriendRequests)
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    api.get<Game[]>('/api/v1/library', controller.signal)
      .then((gs) => setLibrary(new Map(gs.map((g) => [g.name, g]))))
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    return () => controller.abort();
  }, [t]);

  const upcomingSessions = useMemo(
    () => sessions.filter((s) => !isPast(s.scheduled_at) || isLateJoinable(s.scheduled_at)).slice(0, 3),
    [sessions],
  );

  const handleRsvp = async (id: string, status: RsvpStatus | null) => {
    try {
      if (status === null) {
        await api.delete(`/api/v1/sessions/${id}/rsvp`);
        setSessions((p) => p.map((s) => s.id === id
          ? { ...s, my_rsvp: null, is_participant: false, participant_count: s.is_participant ? s.participant_count - 1 : s.participant_count }
          : s));
      } else {
        await api.put(`/api/v1/sessions/${id}/rsvp`, { status });
        setSessions((p) => p.map((s) => {
          if (s.id !== id) return s;
          const wasParticipant = s.is_participant;
          const willBeParticipant = status === 'accepted';
          return {
            ...s,
            my_rsvp: status,
            is_participant: willBeParticipant,
            participant_count:
              willBeParticipant && !wasParticipant ? s.participant_count + 1
              : !willBeParticipant && wasParticipant ? s.participant_count - 1
              : s.participant_count,
          };
        }));
      }
    } catch { /* handled by SessionRow */ }
  };

  const handleAccept = async (id: string, groupId: string) => {
    try {
      await api.post(`/api/v1/invitations/${id}/accept`);
      setInvitations((p) => p.filter((inv) => inv.id !== id));
      setMe((m) => m && {
        ...m,
        groups: m.groups.some((g) => g.id === groupId)
          ? m.groups
          : [...m.groups, { id: groupId, name: invitations.find((i) => i.id === id)?.group_name ?? '', is_public: false }],
      });
    } catch { /* silently ignore */ }
  };

  const handleDecline = async (id: string) => {
    try {
      await api.delete(`/api/v1/invitations/${id}`);
      setInvitations((p) => p.filter((inv) => inv.id !== id));
    } catch { /* silently ignore */ }
  };

  const handleAcceptFriend = async (req: FriendRequest) => {
    try {
      await api.post(`/api/v1/friends/requests/${req.id}/accept`);
      setFriendRequests((p) => p.filter((r) => r.id !== req.id));
    } catch { /* silently ignore */ }
  };

  const handleAcceptSessionInvitation = async (id: string) => {
    try {
      await api.post(`/api/v1/session-invitations/${id}/accept`);
      setSessionInvitations((p) => p.filter((inv) => inv.id !== id));
      api.get<Session[]>('/api/v1/sessions/mine').then(setSessions).catch(() => {});
    } catch { /* silently ignore */ }
  };

  const handleDeclineSessionInvitation = async (id: string) => {
    try {
      await api.delete(`/api/v1/session-invitations/${id}`);
      setSessionInvitations((p) => p.filter((inv) => inv.id !== id));
    } catch { /* silently ignore */ }
  };

  const handleDeclineFriend = async (reqId: string) => {
    try {
      await api.delete(`/api/v1/friends/requests/${reqId}`);
      setFriendRequests((p) => p.filter((r) => r.id !== reqId));
    } catch { /* silently ignore */ }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.delete(`/api/v1/sessions/${id}`);
      setSessions((p) => p.filter((s) => s.id !== id));
    } catch { /* handled by SessionRow */ }
  };

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}
      {!me && !error && <p className="text-zinc-500 text-sm">{t('me.loading')}</p>}

      {me && (
        <>
          {/* Greeting */}
          <div className="py-1">
            <h1 className="text-2xl font-bold text-zinc-100" data-testid="greeting">{greeting(me.username)}</h1>
          </div>

          {/* Pending invitations */}
          {invitations.length > 0 && (
            <section className="space-y-2">
              <SectionLabel>{t('me.pending_invitations')}</SectionLabel>
              <ul className="space-y-2">
                {invitations.map((inv) => (
                  <li key={inv.id} className="bg-violet-500/10 border border-violet-500/30 rounded-xl px-4 py-3 flex items-center justify-between gap-4">
                    <div className="min-w-0">
                      <p className="text-sm font-medium text-zinc-100 truncate">{inv.group_name}</p>
                      <p className="text-xs text-zinc-400">{t('me.invited_by', { name: inv.inviter_username })}</p>
                    </div>
                    <div className="shrink-0 flex gap-2">
                      <Button size="sm" variant="primary" onClick={() => handleAccept(inv.id, inv.group_id)}>
                        {t('common.accept')}
                      </Button>
                      <Button size="sm" variant="secondary" onClick={() => handleDecline(inv.id)}>
                        {t('common.decline')}
                      </Button>
                    </div>
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Pending session invitations */}
          {sessionInvitations.length > 0 && (
            <section className="space-y-2">
              <SectionLabel>{t('me.session_invitations')}</SectionLabel>
              <ul className="space-y-2">
                {sessionInvitations.map((inv) => (
                  <li key={inv.id} className="bg-amber-500/10 border border-amber-500/30 rounded-xl px-4 py-3 flex items-center justify-between gap-4">
                    <div className="min-w-0">
                      <p className="text-sm font-medium text-zinc-100 truncate">{inv.game}</p>
                      <p className="text-xs text-zinc-400">
                        {t('me.session_invitation_meta', { datetime: fmtDateTime(inv.scheduled_at), name: inv.inviter_username })}
                      </p>
                    </div>
                    <div className="shrink-0 flex gap-2">
                      <Button size="sm" variant="primary" onClick={() => handleAcceptSessionInvitation(inv.id)}>
                        {t('common.accept')}
                      </Button>
                      <Button size="sm" variant="secondary" onClick={() => handleDeclineSessionInvitation(inv.id)}>
                        {t('common.decline')}
                      </Button>
                    </div>
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Pending friend requests */}
          {friendRequests.length > 0 && (
            <section className="space-y-2">
              <SectionLabel>{t('me.friend_requests')}</SectionLabel>
              <ul className="space-y-2">
                {friendRequests.map((req) => (
                  <li key={req.id} className="bg-emerald-500/10 border border-emerald-500/30 rounded-xl px-4 py-3 flex items-center justify-between gap-4">
                    <div className="min-w-0">
                      <p className="text-sm font-medium text-zinc-100 truncate">{req.from_username}</p>
                      <p className="text-xs text-zinc-400">{t('me.friend_request_subtitle')}</p>
                    </div>
                    <div className="shrink-0 flex gap-2">
                      <Button size="sm" variant="primary" onClick={() => handleAcceptFriend(req)}>
                        {t('common.accept')}
                      </Button>
                      <Button size="sm" variant="secondary" onClick={() => handleDeclineFriend(req.id)}>
                        {t('common.decline')}
                      </Button>
                    </div>
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Upcoming sessions */}
          <section className="space-y-2">
            <div className="flex items-center justify-between">
              <SectionLabel data-testid="section-upcoming-sessions">{t('me.upcoming_sessions')}</SectionLabel>
              <Link to="/sessions" className="text-xs text-zinc-500 hover:text-violet-400 transition-colors">
                {t('common.view_all')}
              </Link>
            </div>
            {upcomingSessions.length === 0 ? (
              <div className="bg-ui-surface border border-ui-border rounded-xl px-4 py-6 text-center space-y-2">
                <p className="text-sm text-zinc-500">{t('me.no_upcoming_sessions')}</p>
                <Link to="/sessions" className="text-sm text-violet-400 hover:text-violet-300 transition-colors">
                  {t('me.create_session_cta')}
                </Link>
              </div>
            ) : (
              <SessionList sessions={upcomingSessions} onRsvp={handleRsvp} onDelete={handleDelete} />
            )}
          </section>

          {/* Groups */}
          <section className="space-y-2">
            <div className="flex items-center justify-between">
              <SectionLabel data-testid="section-your-groups">{t('me.your_groups')}</SectionLabel>
              <Link to="/groups" className="text-xs text-zinc-500 hover:text-violet-400 transition-colors">
                {t('common.view_all')}
              </Link>
            </div>
            {me.groups.length === 0 ? (
              <div className="bg-ui-surface border border-ui-border rounded-xl px-4 py-6 text-center space-y-2">
                <p className="text-sm text-zinc-500">{t('me.no_groups')}</p>
                <Link to="/groups" className="text-sm text-violet-400 hover:text-violet-300 transition-colors">
                  {t('me.discover_groups_cta')}
                </Link>
              </div>
            ) : (
              <ul className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {me.groups.map((g) => (
                  <li key={g.id}>
                    <Link
                      to={`/groups/${g.id}`}
                      className="flex items-center justify-between gap-3 bg-ui-surface border border-ui-border rounded-xl px-4 py-3 hover:border-ui-border hover:bg-ui-raised/50 transition-colors"
                    >
                      <span className="text-sm font-medium text-zinc-100 truncate">{g.name}</span>
                      <Badge variant={g.is_public ? 'public' : 'private'} />
                    </Link>
                  </li>
                ))}
              </ul>
            )}
          </section>

          {/* Games */}
          <section className="space-y-2">
            <div className="flex items-center justify-between">
              <SectionLabel data-testid="section-your-games">{t('me.your_games')}</SectionLabel>
              <Link to="/profile" className="text-xs text-zinc-500 hover:text-violet-400 transition-colors">
                {t('me.edit_games_cta')}
              </Link>
            </div>
            {me.games.length === 0 ? (
              <div className="bg-ui-surface border border-ui-border rounded-xl px-4 py-6 text-center space-y-2">
                <p className="text-sm text-zinc-500">{t('me.no_games')}</p>
                <Link to="/profile" className="text-sm text-violet-400 hover:text-violet-300 transition-colors">
                  {t('me.add_games_cta')}
                </Link>
              </div>
            ) : (
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {me.games.map((name) => (
                  <GameCard
                    key={name}
                    game={library.get(name) ?? { name }}
                  />
                ))}
              </div>
            )}
          </section>
        </>
      )}
    </PageLayout>
  );
}
