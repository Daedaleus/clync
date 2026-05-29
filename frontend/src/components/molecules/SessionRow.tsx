import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import Badge from '../atoms/Badge';
import Button from '../atoms/Button';
import type { RsvpStatus, Session } from '../../types';
import { isPast, isLateJoinable, fmtDateTime } from '../../utils/date';
import { config } from '../../config';

interface Props {
  session: Session;
  onRsvp?: (id: string, status: RsvpStatus | null) => void;
  onDelete?: (id: string) => void;
}

function thumbnailSrc(session: { thumbnail_url?: string | null; game: string }) {
  if (!session.thumbnail_url) return undefined;
  return `${config.apiUrl}/api/v1/library/${encodeURIComponent(session.game)}/thumbnail`;
}

export default function SessionRow({ session: s, onRsvp, onDelete }: Props) {
  const { t } = useTranslation();

  const RSVP_BUTTONS: { status: RsvpStatus; icon: string; ariaLabel: string; active: string; inactive: string }[] = [
    {
      status: 'accepted',
      icon: '✓',
      ariaLabel: t('session_row.rsvp_accept'),
      active: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/40',
      inactive: 'text-zinc-500 border-zinc-700 hover:text-emerald-400 hover:border-emerald-500/40',
    },
    {
      status: 'maybe',
      icon: '?',
      ariaLabel: t('session_row.rsvp_maybe'),
      active: 'bg-amber-500/20 text-amber-400 border-amber-500/40',
      inactive: 'text-zinc-500 border-zinc-700 hover:text-amber-400 hover:border-amber-500/40',
    },
    {
      status: 'declined',
      icon: '✕',
      ariaLabel: t('session_row.rsvp_decline'),
      active: 'bg-red-500/20 text-red-400 border-red-500/40',
      inactive: 'text-zinc-500 border-zinc-700 hover:text-red-400 hover:border-red-500/40',
    },
  ];

  const lateJoinable = isLateJoinable(s.scheduled_at);
  const past = isPast(s.scheduled_at) && !lateJoinable;
  const src = thumbnailSrc(s);

  const handleRsvp = (status: RsvpStatus) => {
    if (!onRsvp) return;
    // Toggle off if already selected
    onRsvp(s.id, s.my_rsvp === status ? null : status);
  };

  return (
    <li data-testid={`session-${s.id}`} className={`flex items-center gap-3 px-3 py-3 ${past ? 'opacity-40' : ''}`}>
      {/* Thumbnail */}
      <Link to={`/sessions/${s.id}`} className="shrink-0">
        {src ? (
          <img
            src={src}
            alt={s.game}
            className="w-10 h-10 rounded-lg object-cover"
            onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
          />
        ) : (
          <div className="w-10 h-10 rounded-lg bg-zinc-800 flex items-center justify-center text-lg">🎮</div>
        )}
      </Link>

      {/* Info */}
      <Link to={`/sessions/${s.id}`} className="min-w-0 flex-1 group space-y-0.5">
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-sm font-medium text-zinc-100 group-hover:text-violet-300 transition-colors truncate">
            {s.game}
          </span>
          <Badge variant={s.scope === 'global' ? 'global' : 'groups'} />
          {lateJoinable && (
            <span className="text-xs font-medium text-amber-400 bg-amber-400/10 px-1.5 py-0.5 rounded-full">
              {t('session_row.late_joinable')}
            </span>
          )}
          {past && <span className="text-xs text-zinc-600">{t('session_row.past')}</span>}
        </div>
        <p className="text-xs text-zinc-500">
          {fmtDateTime(s.scheduled_at)}
          <span className="mx-1.5 text-zinc-700">·</span>
          {t('session_row.participant_count', { count: s.participant_count })}
        </p>
        <p className="text-xs text-zinc-600 truncate">
          {s.username}
          {s.scope === 'groups' && s.group_names && s.group_names.length > 0 && (
            <> · <span className="text-zinc-500">{s.group_names.join(', ')}</span></>
          )}
        </p>
      </Link>

      {/* Actions */}
      <div className="shrink-0 flex items-center gap-1.5">
        {!s.is_mine && !past && onRsvp && (
          <div className="flex gap-1">
            {RSVP_BUTTONS.map(({ status, icon, ariaLabel, active, inactive }) => (
              <button
                key={status}
                onClick={() => handleRsvp(status)}
                aria-label={ariaLabel}
                className={`w-7 h-7 rounded-lg border text-xs font-bold transition-colors ${
                  s.my_rsvp === status ? active : inactive
                }`}
              >
                {icon}
              </button>
            ))}
          </div>
        )}
        {s.is_mine && onDelete && (
          <Button variant="danger" size="sm" onClick={() => onDelete(s.id)}>{t('session_row.delete')}</Button>
        )}
      </div>
    </li>
  );
}
