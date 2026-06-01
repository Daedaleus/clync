import SessionRow from '../molecules/SessionRow';
import type { RsvpStatus, Session } from '../../types';

interface Props {
  sessions: Session[];
  onRsvp?: (id: string, status: RsvpStatus | null) => void;
  onDelete?: (id: string) => void;
  emptyMessage?: string;
}

export default function SessionList({ sessions, onRsvp, onDelete, emptyMessage = 'Keine Spielzeiten.' }: Props) {
  if (sessions.length === 0) {
    return <p className="text-zinc-500 text-sm">{emptyMessage}</p>;
  }
  return (
    <ul className="divide-y divide-zinc-800 bg-ui-surface border border-ui-border rounded-xl overflow-hidden">
      {sessions.map((s) => (
        <SessionRow key={s.id} session={s} onRsvp={onRsvp} onDelete={onDelete} />
      ))}
    </ul>
  );
}
