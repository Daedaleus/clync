import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Link, useParams } from 'react-router-dom';
import Avatar from '../components/atoms/Avatar';
import Badge from '../components/atoms/Badge';
import Button from '../components/atoms/Button';
import GameCard from '../components/molecules/GameCard';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import PageLayout from '../components/templates/PageLayout';
import keycloak from '../services/auth';
import { api } from '../services/api';
import { isAbortError } from '../utils/abort';
import type { Game, GroupSummary } from '../types';

interface UserProfile {
  username: string;
  games: string[];
  is_friend: boolean;
  steam_handle: string | null;
  discord_handle: string | null;
}

// ── Icons ────────────────────────────────────────────────────────────────────

function SteamIcon() {
  return (
    <svg viewBox="0 0 24 24" className="w-4 h-4 fill-current" aria-hidden>
      <path d="M11.979 0C5.678 0 .511 4.86.022 11.037l6.432 2.658c.545-.371 1.203-.59 1.912-.59.063 0 .125.004.188.006l2.861-4.142V8.91c0-2.495 2.028-4.524 4.524-4.524 2.494 0 4.524 2.031 4.524 4.527s-2.03 4.525-4.524 4.525h-.105l-4.076 2.911c0 .052.004.105.004.159 0 1.875-1.515 3.396-3.39 3.396-1.635 0-3.016-1.173-3.331-2.727L.436 15.27C1.862 20.307 6.486 24 11.979 24c6.627 0 11.999-5.373 11.999-12S18.606 0 11.979 0zM7.54 18.21l-1.473-.61c.262.543.714.999 1.314 1.25 1.297.539 2.793-.076 3.332-1.375.263-.63.264-1.319.005-1.949s-.75-1.121-1.377-1.383c-.624-.26-1.29-.249-1.878-.03l1.523.63c.956.4 1.409 1.497 1.009 2.455-.397.957-1.494 1.409-2.455 1.012zm11.415-9.303c0-1.662-1.353-3.015-3.015-3.015-1.665 0-3.015 1.353-3.015 3.015 0 1.665 1.35 3.015 3.015 3.015 1.662 0 3.015-1.35 3.015-3.015zm-5.273.016c0-1.252 1.013-2.266 2.265-2.266 1.249 0 2.266 1.014 2.266 2.266 0 1.251-1.017 2.265-2.266 2.265-1.252 0-2.265-1.014-2.265-2.265z" />
    </svg>
  );
}

function DiscordIcon() {
  return (
    <svg viewBox="0 0 24 24" className="w-4 h-4 fill-current" aria-hidden>
      <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057c.002.022.015.043.032.055a19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z" />
    </svg>
  );
}

// ── Steam URL helper ─────────────────────────────────────────────────────────

function steamUrl(handle: string): string {
  if (handle.startsWith('http')) return handle;
  return `https://steamcommunity.com/id/${encodeURIComponent(handle)}`;
}

// ── UserPage ─────────────────────────────────────────────────────────────────

