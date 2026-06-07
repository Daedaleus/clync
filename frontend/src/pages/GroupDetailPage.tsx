import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate, useParams } from 'react-router-dom';
import Avatar from '../components/atoms/Avatar';
import Badge from '../components/atoms/Badge';
import GameCard from '../components/molecules/GameCard';
import Button from '../components/atoms/Button';
import SectionLabel from '../components/atoms/SectionLabel';
import UserRow from '../components/molecules/UserRow';
import WeekCalendar from '../components/organisms/WeekCalendar';
import PageLayout from '../components/templates/PageLayout';
import type { Game, RsvpStatus, Session } from '../types';
import { api } from '../services/api';
import { isAbortError } from '../utils/abort';
import { useAuth } from 'react-oidc-context';
import { isAdmin } from '../utils/auth';
import ErrorBanner from '../components/molecules/ErrorBanner';
import { useNotifications } from '../hooks/useNotifications';
import { config } from '../config';

const API_BASE = config.apiUrl;

interface Member { keycloak_id: string; username: string; is_friend: boolean }
interface PossibleGame { name: string; count: number; total: number }
interface GroupDetail {
  id: string; name: string; is_public: boolean;
  creator_id: string | null; discord_invite: string | null;
  members: Member[]; common_games: string[]; possible_games: PossibleGame[];
}
interface InvitableUser { keycloak_id: string; username: string }

