import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { isAbortError } from '../utils/abort';
import Badge from '../components/atoms/Badge';
import Button from '../components/atoms/Button';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import SearchBar from '../components/molecules/SearchBar';
import UserRow from '../components/molecules/UserRow';
import PageLayout from '../components/templates/PageLayout';
import type { Friend } from '../types';
import { api } from '../services/api';

interface FriendRequest {
  id: string;
  from_id: string;
  from_username: string;
}

export default function FriendsPage() {
  const { t } = useTranslation();
  const [friends, setFriends] = useState<Friend[]>([]);
  const [incoming, setIncoming] = useState<FriendRequest[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<Friend[]>([]);
  const [sentIds, setSentIds] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    api.get<Friend[]>('/api/v1/friends', controller.signal).then(setFriends)
      .catch((err: unknown) => { if (!isAbortError(err)) setError(err instanceof Error ? err.message : t('common.error')); });
    api.get<FriendRequest[]>('/api/v1/friends/requests', controller.signal).then(setIncoming)
      .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    return () => controller.abort();
  }, [t]);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      setSearchResults(await api.get<Friend[]>(`/api/v1/users/search?q=${encodeURIComponent(searchQuery)}`));
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleRemove = async (id: string) => {
    try {
      await api.delete(`/api/v1/friends/${id}`);
      setFriends((p) => p.filter((f) => f.keycloak_id !== id));
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleSendRequest = async (id: string) => {
    try {
      await api.post(`/api/v1/friends/requests/${id}`);
      setSentIds((p) => new Set([...p, id]));
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleAccept = async (req: FriendRequest) => {
    try {
      await api.post(`/api/v1/friends/requests/${req.id}/accept`);
      setIncoming((p) => p.filter((r) => r.id !== req.id));
      setFriends((p) => [...p, { keycloak_id: req.from_id, username: req.from_username }]);
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleDecline = async (reqId: string) => {
    try {
      await api.delete(`/api/v1/friends/requests/${reqId}`);
      setIncoming((p) => p.filter((r) => r.id !== reqId));
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const friendIds = new Set(friends.map((f) => f.keycloak_id));

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}

      {/* Incoming requests */}
      {incoming.length > 0 && (
        <section className="space-y-3">
          <SectionLabel count={incoming.length}>{t('friends.incoming_requests')}</SectionLabel>
          <ul className="divide-y divide-zinc-800 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
            {incoming.map((req) => (
              <li key={req.id}>
                <UserRow
                  keycloak_id={req.from_id}
                  username={req.from_username}
                  right={
                    <div className="flex gap-2">
                      <Button size="sm" onClick={() => handleAccept(req)}>{t('common.accept')}</Button>
                      <Button size="sm" variant="secondary" onClick={() => handleDecline(req.id)}>{t('common.decline')}</Button>
                    </div>
                  }
                />
              </li>
            ))}
          </ul>
        </section>
      )}

      {/* Friend list */}
      <section className="space-y-3">
        <SectionLabel count={friends.length}>{t('friends.my_friends')}</SectionLabel>
        {friends.length === 0 ? (
          <p className="text-zinc-500 text-sm">{t('friends.no_friends')}</p>
        ) : (
          <ul className="divide-y divide-zinc-800 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
            {friends.map((f) => (
              <li key={f.keycloak_id}>
                <UserRow
                  keycloak_id={f.keycloak_id}
                  username={f.username}
                  right={
                    <Button variant="danger" size="sm" onClick={() => handleRemove(f.keycloak_id)}>
                      {t('friends.remove')}
                    </Button>
                  }
                />
              </li>
            ))}
          </ul>
        )}
      </section>

      <hr className="border-zinc-800" />

      {/* User search */}
      <section className="space-y-3">
        <SectionLabel>{t('friends.search_title')}</SectionLabel>
        <SearchBar value={searchQuery} onChange={setSearchQuery} onSubmit={handleSearch} placeholder={t('friends.search_placeholder')} />

        {searchResults.length > 0 && (
          <ul className="divide-y divide-zinc-800 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
            {searchResults.map((u) => {
              const isFriend = friendIds.has(u.keycloak_id);
              const isSent = sentIds.has(u.keycloak_id);
              return (
                <li key={u.keycloak_id}>
                  <UserRow
                    keycloak_id={u.keycloak_id}
                    username={u.username}
                    right={
                      isFriend
                        ? <Badge variant="friend" />
                        : isSent
                          ? <span className="text-xs text-zinc-500">{t('friends.request_sent')}</span>
                          : <Button size="sm" onClick={() => handleSendRequest(u.keycloak_id)}>
                              {t('friends.send_request')}
                            </Button>
                    }
                  />
                </li>
              );
            })}
          </ul>
        )}
        {searchResults.length === 0 && searchQuery && (
          <p className="text-zinc-500 text-sm">{t('friends.no_results')}</p>
        )}
      </section>
    </PageLayout>
  );
}
