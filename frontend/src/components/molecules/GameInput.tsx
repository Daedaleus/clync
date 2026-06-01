import { useEffect, useRef, useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { api } from '../../services/api';

interface Props {
  onAdd: (name: string) => void;
}

export default function GameInput({ onAdd }: Props) {
  const [value, setValue] = useState('');
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [open, setOpen] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  const fetchSuggestions = (q: string) => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (q.trim().length < 2) { setSuggestions([]); setOpen(false); return; }
    debounceRef.current = setTimeout(async () => {
      try {
        const results = await api.get<string[]>(`/api/v1/games?q=${encodeURIComponent(q)}`);
        setSuggestions(results);
        setOpen(results.length > 0);
      } catch { /* ignore */ }
    }, 300);
  };

  const submit = (name: string) => {
    const t = name.trim();
    if (!t) return;
    onAdd(t);
    setValue(''); setSuggestions([]); setOpen(false);
  };

  return (
    <div ref={containerRef} className="relative">
      <div className="flex gap-2">
        <Input
          value={value}
          onChange={(e) => { setValue(e.target.value); fetchSuggestions(e.target.value); }}
          onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); submit(value); } if (e.key === 'Escape') setOpen(false); }}
          onFocus={() => suggestions.length > 0 && setOpen(true)}
          placeholder="Spiel suchen oder hinzufügen…"
          className="flex-1"
        />
        <Button type="button" onClick={() => submit(value)}>Hinzufügen</Button>
      </div>

      {open && (
        <ul className="absolute z-10 top-full left-0 right-32 mt-1 bg-ui-raised border border-ui-border rounded-lg shadow-xl overflow-hidden">
          {suggestions.map((s) => (
            <li key={s}>
              <button
                type="button"
                onMouseDown={(e) => { e.preventDefault(); submit(s); }}
                className="w-full text-left px-4 py-2.5 text-sm text-zinc-200 hover:bg-ui-raised transition-colors cursor-pointer"
              >
                {s}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
