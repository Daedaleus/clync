import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import Avatar from '../components/atoms/Avatar';
import Badge from '../components/atoms/Badge';
import Button from '../components/atoms/Button';
import ErrorBanner from '../components/molecules/ErrorBanner';
import PageLayout from '../components/templates/PageLayout';
import { api } from '../services/api';
import { isAdmin } from '../utils/auth';
import { config } from '../config';
import { fmtDateTime, isPast, isLateJoinable } from '../utils/date';

interface Participant {
  keycloak_id: string;
  username: string;
}

interface SessionDetail {
  id: string;
  user_id: string;
  username: string;
  game: string;
  scheduled_at: string;
  scope: string;
  group_ids: string[];
  group_names: string[];   // may be missing from cached sessions before migration
  participants: Participant[];
  participant_count: number;
  is_mine: boolean;
  is_participant: boolean;
  thumbnail_url?: string | null;
}

export default function SessionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [session, setSession] = useState<SessionDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!id) return;
    api.get<SessionDetail>(`/api/v1/sessions/${id}`)
      .then(setSession)
      .catch(() => setError('Session nicht gefunden'));
  }, [id]);

  const refresh = () => id
    ? api.get<SessionDetail>(`/api/v1/sessions/${id}`).then(setSession)
    : Promise.resolve();

  const handleJoin = async () => {
    setLoading(true); setError(null);
    try { await api.post(`/api/v1/sessions/${id}/join`); await refresh(); }
    catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
    finally { setLoading(false); }
  };

  const handleLeave = async () => {
    setLoading(true); setError(null);
    try { await api.delete(`/api/v1/sessions/${id}/join`); await refresh(); }
    catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
    finally { setLoading(false); }
  };

  const handleDelete = async () => {
    setLoading(true); setError(null);
    try { await api.delete(`/api/v1/sessions/${id}`); window.history.back(); }
    catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
    finally { setLoading(false); }
  };

  if (!session && !error) return (
    <PageLayout><p className="text-zinc-500 text-sm">Lade Session…</p></PageLayout>
  );

  if (!session) return (
    <PageLayout>{error && <ErrorBanner message={error} />}</PageLayout>
  );

  const lateJoinable = isLateJoinable(session.scheduled_at);
  const past = isPast(session.scheduled_at) && !lateJoinable;
  const thumbSrc = session.thumbnail_url
    ? `${config.apiUrl}/api/v1/library/${encodeURIComponent(session.game)}/thumbnail`
    : undefined;

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}

      {/* Hero */}
      <div className="relative rounded-2xl overflow-hidden bg-zinc-900 border border-zinc-800">
        {thumbSrc ? (
          <>
            <img
              src={thumbSrc}
              alt={session.game}
              className="w-full h-48 sm:h-64 object-cover"
              onError={(e) => { (e.target as HTMLImageElement).parentElement!.classList.add('no-thumb'); }}
            />
            {/* Gradient overlay */}
            <div className="absolute inset-0 bg-gradient-to-t from-zinc-950/90 via-zinc-950/30 to-transparent" />
          </>
        ) : (
          <div className="w-full h-32 bg-zinc-800 flex items-center justify-center text-5xl">🎮</div>
        )}

        {/* Text over gradient */}
        <div className={`${thumbSrc ? 'absolute bottom-0 left-0 right-0' : ''} p-5 space-y-1`}>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={session.scope === 'global' ? 'global' : 'groups'} />
            {past && <Badge variant="private" label="Vergangen" />}
          </div>
          <h1 className={`text-2xl font-bold leading-tight ${thumbSrc ? 'text-white' : 'text-zinc-100'}`}>
            {session.game}
          </h1>
        </div>
      </div>

      {/* Meta + actions */}
      <div className="flex items-start justify-between gap-4 flex-wrap">
        <div className="space-y-1">
          {/* Date/time */}
          <p className="text-lg font-medium text-zinc-100">{fmtDateTime(session.scheduled_at)}</p>
          {/* Creator */}
          <p className="text-sm text-zinc-500">
            von{' '}
            <Link to={`/users/${session.user_id}`} className="text-zinc-300 hover:text-violet-400 transition-colors">
              {session.username}
            </Link>
          </p>
          {/* Groups */}
          {session.scope === 'groups' && (session.group_names ?? []).length > 0 && (
            <p className="text-sm text-zinc-500">
              mit{' '}
              <span className="text-zinc-300">{(session.group_names ?? []).join(', ')}</span>
            </p>
          )}
        </div>

        {/* Actions */}
        {!past && (
          <div className="flex gap-2 flex-wrap">
            {(session.is_mine || isAdmin()) && (
              <Button variant="danger" size="sm" disabled={loading} onClick={handleDelete}>
                Löschen
              </Button>
            )}
            {!session.is_mine && !session.is_participant && (
              <Button variant="primary" disabled={loading} onClick={handleJoin}>
                {loading ? '…' : '+ Beitreten'}
              </Button>
            )}
            {!session.is_mine && session.is_participant && (
              <Button
                variant="secondary"
                disabled={loading}
                onClick={handleLeave}
                className="hover:bg-red-500/10 hover:text-red-400 hover:border-red-500/30"
              >
                {loading ? '…' : 'Austreten'}
              </Button>
            )}
          </div>
        )}
      </div>

      {/* Participants */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-zinc-400">Teilnehmer</span>
          <span className="text-xs px-2 py-0.5 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-400">
            {session.participant_count}
          </span>
        </div>

        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
          {/* Creator first */}
          <Link
            to={`/users/${session.user_id}`}
            className="flex items-center gap-3 bg-zinc-900 border border-zinc-800 rounded-xl px-3 py-2.5 hover:border-zinc-700 hover:bg-zinc-800/50 transition-colors group"
          >
            <Avatar name={session.username} size="sm" />
            <div className="min-w-0">
              <p className="text-sm text-zinc-200 truncate group-hover:text-violet-300 transition-colors">
                {session.username}
              </p>
              <p className="text-[10px] text-zinc-600">Ersteller</p>
            </div>
          </Link>

          {session.participants.map((p) => (
            <Link
              key={p.keycloak_id}
              to={`/users/${p.keycloak_id}`}
              className="flex items-center gap-3 bg-zinc-900 border border-zinc-800 rounded-xl px-3 py-2.5 hover:border-zinc-700 hover:bg-zinc-800/50 transition-colors group"
            >
              <Avatar name={p.username} size="sm" />
              <p className="text-sm text-zinc-200 truncate group-hover:text-violet-300 transition-colors">
                {p.username}
              </p>
            </Link>
          ))}
        </div>

        {session.participants.length === 0 && !session.is_mine && (
          <p className="text-sm text-zinc-600">Noch keine weiteren Teilnehmer.</p>
        )}
      </section>
    </PageLayout>
  );
}