export default function GroupDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [group, setGroup] = useState<GroupDetail | null>(null);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [library, setLibrary] = useState<Map<string, Game>>(new Map());
  const [error, setError] = useState<string | null>(null);
  const [showInvite, setShowInvite] = useState(false);
  const [invitable, setInvitable] = useState<InvitableUser[]>([]);
  const [invitedIds, setInvitedIds] = useState<Set<string>>(new Set());
  const [showAllMembers, setShowAllMembers] = useState(false);
  const [showAllGames, setShowAllGames] = useState(false);

  const MEMBER_LIMIT = 10;
  const GAME_LIMIT = 10;
  const { notify } = useNotifications();
  const auth = useAuth();
  const myId = auth.user?.profile.sub;

  useEffect(() => {
    if (!id) return;
    const controller = new AbortController();
    api.get<GroupDetail>(`/api/v1/groups/${id}`, controller.signal).then(setGroup)
      .catch((err: unknown) => { if (!isAbortError(err)) setError(t('group_detail.not_found')); });
    api.get<Session[]>(`/api/v1/groups/${id}/sessions`, controller.signal).then(setSessions)
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    api.get<Game[]>('/api/v1/library', controller.signal)
      .then((gs) => setLibrary(new Map(gs.map((g) => [g.name, g]))))
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    return () => controller.abort();
  }, [id, t]);

  // SSE subscription for real-time session updates.
  useEffect(() => {
    const token = auth.user?.access_token;
    if (!id || !token) return;

    const url = `${API_BASE}/api/v1/groups/${id}/events?token=${encodeURIComponent(token)}`;
    const es = new EventSource(url);

    es.addEventListener('session_created', (e) => {
      const session: Session = JSON.parse(e.data);
      setSessions((prev) => {
        if (prev.some((s) => s.id === session.id)) return prev;
        return [...prev, session].sort((a, b) => a.scheduled_at.localeCompare(b.scheduled_at));
      });
      if (session.user_id !== myId) {
        notify(
          `Clync – ${session.game}`,
          `${session.username} möchte ${session.game} spielen`,
          `/groups/${id}`,
        );
      }
    });

    es.addEventListener('session_joined', (e) => {
      const { id: sessionId, participant_count }: { id: string; participant_count: number } = JSON.parse(e.data);
      setSessions((prev) =>
        prev.map((s) => (s.id === sessionId ? { ...s, participant_count } : s)),
      );
    });

    es.addEventListener('session_deleted', (e) => {
      const { id: sessionId }: { id: string } = JSON.parse(e.data);
      setSessions((prev) => prev.filter((s) => s.id !== sessionId));
    });

    return () => es.close();
    // Reconnects whenever the access token changes — react-oidc-context hands
    // out a new `user` object every time the token is renewed.
  }, [id, notify, myId, auth.user?.access_token]);

  const handleShowInvite = async () => {
    if (!id) return;
    if (!showInvite) {
      const users = await api.get<InvitableUser[]>(`/api/v1/groups/${id}/invitable`).catch(() => []);
      setInvitable(users);
    }
    setShowInvite((v) => !v);
  };

  const handleInvite = async (userId: string) => {
    if (!id) return;
    try {
      await api.post(`/api/v1/groups/${id}/invite`, { user_id: userId });
      setInvitedIds((p) => new Set([...p, userId]));
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleDeleteGroup = async () => {
    if (!id || !window.confirm(t('group_detail.delete_confirm', { name: group?.name }))) return;
    try {
      await api.delete(`/api/v1/groups/${id}`);
      navigate('/groups');
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleRsvp = async (sessionId: string, status: RsvpStatus | null) => {
    try {
      if (status === null) {
        await api.delete(`/api/v1/sessions/${sessionId}/rsvp`);
        setSessions((p) => p.map((s) => s.id === sessionId
          ? { ...s, my_rsvp: null, is_participant: false, participant_count: s.is_participant ? s.participant_count - 1 : s.participant_count }
          : s));
      } else {
        await api.put(`/api/v1/sessions/${sessionId}/rsvp`, { status });
        setSessions((p) => p.map((s) => {
          if (s.id !== sessionId) return s;
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
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  return (
    <PageLayout size="lg">
      {error && <ErrorBanner message={error} />}
      {!group && !error && <p className="text-zinc-500 text-sm">{t('group_detail.loading')}</p>}

      {group && (
        <>
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-3 min-w-0">
              <h1 className="text-xl font-bold text-zinc-100 truncate">{group.name}</h1>
              <Badge variant={group.is_public ? 'public' : 'private'} />
            </div>
            <div className="flex items-center gap-2 shrink-0">
              {(myId && (myId === group.creator_id || isAdmin(auth.user))) && (
                <Button variant="secondary" size="sm" onClick={() => navigate(`/groups/${id}/edit`)}>
                  {t('group_detail.edit')}
                </Button>
              )}
              {isAdmin(auth.user) && (
                <Button variant="danger" size="sm" onClick={handleDeleteGroup}>
                  {t('group_detail.delete')}
                </Button>
              )}
            </div>
          </div>

          {group.discord_invite && (
            <a
              href={group.discord_invite}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-3 bg-[#5865F2]/10 border border-[#5865F2]/30 hover:border-[#5865F2]/60 rounded-xl px-4 py-3 transition-colors group w-fit"
            >
              <svg className="w-5 h-5 text-[#5865F2] shrink-0" viewBox="0 0 24 24" fill="currentColor">
                <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057c.002.022.015.043.03.056a19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03z"/>
              </svg>
              <span className="text-sm font-medium text-[#5865F2] group-hover:text-[#7289da]">
                {t('group_detail.discord_join')}
              </span>
            </a>
          )}

          <section className="space-y-3">
            <SectionLabel>{t('group_detail.week_sessions')}</SectionLabel>
            <WeekCalendar sessions={sessions} onRsvp={handleRsvp} />
          </section>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <section className="bg-ui-surface border border-ui-border rounded-xl p-4 space-y-3">
              <div className="flex items-center justify-between">
                <SectionLabel count={group.members.length}>{t('group_detail.members')}</SectionLabel>
                {!group.is_public && myId && group.members.some((m) => m.keycloak_id === myId) && (
                  <Button size="sm" variant="secondary" onClick={handleShowInvite}>
                    {showInvite ? t('group_detail.invite_toggle_close') : t('group_detail.invite_toggle_open')}
                  </Button>
                )}
              </div>

              {showInvite && (
                <div className="border border-ui-border rounded-lg overflow-hidden">
                  {invitable.length === 0 ? (
                    <p className="px-3 py-2 text-xs text-zinc-500">{t('group_detail.no_invitable_friends')}</p>
                  ) : (
                    <ul className="divide-y divide-zinc-800">
                      {invitable.map((u) => (
                        <li key={u.keycloak_id} className="flex items-center gap-3 px-3 py-2">
                          <Avatar name={u.username} size="sm" />
                          <span className="text-sm text-zinc-200 flex-1">{u.username}</span>
                          <Button
                            size="sm"
                            variant={invitedIds.has(u.keycloak_id) ? 'secondary' : 'primary'}
                            disabled={invitedIds.has(u.keycloak_id)}
                            onClick={() => handleInvite(u.keycloak_id)}
                          >
                            {invitedIds.has(u.keycloak_id) ? t('group_detail.invited_check') : t('group_detail.invite_button')}
                          </Button>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              )}

              {group.members.length === 0
                ? <p className="text-zinc-500 text-sm">{t('group_detail.no_members')}</p>
                : (() => {
                  const visible = showAllMembers ? group.members : group.members.slice(0, MEMBER_LIMIT);
                  const hidden = group.members.length - MEMBER_LIMIT;
                  return (
                    <>
                      <ul className="space-y-1 -mx-4">
                        {visible.map((m) => (
                          <li key={m.keycloak_id}>
                            <UserRow
                              keycloak_id={m.keycloak_id}
                              username={m.username}
                              right={m.is_friend ? <Badge variant="friend" /> : undefined}
                            />
                          </li>
                        ))}
                      </ul>
                      {hidden > 0 && (
                        <button
                          onClick={() => setShowAllMembers((v) => !v)}
                          className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
                        >
                          {showAllMembers
                            ? t('group_detail.show_less')
                            : t('group_detail.show_more', { count: hidden })}
                        </button>
                      )}
                    </>
                  );
                })()}
            </section>

            <section className="bg-ui-surface border border-ui-border rounded-xl p-4 space-y-3">
              <SectionLabel count={group.possible_games.length}>{t('group_detail.possible_games')}</SectionLabel>
              {group.possible_games.length === 0 ? (
                <p className="text-zinc-500 text-sm">
                  {group.members.length < 2
                    ? t('group_detail.no_games_too_few_members')
                    : t('group_detail.no_games')}
                </p>
              ) : (() => {
                const visible = showAllGames ? group.possible_games : group.possible_games.slice(0, GAME_LIMIT);
                const hidden = group.possible_games.length - GAME_LIMIT;
                return (
                  <>
                    <ul className="space-y-1.5">
                      {visible.map(({ name, count, total }) => {
                        const all = count === total;
                        return (
                          <li key={name} className="flex items-center gap-2">
                            <GameCard game={library.get(name) ?? { name }} className="flex-1 min-w-0" />
                            <span className={`shrink-0 text-xs font-medium px-2 py-0.5 rounded-full border ${
                              all
                                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                                : 'bg-ui-raised text-zinc-400 border-ui-border'
                            }`}>
                              {count}/{total}
                            </span>
                          </li>
                        );
                      })}
                    </ul>
                    {hidden > 0 && (
                      <button
                        onClick={() => setShowAllGames((v) => !v)}
                        className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
                      >
                        {showAllGames
                          ? t('group_detail.show_less')
                          : t('group_detail.show_more', { count: hidden })}
                      </button>
                    )}
                  </>
                );
              })()}
            </section>
          </div>
        </>
      )}
    </PageLayout>
  );
}
