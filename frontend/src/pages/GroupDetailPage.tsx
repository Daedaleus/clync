import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import Avatar from '../components/atoms/Avatar';
import Badge from '../components/atoms/Badge';
import GameCard from '../components/molecules/GameCard';
import Button from '../components/atoms/Button';
import SectionLabel from '../components/atoms/SectionLabel';
import UserRow from '../components/molecules/UserRow';
import WeekCalendar from '../components/organisms/WeekCalendar';
import PageLayout from '../components/templates/PageLayout';
import type { Game, Session } from '../types';
import { api } from '../services/api';
import keycloak from '../services/auth';
import { isAdmin } from '../utils/auth';
import ErrorBanner from '../components/molecules/ErrorBanner';
import { useNotifications } from '../hooks/useNotifications';
import { config } from '../config';

const API_BASE = config.apiUrl;

interface Member { keycloak_id: string; username: string; is_friend: boolean }
interface PossibleGame { name: string; count: number; total: number }
interface GroupDetail { id: string; name: string; is_public: boolean; members: Member[]; common_games: string[]; possible_games: PossibleGame[] }
interface InvitableUser { keycloak_id: string; username: string }

export default function GroupDetailPage() {
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
  const myId = keycloak.tokenParsed?.sub as string | undefined;

  useEffect(() => {
    if (!id) return;
    api.get<GroupDetail>(`/api/v1/groups/${id}`).then(setGroup).catch(() => setError('Gruppe nicht gefunden'));
    api.get<Session[]>(`/api/v1/groups/${id}/sessions`).then(setSessions).catch(console.error);
    api.get<Game[]>('/api/v1/library').then((gs) => setLibrary(new Map(gs.map((g) => [g.name, g])))).catch(console.error);
  }, [id]);

  // SSE subscription for real-time session updates
  useEffect(() => {
    if (!id || !keycloak.token) return;

    const url = `${API_BASE}/api/v1/groups/${id}/events?token=${encodeURIComponent(keycloak.token)}`;
    const es = new EventSource(url);

    es.addEventListener('session_created', (e) => {
      const session: Session = JSON.parse(e.data);
      setSessions((prev) => {
        if (prev.some((s) => s.id === session.id)) return prev;
        return [...prev, session].sort((a, b) => a.scheduled_at.localeCompare(b.scheduled_at));
      });
      // Direct notification when tab is active (Web Push handles the background case)
      if (session.user_id !== keycloak.tokenParsed?.sub) {
        notify(
          `WhatsUp – ${session.game}`,
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
  }, [id, notify]);

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
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
  };

  const handleDeleteGroup = async () => {
    if (!id || !window.confirm(`Gruppe "${group?.name}" wirklich löschen?`)) return;
    try {
      await api.delete(`/api/v1/groups/${id}`);
      navigate('/groups');
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
  };

  const handleJoin = async (sessionId: string) => {
    try {
      await api.post(`/api/v1/sessions/${sessionId}/join`);
      setSessions((p) => p.map((s) => s.id === sessionId
        ? { ...s, is_participant: true, participant_count: s.participant_count + 1 }
        : s));
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
  };

  return (
    <PageLayout size="lg">
      {error && <ErrorBanner message={error} />}
      {!group && !error && <p className="text-zinc-500 text-sm">Lade Gruppe…</p>}

      {group && (
        <>
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-3 min-w-0">
              <h1 className="text-xl font-bold text-zinc-100 truncate">{group.name}</h1>
              <Badge variant={group.is_public ? 'public' : 'private'} />
            </div>
            {isAdmin() && (
              <Button variant="danger" size="sm" onClick={handleDeleteGroup}>
                Gruppe löschen
              </Button>
            )}
          </div>

          <section className="space-y-3">
            <SectionLabel>Spielzeiten diese Woche</SectionLabel>
            <WeekCalendar sessions={sessions} onJoin={handleJoin} />
          </section>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <section className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
              <div className="flex items-center justify-between">
                <SectionLabel count={group.members.length}>Mitglieder</SectionLabel>
                {!group.is_public && myId && group.members.some((m) => m.keycloak_id === myId) && (
                  <Button size="sm" variant="secondary" onClick={handleShowInvite}>
                    {showInvite ? 'Schließen' : '+ Einladen'}
                  </Button>
                )}
              </div>

              {showInvite && (
                <div className="border border-zinc-700 rounded-lg overflow-hidden">
                  {invitable.length === 0 ? (
                    <p className="px-3 py-2 text-xs text-zinc-500">Keine einladbaren Freunde verfügbar.</p>
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
                            {invitedIds.has(u.keycloak_id) ? 'Eingeladen ✓' : 'Einladen'}
                          </Button>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              )}

              {group.members.length === 0
                ? <p className="text-zinc-500 text-sm">Keine Mitglieder.</p>
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
                          {showAllMembers ? '▲ Weniger anzeigen' : `▼ ${hidden} weitere anzeigen`}
                        </button>
                      )}
                    </>
                  );
                })()}
            </section>

            <section className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
              <SectionLabel count={group.possible_games.length}>Mögliche Spiele</SectionLabel>
              {group.possible_games.length === 0 ? (
                <p className="text-zinc-500 text-sm">
                  {group.members.length < 2 ? 'Noch zu wenige Mitglieder.' : 'Keine Spiele eingetragen.'}
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
                                : 'bg-zinc-800 text-zinc-400 border-zinc-700'
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
                        {showAllGames ? '▲ Weniger anzeigen' : `▼ ${hidden} weitere anzeigen`}
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
