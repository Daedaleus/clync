import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import ErrorBanner from '../components/molecules/ErrorBanner';
import PageLayout from '../components/templates/PageLayout';
import SectionLabel from '../components/atoms/SectionLabel';
import { api } from '../services/api';
import { isAbortError } from '../utils/abort';
import keycloak from '../services/auth';
import { isAdmin } from '../utils/auth';

interface GroupDetail {
  id: string;
  name: string;
  is_public: boolean;
  creator_id: string | null;
  discord_invite: string | null;
}

export default function GroupEditPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [group, setGroup] = useState<GroupDetail | null>(null);
  const [discordInvite, setDiscordInvite] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const myId = keycloak.tokenParsed?.sub as string | undefined;

  useEffect(() => {
    if (!id) return;
    const controller = new AbortController();
    api.get<GroupDetail>(`/api/v1/groups/${id}`, controller.signal)
      .then((g) => {
        // Redirect away if not creator or admin
        if (g.creator_id !== myId && !isAdmin()) {
          navigate(`/groups/${id}`);
          return;
        }
        setGroup(g);
        setDiscordInvite(g.discord_invite ?? '');
      })
      .catch((err: unknown) => { if (!isAbortError(err)) setError('Gruppe nicht gefunden'); });
    return () => controller.abort();
  }, [id, myId, navigate]);

  const handleSave = async () => {
    if (!id) return;
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      await api.put(`/api/v1/groups/${id}/discord-invite`, {
        url: discordInvite.trim() || null,
      });
      setSaved(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Fehler beim Speichern');
    } finally {
      setSaving(false);
    }
  };

  if (!group && !error) {
    return (
      <PageLayout>
        <p className="text-zinc-500 text-sm">Lade Gruppe…</p>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      <div className="space-y-6 max-w-lg">
        <div className="space-y-1">
          <button
            onClick={() => navigate(`/groups/${id}`)}
            className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors mb-1"
          >
            ← Zurück zur Gruppe
          </button>
          <h1 className="text-xl font-bold text-zinc-100">
            {group?.name} — Bearbeiten
          </h1>
        </div>

        {error && <ErrorBanner message={error} />}

        {saved && (
          <p className="text-sm text-green-400">Änderungen gespeichert.</p>
        )}

        <section className="bg-zinc-900 border border-zinc-800 rounded-xl p-5 space-y-4">
          <SectionLabel>Discord</SectionLabel>

          <div className="space-y-2">
            <label className="text-xs text-zinc-400">
              Einladungslink (leer lassen zum Entfernen)
            </label>
            <Input
              value={discordInvite}
              onChange={(e) => { setDiscordInvite(e.target.value); setSaved(false); }}
              placeholder="https://discord.gg/..."
              className="w-full font-mono text-sm"
            />
            <p className="text-xs text-zinc-600">
              Erlaubt: discord.gg/… oder discord.com/invite/…
            </p>
          </div>

          <div className="flex gap-3">
            <Button onClick={handleSave} disabled={saving}>
              {saving ? 'Speichern…' : 'Speichern'}
            </Button>
            <Button variant="secondary" onClick={() => navigate(`/groups/${id}`)}>
              Abbrechen
            </Button>
          </div>
        </section>
      </div>
    </PageLayout>
  );
}
