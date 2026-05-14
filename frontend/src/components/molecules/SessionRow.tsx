import { Link } from 'react-router-dom';
import Badge from '../atoms/Badge';
import Button from '../atoms/Button';
import type { Session } from '../../types';
import { isPast, isLateJoinable, fmtDateTime } from '../../utils/date';
import { config } from '../../config';

interface Props {
  session: Session;
  onJoin?: (id: string) => void;
  onDelete?: (id: string) => void;
}

function thumbnailSrc(session: { thumbnail_url?: string | null; game: string }) {
  if (!session.thumbnail_url) return undefined;
  return `${config.apiUrl}/api/v1/library/${encodeURIComponent(session.game)}/thumbnail`;
}

export default function SessionRow({ session: s, onJoin, onDelete }: Props) {
  const lateJoinable = isLateJoinable(s.scheduled_at);
  const past = isPast(s.scheduled_at) && !lateJoinable;
  const src = thumbnailSrc(s);

  return (
    <li className={`flex items-center gap-3 px-3 py-3 ${past ? 'opacity-40' : ''}`}>
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
              läuft gerade
            </span>
          )}
          {past && <span className="text-xs text-zinc-600">vergangen</span>}
        </div>
        <p className="text-xs text-zinc-500">
          {fmtDateTime(s.scheduled_at)}
          <span className="mx-1.5 text-zinc-700">·</span>
          {s.participant_count} {s.participant_count === 1 ? 'Person' : 'Personen'}
        </p>
        <p className="text-xs text-zinc-600 truncate">
          {s.username}
          {s.scope === 'groups' && s.group_names && s.group_names.length > 0 && (
            <> · <span className="text-zinc-500">{s.group_names.join(', ')}</span></>
          )}
        </p>
      </Link>

      {/* Actions */}
      <div className="shrink-0 flex items-center gap-2">
        {s.is_participant && (
          <span className="text-xs text-emerald-400 font-medium">Zugesagt</span>
        )}
        {!s.is_mine && !s.is_participant && !past && onJoin && (
          <Button variant="primary" size="sm" onClick={() => onJoin(s.id)}>Beitreten</Button>
        )}
        {s.is_mine && onDelete && (
          <Button variant="danger" size="sm" onClick={() => onDelete(s.id)}>Löschen</Button>
        )}
      </div>
    </li>
  );
}
