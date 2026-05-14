import SessionRow from '../molecules/SessionRow';
import type { Session } from '../../types';

interface Props {
  sessions: Session[];
  onJoin?: (id: string) => void;
  onDelete?: (id: string) => void;
  emptyMessage?: string;
}

export default function SessionList({ sessions, onJoin, onDelete, emptyMessage = 'Keine Spielzeiten.' }: Props) {
  if (sessions.length === 0) {
    return <p className="text-zinc-500 text-sm">{emptyMessage}</p>;
  }
  return (
    <ul className="divide-y divide-zinc-800 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
      {sessions.map((s) => (
        <SessionRow key={s.id} session={s} onJoin={onJoin} onDelete={onDelete} />
      ))}
    </ul>
  );
}
