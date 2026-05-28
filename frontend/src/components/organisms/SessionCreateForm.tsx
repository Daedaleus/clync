import { useMemo, useState } from 'react';
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

/** All 96 quarter-hour slots for a day. */
const ALL_SLOTS = Array.from({ length: 96 }, (_, i) => {
  const h = String(Math.floor(i / 4)).padStart(2, '0');
  const m = String((i % 4) * 15).padStart(2, '0');
  return `${h}:${m}`;
});

function todayLocal(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

/** Returns the next 15-min-aligned slot at least 15 min from now. */
function nextSlot(): { date: string; time: string } {
  const now = new Date();
  now.setMinutes(now.getMinutes() + 15);
  const remainder = now.getMinutes() % 15;
  if (remainder !== 0) now.setMinutes(now.getMinutes() + (15 - remainder));
  now.setSeconds(0, 0);

  const date = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  const time = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
  return { date, time };
}

/** First slot index that is strictly in the future for today (with 5-min buffer). */
function firstAvailableMinutes(): number {
  const now = new Date();
  now.setMinutes(now.getMinutes() + 5);
  return Math.ceil((now.getHours() * 60 + now.getMinutes()) / 15) * 15;
}

// ── Component ─────────────────────────────────────────────────────────────────

export default function SessionCreateForm({ groups, onCreate, onError, onCancel }: Props) {
  const defaults = nextSlot();
  const [game, setGame] = useState('');
  const [date, setDate] = useState(defaults.date);
  const [time, setTime] = useState(defaults.time);
  const [scope, setScope] = useState<'global' | 'groups'>('global');
  const [selectedGroups, setSelectedGroups] = useState<Set<string>>(new Set());
  const [notes, setNotes] = useState('');
  const [validationError, setValidationError] = useState<string | null>(null);

  const availableSlots = useMemo(() => {
    if (date !== todayLocal()) return ALL_SLOTS;
    const minMins = firstAvailableMinutes();
    return ALL_SLOTS.filter((slot) => {
      const [h, m] = slot.split(':').map(Number);
      return h * 60 + m >= minMins;
    });
  }, [date]);

  // If the selected time is no longer available after a date change, bump to first slot.
  const effectiveTime = availableSlots.includes(time) ? time : (availableSlots[0] ?? '00:00');

  const toggleGroup = (id: string) =>
    setSelectedGroups((p) => {
      const n = new Set(p);
      if (n.has(id)) { n.delete(id); } else { n.add(id); }
      return n;
    });

  const handleDateChange = (val: string) => {
    setDate(val);
    setValidationError(null);
    // If the current time slot is no longer valid for the new date, reset it.
    if (val === todayLocal()) {
      const minMins = firstAvailableMinutes();
      const [h, m] = time.split(':').map(Number);
      if (h * 60 + m < minMins) {
        const first = ALL_SLOTS.find((s) => {
          const [sh, sm] = s.split(':').map(Number);
          return sh * 60 + sm >= minMins;
        });
        if (first) setTime(first);
      }
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setValidationError(null);

    if (!game.trim()) {
      setValidationError('Bitte ein Spiel aus der Bibliothek wählen.');
      return;
    }

    const scheduledAt = new Date(`${date}T${effectiveTime}:00`).toISOString();
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

  const fieldClass =
    'w-full h-10 bg-zinc-800 border border-zinc-700 text-sm text-zinc-200 rounded-lg px-3 ' +
    'focus:outline-none focus:ring-1 focus:ring-violet-500 [color-scheme:dark]';

  return (
    <form onSubmit={handleSubmit} className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-4">

      {/* Game */}
      <div className="space-y-1.5">
        <SectionLabel>Spiel</SectionLabel>
        <GamePicker value={game} onChange={setGame} required />
      </div>

      {/* Date + Time — stacked on mobile, side-by-side on sm+ */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div className="space-y-1.5">
          <SectionLabel>Datum</SectionLabel>
          <input
            type="date"
            value={date}
            min={todayLocal()}
            onChange={(e) => handleDateChange(e.target.value)}
            required
            className={fieldClass}
          />
        </div>
        <div className="space-y-1.5">
          <SectionLabel>Uhrzeit</SectionLabel>
          <div className="relative">
            <select
              value={effectiveTime}
              onChange={(e) => { setTime(e.target.value); setValidationError(null); }}
              className={`${fieldClass} appearance-none pr-9`}
            >
              {availableSlots.map((slot) => (
                <option key={slot} value={slot}>{slot} Uhr</option>
              ))}
              {availableSlots.length === 0 && (
                <option disabled>Kein Slot verfügbar</option>
              )}
            </select>
            <svg className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-zinc-400" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </div>
        </div>
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
        <Button type="submit" disabled={availableSlots.length === 0}>Eintragen</Button>
        <Button type="button" variant="secondary" onClick={onCancel}>Abbrechen</Button>
      </div>
    </form>
  );
}