export default function UserPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [library, setLibrary] = useState<Map<string, Game>>(new Map());
  const [myGames, setMyGames] = useState<Set<string>>(new Set());
  const [publicGroups, setPublicGroups] = useState<GroupSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const isOwnProfile = id === keycloak.tokenParsed?.sub;

  useEffect(() => {
    if (!id) return;
    const controller = new AbortController();
    api.get<UserProfile>(`/api/v1/users/${id}`, controller.signal)
      .then(setProfile)
      .catch((err: unknown) => { if (!isAbortError(err)) setError(t('user.not_found')); });
    api.get<Game[]>('/api/v1/library', controller.signal)
      .then((gs) => setLibrary(new Map(gs.map((g) => [g.name, g]))))
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    api.get<GroupSummary[]>(`/api/v1/users/${id}/groups`, controller.signal)
      .then(setPublicGroups)
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    if (!isOwnProfile) {
      api.get<{ games: string[] }>('/api/v1/me', controller.signal)
        .then((me) => setMyGames(new Set(me.games)))
        .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    }
    return () => controller.abort();
  }, [id, isOwnProfile, t]);

  const commonGames = useMemo(
    () => profile?.games.filter((g) => myGames.has(g)) ?? [],
    [profile, myGames],
  );
  const otherGames = useMemo(
    () => profile?.games.filter((g) => !myGames.has(g)) ?? [],
    [profile, myGames],
  );

  const handleToggleFriend = async () => {
    if (!profile || !id) return;
    setLoading(true); setError(null);
    try {
      if (profile.is_friend) {
        await api.delete(`/api/v1/friends/${id}`);
        setProfile((p) => p && { ...p, is_friend: false });
      } else {
        await api.post(`/api/v1/friends/${id}`);
        setProfile((p) => p && { ...p, is_friend: true });
      }
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
    finally { setLoading(false); }
  };

  const hasSocial = profile && (profile.steam_handle || profile.discord_handle);

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}
      {!profile && !error && <p className="text-zinc-500 text-sm">{t('user.loading')}</p>}

      {profile && (
        <>
          {/* Header card */}
          <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-5">
            <div className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-4 min-w-0">
                <Avatar name={profile.username} size="lg" />
                <div className="min-w-0">
                  <h1 className="text-lg font-semibold text-zinc-100">{profile.username}</h1>
                  {profile.is_friend && (
                    <span className="text-xs text-violet-400">{t('user.is_friend')}</span>
                  )}
                </div>
              </div>

              {isOwnProfile ? (
                <Link to="/profile">
                  <Button variant="secondary" size="sm">{t('user.edit_profile')}</Button>
                </Link>
              ) : (
                <Button
                  disabled={loading}
                  onClick={handleToggleFriend}
                  variant={profile.is_friend ? 'secondary' : 'primary'}
                  size="sm"
                  className={profile.is_friend ? 'hover:bg-red-500/10 hover:text-red-400 hover:border-red-500/30' : ''}
                >
                  {loading ? '…' : profile.is_friend ? t('user.remove_friend') : t('user.add_friend')}
                </Button>
              )}
            </div>
          </div>

          {/* Social accounts */}
          {hasSocial && (
            <section className="space-y-2">
              <SectionLabel>{t('user.social_section')}</SectionLabel>
              <div className="bg-zinc-900 border border-zinc-800 rounded-xl divide-y divide-zinc-800">
                {profile.steam_handle && (
                  <a
                    href={steamUrl(profile.steam_handle)}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-3 px-4 py-3 text-sm text-zinc-300 hover:text-zinc-100 hover:bg-zinc-800/50 transition-colors rounded-t-xl group"
                  >
                    <span className="text-[#c6d4df] group-hover:text-[#c6d4df]"><SteamIcon /></span>
                    <span className="font-medium text-zinc-400 w-16 shrink-0">Steam</span>
                    <span className="truncate">{profile.steam_handle}</span>
                    <span className="ml-auto text-zinc-600 text-xs">↗</span>
                  </a>
                )}
                {profile.discord_handle && (
                  <div className="flex items-center gap-3 px-4 py-3 text-sm text-zinc-300">
                    <span className="text-[#5865F2]"><DiscordIcon /></span>
                    <span className="font-medium text-zinc-400 w-16 shrink-0">Discord</span>
                    <span className="truncate">{profile.discord_handle}</span>
                  </div>
                )}
              </div>
            </section>
          )}

          {/* Common games — only shown when viewing someone else's profile */}
          {!isOwnProfile && commonGames.length > 0 && (
            <section className="space-y-2">
              <div className="flex items-center gap-2">
                <SectionLabel>{t('user.common_games')}</SectionLabel>
                <span className="text-xs px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 font-medium">
                  {commonGames.length}
                </span>
              </div>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {commonGames.map((name) => (
                  <div key={name} className="relative">
                    <GameCard game={library.get(name) ?? { name }} />
                    <span className="absolute top-1.5 right-1.5 text-[10px] px-1.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 font-medium border border-emerald-500/30">
                      {t('user.common_badge')}
                    </span>
                  </div>
                ))}
              </div>
            </section>
          )}

          {/* Public groups */}
          {!isOwnProfile && publicGroups.length > 0 && (
            <section className="space-y-2">
              <SectionLabel count={publicGroups.length}>{t('user.public_groups')}</SectionLabel>
              <ul className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {publicGroups.map((g) => (
                  <li key={g.id}>
                    <Link
                      to={`/groups/${g.id}`}
                      className="flex items-center justify-between gap-3 bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 hover:border-zinc-700 hover:bg-zinc-800/50 transition-colors"
                    >
                      <span className="text-sm font-medium text-zinc-100 truncate">{g.name}</span>
                      <Badge variant="public" />
                    </Link>
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Their other games */}
          {(isOwnProfile ? profile.games : otherGames).length > 0 && (
            <section className="space-y-2">
              <SectionLabel>
                {isOwnProfile
                  ? t('user.games')
                  : otherGames.length > 0 ? t('user.more_games') : t('user.games')}
              </SectionLabel>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {(isOwnProfile ? profile.games : otherGames).map((name) => (
                  <GameCard key={name} game={library.get(name) ?? { name }} />
                ))}
              </div>
            </section>
          )}
        </>
      )}
    </PageLayout>
  );
}
