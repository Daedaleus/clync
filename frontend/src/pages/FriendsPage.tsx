import { useEffect, useState } from 'react';
import Badge from '../components/atoms/Badge';
import Button from '../components/atoms/Button';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import SearchBar from '../components/molecules/SearchBar';
import UserRow from '../components/molecules/UserRow';
import PageLayout from '../components/templates/PageLayout';
import type { Friend } from '../types';
import { api } from '../services/api';

export default function FriendsPage() {
  const [friends, setFriends] = useState<Friend[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<Friend[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.get<Friend[]>('/api/v1/friends').then(setFriends)
      .catch((err: unknown) => setError(err instanceof Error ? err.message : 'Fehler'));
  }, []);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      setSearchResults(await api.get<Friend[]>(`/api/v1/users/search?q=${encodeURIComponent(searchQuery)}`));
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
  };

  const handleRemove = async (id: string) => {
    try {
      await api.delete(`/api/v1/friends/${id}`);
      setFriends((p) => p.filter((f) => f.keycloak_id !== id));
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
  };

  const friendIds = new Set(friends.map((f) => f.keycloak_id));

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}

      <section className="space-y-3">
        <SectionLabel count={friends.length}>Meine Freunde</SectionLabel>
        {friends.length === 0 ? (
          <p className="text-zinc-500 text-sm">Noch keine Freunde hinzugefügt.</p>
        ) : (
          <ul className="divide-y divide-zinc-800 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
            {friends.map((f) => (
              <li key={f.keycloak_id}>
                <UserRow
                  keycloak_id={f.keycloak_id}
                  username={f.username}
                  right={
                    <Button variant="danger" size="sm" onClick={() => handleRemove(f.keycloak_id)}>
                      Entfernen
                    </Button>
                  }
                />
              </li>
            ))}
          </ul>
        )}
      </section>

      <hr className="border-zinc-800" />

      <section className="space-y-3">
        <SectionLabel>Benutzer suchen</SectionLabel>
        <SearchBar value={searchQuery} onChange={setSearchQuery} onSubmit={handleSearch} placeholder="Benutzername…" />

        {searchResults.length > 0 && (
          <ul className="divide-y divide-zinc-800 bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
            {searchResults.map((u) => (
              <li key={u.keycloak_id}>
                <UserRow
                  keycloak_id={u.keycloak_id}
                  username={u.username}
                  right={friendIds.has(u.keycloak_id)
                    ? <Badge variant="friend" />
                    : <span className="text-sm text-zinc-500 hover:text-violet-400 transition-colors">Profil →</span>
                  }
                />
              </li>
            ))}
          </ul>
        )}
        {searchResults.length === 0 && searchQuery && <p className="text-zinc-500 text-sm">Keine Benutzer gefunden.</p>}
      </section>
    </PageLayout>
  );
}
