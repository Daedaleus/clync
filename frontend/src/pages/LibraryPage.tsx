import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import AutofillPicker, { type AutofillCandidate } from '../components/molecules/AutofillPicker';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import GameCard from '../components/molecules/GameCard';
import PageLayout from '../components/templates/PageLayout';
import type { Game } from '../types';
import { api } from '../services/api';
import { isAbortError } from '../utils/abort';

interface MeGames { games: string[] }

export default function LibraryPage() {
  const { t } = useTranslation();
  const [games, setGames] = useState<Game[]>([]);
  const [myGames, setMyGames] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState('');
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState('');
  const [newGenre, setNewGenre] = useState('');
  const [newDesc, setNewDesc] = useState('');
  const [newFile, setNewFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [autofilling, setAutofilling] = useState(false);
  const [candidates, setCandidates] = useState<AutofillCandidate[] | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const controller = new AbortController();
    api.get<Game[]>('/api/v1/library', controller.signal)
      .then(setGames)
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    api.get<MeGames>('/api/v1/me', controller.signal)
      .then((me) => setMyGames(new Set(me.games)))
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    return () => controller.abort();
  }, []);

  const filtered = useMemo(() =>
    games.filter((g) => g.name.toLowerCase().includes(search.toLowerCase())),
    [games, search],
  );

  const handleToggle = async (name: string) => {
    try {
      if (myGames.has(name)) {
        await api.delete(`/api/v1/me/games?name=${encodeURIComponent(name)}`);
        setMyGames((p) => { const n = new Set(p); n.delete(name); return n; });
      } else {
        await api.post('/api/v1/me/games', { name });
        setMyGames((p) => new Set([...p, name]));
      }
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleAutofillSearch = async () => {
    const name = newName.trim();
    if (!name) return;
    setAutofilling(true); setError(null); setCandidates(null);
    try {
      const result = await api.get<AutofillCandidate[]>(`/api/v1/library/${encodeURIComponent(name)}/autofill`);
      setCandidates(result);
    } catch (err) { setError(err instanceof Error ? err.message : t('library.autofill_error')); }
    finally { setAutofilling(false); }
  };

  const handlePickCandidate = async (candidate: AutofillCandidate) => {
    const name = newName.trim();
    if (!name) return;
    setCandidates(null); setError(null);
    try {
      await api.post(`/api/v1/library/${encodeURIComponent(name)}/autofill`, {
        rawg_id: candidate.rawg_id,
        genre: candidate.genre,
      });
      const updated = await api.get<Game[]>('/api/v1/library');
      setGames(updated);
      setNewName(''); setNewGenre(''); setNewDesc(''); setNewFile(null);
      setShowCreate(false);
    } catch (err) { setError(err instanceof Error ? err.message : t('library.save_error')); }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = newName.trim();
    if (!name) return;
    setError(null);
    try {
      await api.post('/api/v1/library', {
        name,
        genre: newGenre.trim() || null,
        description: newDesc.trim() || null,
      });
      if (newFile) {
        await api.upload<{ url: string }>(
          `/api/v1/library/${encodeURIComponent(name)}/thumbnail`,
          newFile,
        );
      }
      const updated = await api.get<Game[]>('/api/v1/library');
      setGames(updated);
      setNewName(''); setNewGenre(''); setNewDesc(''); setNewFile(null);
      if (fileRef.current) fileRef.current.value = '';
      setShowCreate(false);
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  return (
    <PageLayout>
      <div className="flex items-center justify-between">
        <SectionLabel>{t('library.title')}</SectionLabel>
        <Button variant="secondary" onClick={() => setShowCreate((v) => !v)}>
          {showCreate ? t('common.cancel') : t('library.new_game')}
        </Button>
      </div>

      {error && <ErrorBanner message={error} />}

      {showCreate && (
        <form onSubmit={handleCreate} className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
          <SectionLabel>{t('library.new_game_section')}</SectionLabel>
          <div className="flex gap-2">
            <Input value={newName} onChange={(e) => { setNewName(e.target.value); setCandidates(null); }} placeholder={t('library.name_placeholder')} required className="flex-1" />
            <Button type="button" variant="secondary" disabled={autofilling || !newName.trim()} onClick={handleAutofillSearch}>
              {autofilling ? t('library.autofill_loading') : t('library.autofill_button')}
            </Button>
          </div>
          {(autofilling || candidates !== null) && (
            <AutofillPicker
              candidates={candidates ?? []}
              loading={autofilling}
              onSelect={handlePickCandidate}
              onCancel={() => setCandidates(null)}
            />
          )}
          <Input value={newGenre} onChange={(e) => setNewGenre(e.target.value)} placeholder={t('library.genre_placeholder')} className="w-full" />
          <Input value={newDesc} onChange={(e) => setNewDesc(e.target.value)} placeholder={t('library.description_placeholder')} className="w-full" />
          <div className="space-y-1">
            <p className="text-xs text-zinc-500">{t('library.thumbnail_label')}</p>
            <input
              ref={fileRef}
              type="file"
              accept="image/*"
              onChange={(e) => setNewFile(e.target.files?.[0] ?? null)}
              className="text-sm text-zinc-400 file:mr-3 file:py-1 file:px-3 file:rounded-lg file:border-0 file:text-xs file:bg-zinc-700 file:text-zinc-200 hover:file:bg-zinc-600 cursor-pointer"
            />
          </div>
          <div className="flex gap-2">
            <Button type="submit">{t('common.create')}</Button>
            <Button type="button" variant="secondary" onClick={() => setShowCreate(false)}>{t('common.cancel')}</Button>
          </div>
        </form>
      )}

      <Input
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        placeholder={t('library.search_placeholder')}
        className="w-full"
      />

      {filtered.length === 0 ? (
        <p className="text-zinc-500 text-sm text-center py-8">
          {games.length === 0 ? t('library.empty_library') : t('library.no_results')}
        </p>
      ) : (
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
          {filtered.map((g) => (
            <GameCard
              key={g.name}
              game={g}
              inWishlist={myGames.has(g.name)}
              onToggle={() => handleToggle(g.name)}
            />
          ))}
        </div>
      )}
    </PageLayout>
  );
}
