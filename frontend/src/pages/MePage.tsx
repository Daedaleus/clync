import { useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import Badge from '../components/atoms/Badge';
import Button from '../components/atoms/Button';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import GameCard from '../components/molecules/GameCard';
import SessionList from '../components/organisms/SessionList';
import PageLayout from '../components/templates/PageLayout';
import type { Game, GroupSummary, Invitation, Session } from '../types';
import { api } from '../services/api';
import { isPast } from '../utils/date';

interface MeData {
  username: string;
  games: string[];
  groups: GroupSummary[];
}

function greeting(name: string): string {
  const h = new Date().getHours();
  const salutation = h < 5 ? 'Gute Nacht' : h < 12 ? 'Guten Morgen' : h < 18 ? 'Guten Tag' : 'Guten Abend';
  return `${salutation}, ${name}`;
}

export default function MePage() {
  const [me, setMe] = useState<MeData | null>(null);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [invitations, setInvitations] = useState<Invitation[]>([]);
  const [library, setLibrary] = useState<Map<string, Game>>(new Map());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.get<MeData>('/api/v1/me').then(setMe).catch(() => setError('Profil konnte nicht geladen werden'));
    api.get<Session[]>('/api/v1/sessions/mine').then(setSessions).catch(console.error);
    api.get<Invitation[]>('/api/v1/invitations').then(setInvitations).catch(console.error);
    api.get<Game[]>('/api/v1/library').then((gs) => setLibrary(new Map(gs.map((g) => [g.name, g])))).catch(console.error);
  }, []);

  const upcomingSessions = useMemo(
    () => sessions.filter((s) => !isPast(s.scheduled_at)).slice(0, 3),
    [sessions],
  );

  const handleJoin = async (id: string) => {
    try {
      await api.post(`/api/v1/sessions/${id}/join`);
      setSessions((p) => p.map((s) => s.id === id
        ? { ...s, is_participant: true, participant_count: s.participant_count + 1 }
        : s));
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

  const handleDelete = async (id: string) => {
    try {
      await api.delete(`/api/v1/sessions/${id}`);
      setSessions((p) => p.filter((s) => s.id !== id));
    } catch { /* handled by SessionRow */ }
  };

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}
      {!me && !error && <p className="text-zinc-500 text-sm">Lade…</p>}

      {me && (
        <>
          {/* Greeting */}
          <div className="py-1">
            <h1 className="text-2xl font-bold text-zinc-100">{greeting(me.username)}</h1>
          </div>

          {/* Pending invitations */}
          {invitations.length > 0 && (
            <section className="space-y-2">
              <SectionLabel>Offene Einladungen</SectionLabel>
              <ul className="space-y-2">
                {invitations.map((inv) => (
                  <li key={inv.id} className="bg-violet-500/10 border border-violet-500/30 rounded-xl px-4 py-3 flex items-center justify-between gap-4">
                    <div className="min-w-0">
                      <p className="text-sm font-medium text-zinc-100 truncate">{inv.group_name}</p>
                      <p className="text-xs text-zinc-400">Eingeladen von {inv.inviter_username}</p>
                    </div>
                    <div className="shrink-0 flex gap-2">
                      <Button size="sm" variant="primary" onClick={() => handleAccept(inv.id, inv.group_id)}>
                        Annehmen
                      </Button>
                      <Button size="sm" variant="secondary" onClick={() => handleDecline(inv.id)}>
                        Ablehnen
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
              <SectionLabel>Nächste Sessions</SectionLabel>
              <Link to="/sessions" className="text-xs text-zinc-500 hover:text-violet-400 transition-colors">
                Alle ansehen →
              </Link>
            </div>
            {upcomingSessions.length === 0 ? (
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-6 text-center space-y-2">
                <p className="text-sm text-zinc-500">Keine anstehenden Sessions.</p>
                <Link to="/sessions" className="text-sm text-violet-400 hover:text-violet-300 transition-colors">
                  Jetzt eine erstellen →
                </Link>
              </div>
            ) : (
              <SessionList sessions={upcomingSessions} onJoin={handleJoin} onDelete={handleDelete} />
            )}
          </section>

          {/* Groups */}
          <section className="space-y-2">
            <div className="flex items-center justify-between">
              <SectionLabel>Deine Gruppen</SectionLabel>
              <Link to="/groups" className="text-xs text-zinc-500 hover:text-violet-400 transition-colors">
                Alle ansehen →
              </Link>
            </div>
            {me.groups.length === 0 ? (
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-6 text-center space-y-2">
                <p className="text-sm text-zinc-500">Noch keiner Gruppe beigetreten.</p>
                <Link to="/groups" className="text-sm text-violet-400 hover:text-violet-300 transition-colors">
                  Gruppen entdecken →
                </Link>
              </div>
            ) : (
              <ul className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {me.groups.map((g) => (
                  <li key={g.id}>
                    <Link
                      to={`/groups/${g.id}`}
                      className="flex items-center justify-between gap-3 bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 hover:border-zinc-700 hover:bg-zinc-800/50 transition-colors"
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
              <SectionLabel>Deine Spiele</SectionLabel>
              <Link to="/profile" className="text-xs text-zinc-500 hover:text-violet-400 transition-colors">
                Bearbeiten →
              </Link>
            </div>
            {me.games.length === 0 ? (
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-6 text-center space-y-2">
                <p className="text-sm text-zinc-500">Noch keine Spiele eingetragen.</p>
                <Link to="/profile" className="text-sm text-violet-400 hover:text-violet-300 transition-colors">
                  Spiele hinzufügen →
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
