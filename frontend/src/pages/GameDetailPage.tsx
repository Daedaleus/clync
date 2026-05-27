import { useEffect, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import SectionLabel from '../components/atoms/SectionLabel';
import AutofillPicker, { type AutofillCandidate } from '../components/molecules/AutofillPicker';
import ErrorBanner from '../components/molecules/ErrorBanner';
import PageLayout from '../components/templates/PageLayout';
import type { Game } from '../types';
import { api } from '../services/api';
import { isAbortError } from '../utils/abort';
import { config } from '../config';

interface MeGames { games: string[] }

function thumbnailSrc(name: string, url: string | null | undefined): string | undefined {
  if (!url) return undefined;
  if (url.startsWith('http')) return url;
  return `${config.apiUrl}/api/v1/library/${encodeURIComponent(name)}/thumbnail`;
}

export default function GameDetailPage() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [game, setGame] = useState<Game | null>(null);
  const [inWishlist, setInWishlist] = useState(false);
  const [editing, setEditing] = useState(false);
  const [editGenre, setEditGenre] = useState('');
  const [editDesc, setEditDesc] = useState('');
  const [editFile, setEditFile] = useState<File | null>(null);
  const [saving, setSaving] = useState(false);
  const [autofilling, setAutofilling] = useState(false);
  const [candidates, setCandidates] = useState<AutofillCandidate[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!name) return;
    const controller = new AbortController();
    api.get<Game>(`/api/v1/library/${encodeURIComponent(name)}`, controller.signal).then((g) => {
      setGame(g);
      setEditGenre(g.genre ?? '');
      setEditDesc(g.description ?? '');
    }).catch((err: unknown) => { if (!isAbortError(err)) setError('Spiel nicht gefunden'); });
    api.get<MeGames>('/api/v1/me', controller.signal)
      .then((me) => setInWishlist(me.games.includes(name)))
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    return () => controller.abort();
  }, [name]);

  const handleDelete = async () => {
    if (!name || !window.confirm(`"${name}" aus der Bibliothek löschen?`)) return;
    try {
      await api.delete(`/api/v1/library/${encodeURIComponent(name)}`);
      navigate('/library');
    } catch (err) { setError(err instanceof Error ? err.message : 'Löschen fehlgeschlagen'); }
  };

  const handleAutofill = async () => {
    if (!name) return;
    setAutofilling(true); setError(null); setCandidates(null);
    try {
      const result = await api.get<AutofillCandidate[]>(`/api/v1/library/${encodeURIComponent(name)}/autofill`);
      setCandidates(result);
    } catch (err) { setError(err instanceof Error ? err.message : 'Autofill fehlgeschlagen'); }
    finally { setAutofilling(false); }
  };

  const handlePickCandidate = async (candidate: AutofillCandidate) => {
    if (!name) return;
    setCandidates(null); setError(null);
    try {
      const updated = await api.post<Game>(`/api/v1/library/${encodeURIComponent(name)}/autofill`, {
        rawg_id: candidate.rawg_id,
        genre: candidate.genre,
      });
      setGame(updated);
      setEditGenre(updated.genre ?? '');
      setEditDesc(updated.description ?? '');
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler beim Speichern'); }
  };

  const handleToggleWishlist = async () => {
    if (!name) return;
    try {
      if (inWishlist) {
        await api.delete(`/api/v1/me/games?name=${encodeURIComponent(name)}`);
        setInWishlist(false);
      } else {
        await api.post('/api/v1/me/games', { name });
        setInWishlist(true);
      }
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
  };

  const handleSave = async () => {
    if (!name) return;
    setSaving(true); setError(null);
    try {
      await api.put(`/api/v1/library/${encodeURIComponent(name)}`, {
        genre: editGenre.trim() || null,
        description: editDesc.trim() || null,
      });

      let newThumb = game?.thumbnail_url ?? null;
      if (editFile) {
        const res = await api.upload<{ url: string }>(
          `/api/v1/library/${encodeURIComponent(name)}/thumbnail`,
          editFile,
        );
        newThumb = res.url;
        if (fileRef.current) fileRef.current.value = '';
        setEditFile(null);
      }

      setGame((g) => g && {
        ...g,
        genre: editGenre.trim() || null,
        description: editDesc.trim() || null,
        thumbnail_url: newThumb,
      });
      setEditing(false);
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
    finally { setSaving(false); }
  };

  const src = game ? thumbnailSrc(game.name, game.thumbnail_url) : undefined;

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}
      {!game && !error && <p className="text-zinc-500 text-sm">Lade Spiel…</p>}

      {game && (
        <>
          {src && (
            <div className="rounded-xl overflow-hidden border border-zinc-800">
              <img src={src} alt={game.name} className="w-full max-h-64 object-cover"
                onError={(e) => { (e.target as HTMLImageElement).parentElement!.style.display = 'none'; }} />
            </div>
          )}

          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0">
              <h1 className="text-2xl font-bold text-zinc-100">{game.name}</h1>
              {game.genre && (
                <span className="inline-block mt-1 text-xs px-2.5 py-0.5 rounded-full bg-zinc-800 border border-zinc-700 text-zinc-400">
                  {game.genre}
                </span>
              )}
            </div>
            <div className="shrink-0 flex gap-2">
              <Button
                variant={inWishlist ? 'secondary' : 'primary'}
                onClick={handleToggleWishlist}
                className={inWishlist ? 'hover:bg-red-500/10 hover:text-red-400 hover:border-red-500/30' : ''}
              >
                {inWishlist ? 'Aus Liste entfernen' : '+ Zur Liste'}
              </Button>
              <Button variant="secondary" disabled={autofilling} onClick={handleAutofill}>
                {autofilling ? 'Laden…' : '✨ Auto-Ausfüllen'}
              </Button>
              <Button variant="secondary" onClick={() => setEditing((v) => !v)}>
                {editing ? 'Abbrechen' : 'Bearbeiten'}
              </Button>
              <Button
                variant="danger"
                onClick={handleDelete}
                title="Aus Bibliothek löschen (nur wenn niemand das Spiel hat)"
              >
                Löschen
              </Button>
            </div>
          </div>

          {/* Autofill picker */}
          {(autofilling || candidates !== null) && (
            <AutofillPicker
              candidates={candidates ?? []}
              loading={autofilling}
              onSelect={handlePickCandidate}
              onCancel={() => setCandidates(null)}
            />
          )}

          {!editing && game.description && (
            <p className="text-sm text-zinc-400 leading-relaxed">{game.description}</p>
          )}
          {!editing && !game.description && !game.genre && !game.thumbnail_url && (
            <p className="text-sm text-zinc-600 italic">Noch keine Details hinterlegt.</p>
          )}

          {editing && (
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
              <SectionLabel>Details bearbeiten</SectionLabel>
              <div className="space-y-1.5">
                <label className="text-xs text-zinc-500">Genre</label>
                <Input value={editGenre} onChange={(e) => setEditGenre(e.target.value)} placeholder="z.B. Shooter, RPG, Strategy" className="w-full" />
              </div>
              <div className="space-y-1.5">
                <label className="text-xs text-zinc-500">Kurzbeschreibung</label>
                <textarea
                  value={editDesc}
                  onChange={(e) => setEditDesc(e.target.value)}
                  placeholder="Kurze Beschreibung des Spiels…"
                  rows={3}
                  className="w-full bg-zinc-800 border border-zinc-700 text-sm text-zinc-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-violet-500 resize-none placeholder-zinc-600"
                />
              </div>
              <div className="space-y-1.5">
                <label className="text-xs text-zinc-500">Thumbnail</label>
                <input
                  ref={fileRef}
                  type="file"
                  accept="image/*"
                  onChange={(e) => setEditFile(e.target.files?.[0] ?? null)}
                  className="text-sm text-zinc-400 file:mr-3 file:py-1 file:px-3 file:rounded-lg file:border-0 file:text-xs file:bg-zinc-700 file:text-zinc-200 hover:file:bg-zinc-600 cursor-pointer"
                />
              </div>
              <div className="flex gap-2">
                <Button onClick={handleSave} disabled={saving}>
                  {saving ? 'Speichern…' : 'Speichern'}
                </Button>
                <Button variant="secondary" onClick={() => setEditing(false)}>Abbrechen</Button>
              </div>
            </div>
          )}
        </>
      )}
    </PageLayout>
  );
}
