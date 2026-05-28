import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { isAbortError } from '../utils/abort';
import SectionLabel from '../components/atoms/SectionLabel';
import { config } from '../config';
import VisibilitySelect, { type Visibility } from '../components/atoms/VisibilitySelect';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import ErrorBanner from '../components/molecules/ErrorBanner';
import GameCard from '../components/molecules/GameCard';
import GameInput from '../components/molecules/GameInput';
import PageLayout from '../components/templates/PageLayout';
import type { Game } from '../types';
import { api } from '../services/api';

interface ProfileData {
  games: string[];
  steam_handle: string | null;
  steam_visibility: Visibility;
  discord_handle: string | null;
  discord_visibility: Visibility;
}


export default function ProfilePage() {
  const { t } = useTranslation();
  const [data, setData] = useState<ProfileData | null>(null);
  const [library, setLibrary] = useState<Map<string, Game>>(new Map());
  const [steam, setSteam] = useState('');
  const [steamVis, setSteamVis] = useState<Visibility>('public');
  const [discord, setDiscord] = useState('');
  const [discordVis, setDiscordVis] = useState<Visibility>('public');
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    api.get<Game[]>('/api/v1/library', controller.signal)
      .then((gs) => setLibrary(new Map(gs.map((g) => [g.name, g]))))
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    api.get<ProfileData>('/api/v1/me', controller.signal).then((d) => {
      setData(d);
      setSteam(d.steam_handle ?? '');
      setSteamVis(d.steam_visibility ?? 'public');
      setDiscord(d.discord_handle ?? '');
      setDiscordVis(d.discord_visibility ?? 'public');
    }).catch((err: unknown) => { if (!isAbortError(err)) setError(t('profile.load_error')); });
    return () => controller.abort();
  }, [t]);

  const handleAddGame = async (name: string) => {
    try {
      await api.post('/api/v1/me/games', { name });
      setData((p) => p && { ...p, games: [...p.games, name] });
    } catch { setError(t('profile.game_add_error')); }
  };

  const handleRemoveGame = async (name: string) => {
    try {
      await api.delete(`/api/v1/me/games?name=${encodeURIComponent(name)}`);
      setData((p) => p && { ...p, games: p.games.filter((g) => g !== name) });
    } catch { setError(t('profile.game_remove_error')); }
  };

  const handleSave = async () => {
    setSaving(true); setError(null); setSaved(false);
    try {
      await api.put('/api/v1/me/profile', {
        steam_handle: steam.trim() || null,
        steam_visibility: steamVis,
        discord_handle: discord.trim() || null,
        discord_visibility: discordVis,
      });
      setSaved(true);
      setTimeout(() => setSaved(false), 2500);
    } catch { setError(t('profile.save_error')); }
    finally { setSaving(false); }
  };

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}
      {!data && !error && <p className="text-zinc-500 text-sm">{t('profile.loading')}</p>}

      {data && (
        <>
          {/* Keycloak account management */}
          <a
            href={`${config.keycloakUrl}/realms/${config.keycloakRealm}/account/`}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-between bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 hover:border-zinc-700 hover:bg-zinc-800/50 transition-colors group"
          >
            <div>
              <p className="text-sm font-medium text-zinc-200">{t('profile.account_settings')}</p>
              <p className="text-xs text-zinc-500 mt-0.5">{t('profile.account_settings_subtitle')}</p>
            </div>
            <span className="text-zinc-600 group-hover:text-zinc-400 transition-colors text-sm">↗</span>
          </a>

          <section className="space-y-3">
            <SectionLabel>{t('profile.my_games')}</SectionLabel>
            {data.games.length > 0 && (
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {data.games.map((name) => (
                  <div key={name} className="relative group">
                    <GameCard game={library.get(name) ?? { name }} />
                    <button
                      onClick={() => handleRemoveGame(name)}
                      title={t('common.delete')}
                      className="absolute top-1.5 right-1.5 w-6 h-6 rounded-full bg-zinc-900/80 text-zinc-400 hover:text-red-400 hover:bg-zinc-800 transition-colors text-xs flex items-center justify-center opacity-0 group-hover:opacity-100"
                    >
                      ✕
                    </button>
                  </div>
                ))}
              </div>
            )}
            <GameInput onAdd={handleAddGame} />
          </section>

          <hr className="border-zinc-800" />

          <section className="space-y-4">
            <SectionLabel>{t('profile.social_accounts')}</SectionLabel>

            {/* Steam */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
              <div className="flex items-center justify-between gap-3">
                <span className="text-sm font-medium text-zinc-200">Steam</span>
                <VisibilitySelect value={steamVis} onChange={setSteamVis} />
              </div>
              <Input
                placeholder={t('profile.steam_placeholder')}
                value={steam}
                onChange={(e) => setSteam(e.target.value)}
              />
            </div>

            {/* Discord */}
            <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
              <div className="flex items-center justify-between gap-3">
                <span className="text-sm font-medium text-zinc-200">Discord</span>
                <VisibilitySelect value={discordVis} onChange={setDiscordVis} />
              </div>
              <Input
                placeholder={t('profile.discord_placeholder')}
                value={discord}
                onChange={(e) => setDiscord(e.target.value)}
              />
            </div>

            <Button onClick={handleSave} disabled={saving} className="w-full">
              {saving ? t('profile.saving') : saved ? t('profile.saved') : t('profile.save')}
            </Button>
          </section>
        </>
      )}
    </PageLayout>
  );
}
