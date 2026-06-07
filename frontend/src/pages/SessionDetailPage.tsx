import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Link, useParams } from 'react-router-dom';
import Avatar from '../components/atoms/Avatar';
import Badge from '../components/atoms/Badge';
import Button from '../components/atoms/Button';
import ErrorBanner from '../components/molecules/ErrorBanner';
import PageLayout from '../components/templates/PageLayout';
import { api } from '../services/api';
import { isAbortError } from '../utils/abort';
import { useAuth } from 'react-oidc-context';
import { isAdmin } from '../utils/auth';
import { config } from '../config';
import { fmtDateTime, isPast, isLateJoinable } from '../utils/date';
import type { RsvpStatus } from '../types';

interface Participant {
  keycloak_id: string;
  username: string;
}

interface RsvpInfo {
  user_id: string;
  username: string;
  status: RsvpStatus;
}

interface InvitableUser {
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
  my_rsvp: RsvpStatus | null;
  rsvps: RsvpInfo[];
  thumbnail_url?: string | null;
  notes?: string | null;
}

export default function SessionDetailPage() {
  const { t } = useTranslation();
  const auth = useAuth();
  const { id } = useParams<{ id: string }>();
  const [session, setSession] = useState<SessionDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [showInvite, setShowInvite] = useState(false);
  const [invitableUsers, setInvitableUsers] = useState<InvitableUser[] | null>(null);
  const [invitedIds, setInvitedIds] = useState<Set<string>>(new Set());
  const invitePanelRef = useRef<HTMLDivElement>(null);

  const RSVP_OPTIONS: { status: RsvpStatus; label: string; icon: string; active: string; hover: string }[] = [
    {
      status: 'accepted',
      label: t('session_detail.rsvp_accept'),
      icon: '✓',
      active: 'bg-emerald-500/15 text-emerald-400 border-emerald-500/40',
      hover: 'hover:bg-emerald-500/10 hover:text-emerald-400 hover:border-emerald-500/30',
    },
    {
      status: 'maybe',
      label: t('session_detail.rsvp_maybe'),
      icon: '?',
      active: 'bg-amber-500/15 text-amber-400 border-amber-500/40',
      hover: 'hover:bg-amber-500/10 hover:text-amber-400 hover:border-amber-500/30',
    },
    {
      status: 'declined',
      label: t('session_detail.rsvp_decline'),
      icon: '✕',
      active: 'bg-red-500/15 text-red-400 border-red-500/40',
      hover: 'hover:bg-red-500/10 hover:text-red-400 hover:border-red-500/30',
    },
  ];

  const RSVP_STATUS_LABEL: Record<RsvpStatus, string> = {
    accepted: t('session_detail.rsvp_accept'),
    maybe: t('session_detail.rsvp_maybe'),
    declined: t('session_detail.rsvp_decline'),
  };

  useEffect(() => {
    if (!id) return;
    const controller = new AbortController();
    api.get<SessionDetail>(`/api/v1/sessions/${id}`, controller.signal)
      .then(setSession)
      .catch((err: unknown) => { if (!isAbortError(err)) setError(t('session_detail.not_found')); });
    return () => controller.abort();
  }, [id, t]);

  const refresh = () => id
    ? api.get<SessionDetail>(`/api/v1/sessions/${id}`).then(setSession)
    : Promise.resolve();

  const handleRsvp = async (status: RsvpStatus) => {
    if (!session) return;
    setLoading(true); setError(null);
    try {
      if (session.my_rsvp === status) {
        // Toggle off → remove RSVP
        await api.delete(`/api/v1/sessions/${id}/rsvp`);
        await refresh();
      } else {
        await api.put(`/api/v1/sessions/${id}/rsvp`, { status });
        await refresh();
      }
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
    finally { setLoading(false); }
  };

  const handleToggleInvite = async () => {
    if (showInvite) {
      setShowInvite(false);
      return;
    }
    if (invitableUsers === null) {
      const users = await api.get<InvitableUser[]>(`/api/v1/sessions/${id}/invitable`);
      setInvitableUsers(users);
    }
    setShowInvite(true);
    setTimeout(() => invitePanelRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' }), 50);
  };

  const handleInvite = async (userId: string) => {
    try {
      await api.post(`/api/v1/sessions/${id}/invite`, { user_id: userId });
      setInvitedIds((p) => new Set(p).add(userId));
    } catch { /* silently ignore */ }
  };

  const handleDelete = async () => {
    setLoading(true); setError(null);
    try { await api.delete(`/api/v1/sessions/${id}`); window.history.back(); }
    catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
    finally { setLoading(false); }
  };

  if (!session && !error) return (
    <PageLayout><p className="text-zinc-500 text-sm">{t('session_detail.loading')}</p></PageLayout>
  );

  if (!session) return (
    <PageLayout>{error && <ErrorBanner message={error} />}</PageLayout>
  );

  const lateJoinable = isLateJoinable(session.scheduled_at);
  const past = isPast(session.scheduled_at) && !lateJoinable;
  const thumbSrc = session.thumbnail_url
    ? `${config.apiUrl}/api/v1/library/${encodeURIComponent(session.game)}/thumbnail`
    : undefined;

  // Group RSVPs by status for the breakdown
  const rsvpByStatus = {
    accepted: session.rsvps.filter((r) => r.status === 'accepted'),
    maybe: session.rsvps.filter((r) => r.status === 'maybe'),
    declined: session.rsvps.filter((r) => r.status === 'declined'),
  };

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}

      {/* Hero */}
      <div className="relative rounded-2xl overflow-hidden bg-ui-surface border border-ui-border">
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
          <div className="w-full h-32 bg-ui-raised flex items-center justify-center text-5xl">🎮</div>
        )}

        {/* Text over gradient */}
        <div className={`${thumbSrc ? 'absolute bottom-0 left-0 right-0' : ''} p-5 space-y-1`}>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={session.scope === 'global' ? 'global' : 'groups'} />
            {past && <Badge variant="private" label={t('session_detail.past_label')} />}
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
            {t('session_detail.created_by')}{' '}
            <Link to={`/users/${session.user_id}`} className="text-zinc-300 hover:text-violet-400 transition-colors">
              {session.username}
            </Link>
          </p>
          {/* Groups */}
          {session.scope === 'groups' && (session.group_names ?? []).length > 0 && (
            <p className="text-sm text-zinc-500">
              {t('session_detail.with_groups')}{' '}
              <span className="text-zinc-300">{(session.group_names ?? []).join(', ')}</span>
            </p>
          )}
        </div>

        {/* Actions */}
        {!past && (
          <div className="flex gap-2 flex-wrap">
            {(session.is_mine || isAdmin(auth.user)) && (
              <Button variant="danger" size="sm" disabled={loading} onClick={handleDelete}>
                {t('session_detail.delete')}
              </Button>
            )}
            {!session.is_mine && (
              <div className="flex gap-1.5">
                {RSVP_OPTIONS.map(({ status, label, icon, active, hover }) => (
                  <button
                    key={status}
                    disabled={loading}
                    onClick={() => handleRsvp(status)}
                    className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-sm font-medium transition-colors disabled:opacity-50 ${
                      session.my_rsvp === status
                        ? active
                        : `border-ui-border text-zinc-400 ${hover}`
                    }`}
                  >
                    <span className="text-xs">{icon}</span>
                    {label}
                  </button>
                ))}
              </div>
            )}
            {(session.is_mine || session.is_participant) && (
              <Button variant="secondary" size="sm" onClick={handleToggleInvite}>
                {showInvite ? t('session_detail.invite_toggle_close') : t('session_detail.invite_toggle_open')}
              </Button>
            )}
          </div>
        )}
      </div>

      {/* RSVP breakdown */}
      {!session.is_mine && session.my_rsvp && (
        <p className="text-sm text-zinc-500">
          {t('session_detail.your_rsvp')}{' '}
          <span className={
            session.my_rsvp === 'accepted' ? 'text-emerald-400 font-medium'
            : session.my_rsvp === 'maybe' ? 'text-amber-400 font-medium'
            : 'text-red-400 font-medium'
          }>
            {RSVP_STATUS_LABEL[session.my_rsvp]}
          </span>
        </p>
      )}

      {/* Participants */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-zinc-400">{t('session_detail.participants')}</span>
          <span className="text-xs px-2 py-0.5 rounded-full bg-ui-raised border border-ui-border text-zinc-400">
            {session.participant_count}
          </span>
        </div>

        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
          {/* Creator first */}
          <Link
            to={`/users/${session.user_id}`}
            className="flex items-center gap-3 bg-ui-surface border border-ui-border rounded-xl px-3 py-2.5 hover:border-ui-border hover:bg-ui-raised/50 transition-colors group"
          >
            <Avatar name={session.username} size="sm" />
            <div className="min-w-0">
              <p className="text-sm text-zinc-200 truncate group-hover:text-violet-300 transition-colors">
                {session.username}
              </p>
              <p className="text-[10px] text-zinc-600">{t('session_detail.creator_role')}</p>
            </div>
          </Link>

          {session.participants.map((p) => (
            <Link
              key={p.keycloak_id}
              to={`/users/${p.keycloak_id}`}
              className="flex items-center gap-3 bg-ui-surface border border-ui-border rounded-xl px-3 py-2.5 hover:border-ui-border hover:bg-ui-raised/50 transition-colors group"
            >
              <Avatar name={p.username} size="sm" />
              <p className="text-sm text-zinc-200 truncate group-hover:text-violet-300 transition-colors">
                {p.username}
              </p>
            </Link>
          ))}
        </div>

        {session.participants.length === 0 && !session.is_mine && (
          <p className="text-sm text-zinc-600">{t('session_detail.no_other_participants')}</p>
        )}

        {/* RSVP breakdown: maybe + declined */}
        {(rsvpByStatus.maybe.length > 0 || rsvpByStatus.declined.length > 0) && (
          <div className="flex flex-wrap gap-3 pt-1">
            {rsvpByStatus.maybe.length > 0 && (
              <div className="flex items-center gap-1.5 text-xs text-amber-400">
                <span className="font-bold">?</span>
                <span>{rsvpByStatus.maybe.map((r) => r.username).join(', ')}</span>
              </div>
            )}
            {rsvpByStatus.declined.length > 0 && (
              <div className="flex items-center gap-1.5 text-xs text-red-400/70">
                <span className="font-bold">✕</span>
                <span>{rsvpByStatus.declined.map((r) => r.username).join(', ')}</span>
              </div>
            )}
          </div>
        )}
      </section>

      {/* Notes */}
      {session.notes && (
        <section className="space-y-1.5">
          <span className="text-xs font-semibold text-zinc-500 uppercase tracking-wider">{t('session_detail.notes_label')}</span>
          <p className="text-sm text-zinc-300 bg-ui-surface border border-ui-border rounded-xl px-4 py-3 whitespace-pre-wrap">
            {session.notes}
          </p>
        </section>
      )}

      {/* Invite panel */}
      {showInvite && (
        <section ref={invitePanelRef} className="space-y-2">
          <span className="text-sm font-medium text-zinc-400">{t('session_detail.invite_friends')}</span>
          {invitableUsers === null && (
            <p className="text-sm text-zinc-500">{t('session_detail.invite_loading')}</p>
          )}
          {invitableUsers !== null && invitableUsers.length === 0 && (
            <p className="text-sm text-zinc-500">{t('session_detail.no_invitable_friends')}</p>
          )}
          {invitableUsers !== null && invitableUsers.length > 0 && (
            <ul className="space-y-2">
              {invitableUsers.map((u) => (
                <li key={u.keycloak_id} className="flex items-center justify-between gap-3 bg-ui-surface border border-ui-border rounded-xl px-4 py-2.5">
                  <span className="text-sm text-zinc-200">{u.username}</span>
                  <Button
                    size="sm"
                    variant={invitedIds.has(u.keycloak_id) ? 'secondary' : 'primary'}
                    disabled={invitedIds.has(u.keycloak_id)}
                    onClick={() => handleInvite(u.keycloak_id)}
                  >
                    {invitedIds.has(u.keycloak_id) ? t('session_detail.invited_check') : t('session_detail.invite_button')}
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </section>
      )}
    </PageLayout>
  );
}
