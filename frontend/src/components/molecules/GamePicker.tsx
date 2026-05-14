import { useEffect, useMemo, useRef, useState } from 'react';
import { config } from '../../config';
import type { Game } from '../../types';
import { api } from '../../services/api';

interface Props {
  value: string;
  onChange: (name: string) => void;
  required?: boolean;
}

function thumbnailSrc(name: string, url: string | null | undefined) {
  if (!url) return undefined;
  if (url.startsWith('http')) return url;
  return `${config.apiUrl}/api/v1/library/${encodeURIComponent(name)}/thumbnail`;
}

export default function GamePicker({ value, onChange, required }: Props) {
  const [games, setGames] = useState<Game[]>([]);
  const [localQuery, setLocalQuery] = useState('');
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    api.get<Game[]>('/api/v1/library').then(setGames).catch(console.error);
  }, []);

  const filtered = useMemo(() =>
    localQuery.trim().length === 0
      ? games
      : games.filter((g) => g.name.toLowerCase().includes(localQuery.toLowerCase())),
    [games, localQuery],
  );

  // Close on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  const select = (name: string) => {
    onChange(name);
    setLocalQuery('');
    setOpen(false);
  };

  const inputClass =
    'w-full bg-zinc-800 border border-zinc-700 text-sm text-zinc-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-violet-500 placeholder-zinc-600';

  return (
    <div ref={ref} className="relative">
      <input
        type="text"
        value={open ? localQuery : value}
        required={required}
        placeholder="Spiel aus Bibliothek wählen…"
        className={inputClass}
        onChange={(e) => { setLocalQuery(e.target.value); setOpen(true); onChange(''); }}
        onFocus={() => { setLocalQuery(value); setOpen(true); }}
        autoComplete="off"
      />
      {/* Hidden input to satisfy form validation with the actual selected value */}
      <input type="hidden" value={value} required={required} />

      {open && filtered.length > 0 && (
        <ul className="absolute z-20 mt-1 w-full max-h-56 overflow-y-auto bg-zinc-900 border border-zinc-700 rounded-xl shadow-xl">
          {filtered.map((g) => {
            const src = thumbnailSrc(g.name, g.thumbnail_url);
            return (
              <li key={g.name}>
                <button
                  type="button"
                  className="flex items-center gap-3 w-full px-3 py-2 text-left hover:bg-zinc-800 transition-colors"
                  onClick={() => select(g.name)}
                >
                  {src ? (
                    <img src={src} alt="" className="w-8 h-8 rounded object-cover shrink-0" />
                  ) : (
                    <div className="w-8 h-8 rounded bg-zinc-700 flex items-center justify-center text-sm shrink-0">🎮</div>
                  )}
                  <div className="min-w-0">
                    <p className="text-sm text-zinc-100 truncate">{g.name}</p>
                    {g.genre && <p className="text-xs text-zinc-500 truncate">{g.genre}</p>}
                  </div>
                </button>
              </li>
            );
          })}
        </ul>
      )}

      {open && localQuery.trim().length > 0 && filtered.length === 0 && (
        <div className="absolute z-20 mt-1 w-full bg-zinc-900 border border-zinc-700 rounded-xl shadow-xl px-3 py-2 text-sm text-zinc-500">
          Kein Spiel gefunden.
        </div>
      )}
    </div>
  );
}
