import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { AutofillCandidate } from '../components/molecules/AutofillPicker';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import GameCard from '../components/molecules/GameCard';
import GameSearchResults from '../components/molecules/GameSearchResults';
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
  const [mode, setMode] = useState<'search' | 'manual'>('search');

  // search mode state
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<AutofillCandidate[] | null>(null);
  const [searching, setSearching] = useState(false);
  const [addedNames, setAddedNames] = useState<Set<string>>(new Set());
  const [busyName, setBusyName] = useState<string | null>(null);

  // manual mode state
  const [newName, setNewName] = useState('');
  const [newGenre, setNewGenre] = useState('');
  const [newDesc, setNewDesc] = useState('');
  const [newFile, setNewFile] = useState<File | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const [error, setError] = useState<string | null>(null);

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

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    const q = searchQuery.trim();
    if (!q) return;
    setSearching(true);
    setError(null);
    setSearchResults(null);
    try {
      const results = await api.get<AutofillCandidate[]>(`/api/v1/library/${encodeURIComponent(q)}/autofill`);
      setSearchResults(results);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('library.autofill_error'));
    } finally {
      setSearching(false);
    }
  };

  const handleAddFromSearch = async (candidate: AutofillCandidate) => {
    setBusyName(candidate.rawg_name);
    setError(null);
    try {
      await api.post(`/api/v1/library/${encodeURIComponent(candidate.rawg_name)}/autofill`, {
        rawg_id: candidate.rawg_id,
        genre: candidate.genre,
      });
      setAddedNames((prev) => new Set([...prev, candidate.rawg_name]));
      const updated = await api.get<Game[]>('/api/v1/library');
      setGames(updated);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('library.add_error'));
    } finally {
      setBusyName(null);
    }
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

  const handleCloseCreate = () => {
    setShowCreate(false);
    setSearchQuery('');
    setSearchResults(null);
    setAddedNames(new Set());
    setMode('search');
  };

  return (
    <PageLayout>
      <div className="flex items-center justify-between">
        <SectionLabel data-testid="section-library">{t('library.title')}</SectionLabel>
        <Button variant="secondary" onClick={() => { if (showCreate) { handleCloseCreate(); } else { setShowCreate(true); } }}>
          {showCreate ? t('common.cancel') : t('library.new_game')}
        </Button>
      </div>

      {error && <ErrorBanner message={error} />}

      {showCreate && (
        <div className="bg-ui-surface border border-ui-border rounded-xl p-4 space-y-3">
          <div className="flex gap-2">
            <Button
              type="button"
              size="sm"
              variant={mode === 'search' ? 'primary' : 'secondary'}
              onClick={() => setMode('search')}
            >
              {t('library.search_mode')}
            </Button>
            <Button
              type="button"
              size="sm"
              variant={mode === 'manual' ? 'primary' : 'secondary'}
              onClick={() => setMode('manual')}
            >
              {t('library.manual_mode')}
            </Button>
          </div>

          {mode === 'search' ? (
            <div className="space-y-3">
              <form onSubmit={handleSearch} className="flex gap-2">
                <Input
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder={t('library.search_query_placeholder')}
                  className="flex-1"
                  autoFocus
                />
                <Button type="submit" variant="secondary" disabled={searching || !searchQuery.trim()}>
                  {searching ? t('common.loading') : t('library.search_button')}
                </Button>
              </form>
              {(searching || searchResults !== null) && (
                <GameSearchResults
                  candidates={searchResults ?? []}
                  loading={searching}
                  addedNames={addedNames}
                  busyName={busyName}
                  onAdd={handleAddFromSearch}
                  onClose={() => setSearchResults(null)}
                />
              )}
              {searchResults === null && !searching && (
                <p className="text-sm text-zinc-500 text-center py-4">{t('library.search_empty')}</p>
              )}
            </div>
          ) : (
            <form onSubmit={handleCreate} className="space-y-3">
              <SectionLabel>{t('library.new_game_section')}</SectionLabel>
              <Input value={newName} onChange={(e) => setNewName(e.target.value)} placeholder={t('library.name_placeholder')} required className="w-full" />
              <Input value={newGenre} onChange={(e) => setNewGenre(e.target.value)} placeholder={t('library.genre_placeholder')} className="w-full" />
              <Input value={newDesc} onChange={(e) => setNewDesc(e.target.value)} placeholder={t('library.description_placeholder')} className="w-full" />
              <div className="space-y-1">
                <p className="text-xs text-zinc-500">{t('library.thumbnail_label')}</p>
                <input
                  ref={fileRef}
                  type="file"
                  accept="image/*"
                  onChange={(e) => setNewFile(e.target.files?.[0] ?? null)}
                  className="text-sm text-zinc-400 file:mr-3 file:py-1 file:px-3 file:rounded-lg file:border-0 file:text-xs file:bg-ui-raised file:text-zinc-200 hover:file:bg-zinc-600 cursor-pointer"
                />
              </div>
              <div className="flex gap-2">
                <Button type="submit">{t('common.create')}</Button>
                <Button type="button" variant="secondary" onClick={handleCloseCreate}>{t('common.cancel')}</Button>
              </div>
            </form>
          )}
        </div>
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
