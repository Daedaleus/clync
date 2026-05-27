import { useState } from 'react';
import Button from '../atoms/Button';
import SectionLabel from '../atoms/SectionLabel';
import GamePicker from '../molecules/GamePicker';
import type { GroupSummary, Session } from '../../types';
import { api } from '../../services/api';

interface Props {
  groups: GroupSummary[];
  onCreate: (session: Session) => void;
  onError: (msg: string) => void;
  onCancel: () => void;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/**
 * Formats a Date as the `YYYY-MM-DDTHH:MM` string required by datetime-local inputs.
 * Uses local time so the input reflects what the user sees on their device.
 */
function toDatetimeLocal(d: Date): string {
  return (
    `${d.getFullYear()}-` +
    `${String(d.getMonth() + 1).padStart(2, '0')}-` +
    `${String(d.getDate()).padStart(2, '0')}T` +
    `${String(d.getHours()).padStart(2, '0')}:` +
    `${String(d.getMinutes()).padStart(2, '0')}`
  );
}

/** Returns the next full 15-minute slot at least 15 min from now. */
function nextSlotDefault(): string {
  const now = new Date();
  now.setMinutes(now.getMinutes() + 15);
  // Round up to the next 15-minute boundary
  const remainder = now.getMinutes() % 15;
  if (remainder !== 0) now.setMinutes(now.getMinutes() + (15 - remainder));
  now.setSeconds(0, 0);
  return toDatetimeLocal(now);
}

/** Minimum selectable datetime: now + 2 minutes (small buffer to avoid race). */
function minDatetimeLocal(): string {
  const now = new Date();
  now.setMinutes(now.getMinutes() + 2);
  return toDatetimeLocal(now);
}

// ── Component ─────────────────────────────────────────────────────────────────

export default function SessionCreateForm({ groups, onCreate, onError, onCancel }: Props) {
  const [game, setGame] = useState('');
  const [datetimeLocal, setDatetimeLocal] = useState(nextSlotDefault);
  const [scope, setScope] = useState<'global' | 'groups'>('global');
  const [selectedGroups, setSelectedGroups] = useState<Set<string>>(new Set());
  const [notes, setNotes] = useState('');
  const [validationError, setValidationError] = useState<string | null>(null);

  const toggleGroup = (id: string) =>
    setSelectedGroups((p) => {
      const n = new Set(p);
      if (n.has(id)) { n.delete(id); } else { n.add(id); }
      return n;
    });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setValidationError(null);

    if (!game.trim()) {
      setValidationError('Bitte ein Spiel aus der Bibliothek wählen.');
      return;
    }

    // datetime-local value is local time; new Date() interprets it as local → correct UTC ISO string
    const scheduledAt = new Date(datetimeLocal).toISOString();
    if (new Date(scheduledAt) <= new Date()) {
      setValidationError('Der Zeitpunkt muss in der Zukunft liegen.');
      return;
    }

    try {
      const session = await api.post<Session>('/api/v1/sessions', {
        game,
        scheduled_at: scheduledAt,
        scope,
        group_ids: scope === 'groups' ? [...selectedGroups] : [],
        notes: notes.trim() || null,
      });
      onCreate(session);
    } catch (err) {
      onError(err instanceof Error ? err.message : 'Fehler beim Erstellen');
    }
  };

  const inputClass =
    'w-full h-9 bg-zinc-800 border border-zinc-700 text-sm text-zinc-200 rounded-lg px-3 ' +
    'focus:outline-none focus:ring-1 focus:ring-violet-500 ' +
    // Ensure calendar/clock icons use light colours in Webkit/Blink
    '[color-scheme:dark]';

  return (
    <form onSubmit={handleSubmit} className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-4">

      {/* Game */}
      <div className="space-y-1.5">
        <SectionLabel>Spiel</SectionLabel>
        <GamePicker value={game} onChange={setGame} required />
      </div>

      {/* Date + Time — single datetime-local input */}
      <div className="space-y-1.5">
        <SectionLabel>Datum &amp; Uhrzeit</SectionLabel>
        <input
          type="datetime-local"
          value={datetimeLocal}
          min={minDatetimeLocal()}
          step={900}
          onChange={(e) => { setDatetimeLocal(e.target.value); setValidationError(null); }}
          required
          className={inputClass}
        />
      </div>

      {validationError && (
        <p className="text-xs text-red-400">{validationError}</p>
      )}

      {/* Scope */}
      <div className="space-y-2">
        <SectionLabel>Sichtbarkeit</SectionLabel>
        <div className="flex gap-4">
          {(['global', 'groups'] as const).map((s) => (
            <label key={s} className="flex items-center gap-2 text-sm text-zinc-300 cursor-pointer">
              <input
                type="radio"
                checked={scope === s}
                onChange={() => setScope(s)}
                disabled={s === 'groups' && groups.length === 0}
                className="accent-violet-500"
              />
              {s === 'global' ? 'Global' : 'Bestimmte Gruppen'}
            </label>
          ))}
        </div>
      </div>

      {scope === 'groups' && (
        <div className="flex flex-wrap gap-2">
          {groups.map((g) => (
            <label
              key={g.id}
              className={`flex items-center gap-1.5 text-sm px-3 py-1.5 rounded-full border cursor-pointer transition-colors ${
                selectedGroups.has(g.id)
                  ? 'bg-violet-600 text-white border-violet-600'
                  : 'bg-zinc-800 text-zinc-300 border-zinc-700 hover:border-zinc-500'
              }`}
            >
              <input type="checkbox" className="sr-only" checked={selectedGroups.has(g.id)} onChange={() => toggleGroup(g.id)} />
              {g.name}
            </label>
          ))}
        </div>
      )}

      {/* Notes */}
      <div className="space-y-1.5">
        <SectionLabel>Notizen <span className="normal-case font-normal text-zinc-600">(optional)</span></SectionLabel>
        <textarea
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          maxLength={500}
          rows={3}
          placeholder="z.B. Serveradresse, Passwort, Discord-Link…"
          className="w-full bg-zinc-800 border border-zinc-700 text-sm text-zinc-200 placeholder:text-zinc-600 rounded-lg px-3 py-2 resize-none focus:outline-none focus:ring-1 focus:ring-violet-500"
        />
        {notes.length > 400 && (
          <p className="text-xs text-zinc-500 text-right">{notes.length}/500</p>
        )}
      </div>

      <div className="flex gap-2">
        <Button type="submit">Eintragen</Button>
        <Button type="button" variant="secondary" onClick={onCancel}>Abbrechen</Button>
      </div>
    </form>
  );
}
