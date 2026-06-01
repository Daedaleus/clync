import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import Badge from '../components/atoms/Badge';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import SearchBar from '../components/molecules/SearchBar';
import PageLayout from '../components/templates/PageLayout';
import type { GroupSummary, Session } from '../types';
import { api } from '../services/api';
import { isAdmin } from '../utils/auth';
import { isAbortError } from '../utils/abort';

interface GroupResult extends GroupSummary { member_count: number }

export default function GroupsPage() {
  const { t } = useTranslation();
  const [myGroups, setMyGroups] = useState<GroupSummary[]>([]);
  const [sessionsByGroup, setSessionsByGroup] = useState<Map<string, Session[]>>(new Map());
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<GroupResult[]>([]);
  const [joined, setJoined] = useState<Set<string>>(new Set());
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newName, setNewName] = useState('');
  const [newIsPublic, setNewIsPublic] = useState(true);
  const [error, setError] = useState<string | null>(null);

  function fmtSession(s: Session): string {
    const d = new Date(s.scheduled_at).toLocaleDateString(undefined, {
      weekday: 'short', day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit',
    });
    const n = s.participant_count;
    return `${t('sessions.participant_count', { count: n })} · ${d} · ${s.game}`;
  }

  useEffect(() => {
    const controller = new AbortController();
    api.get<GroupSummary[]>('/api/v1/groups/mine', controller.signal).then(setMyGroups)
      .catch((err: unknown) => { if (!isAbortError(err)) setError(err instanceof Error ? err.message : t('common.error')); });

    // Admin: auto-load all groups (incl. private) without requiring a search
    if (isAdmin()) {
      api.get<GroupResult[]>('/api/v1/groups', controller.signal)
        .then(setSearchResults)
        .catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    }

    api.get<Session[]>('/api/v1/sessions', controller.signal).then((sessions) => {
      const now = new Date();
      const map = new Map<string, Session[]>();
      for (const s of sessions) {
        if (new Date(s.scheduled_at) <= now) continue;
        for (const gid of s.group_ids) {
          if (!map.has(gid)) map.set(gid, []);
          map.get(gid)!.push(s);
        }
      }
      setSessionsByGroup(map);
    }).catch((err: unknown) => { if (!isAbortError(err)) console.error(err); });
    return () => controller.abort();
  }, [t]);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      setSearchResults(await api.get<GroupResult[]>(`/api/v1/groups?q=${encodeURIComponent(searchQuery)}`));
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      const group = await api.post<GroupSummary>('/api/v1/groups', { name: newName, is_public: newIsPublic });
      setMyGroups((p) => [...p, group]);
      setNewName(''); setShowCreateForm(false);
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const handleJoin = async (group: GroupResult) => {
    setError(null);
    try {
      await api.post(`/api/v1/groups/${group.id}/join`);
      setJoined((p) => new Set(p).add(group.id));
      setMyGroups((p) => [...p, { id: group.id, name: group.name, is_public: group.is_public }]);
      setSearchResults((p) => p.map((g) => g.id === group.id ? { ...g, member_count: g.member_count + 1 } : g));
    } catch (err) { setError(err instanceof Error ? err.message : t('common.error')); }
  };

  const myGroupIds = new Set(myGroups.map((g) => g.id));

  return (
    <PageLayout>
      {error && <ErrorBanner message={error} />}

      {/* Meine Gruppen */}
      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <SectionLabel data-testid="section-my-groups" count={myGroups.length}>{t('groups.my_groups')}</SectionLabel>
          <Button variant="secondary" onClick={() => setShowCreateForm((v) => !v)}>
            {showCreateForm ? t('common.cancel') : t('groups.create')}
          </Button>
        </div>

        {showCreateForm && (
          <form onSubmit={handleCreate} className="bg-ui-surface border border-ui-border rounded-xl p-4 flex flex-wrap gap-3 items-end">
            <div className="flex-1 min-w-40 space-y-1.5">
              <SectionLabel>{t('common.edit')}</SectionLabel>
              <Input value={newName} onChange={(e) => setNewName(e.target.value)} placeholder={t('groups.group_name_placeholder')} required className="w-full" />
            </div>
            <label className="flex items-center gap-2 text-sm text-zinc-300 cursor-pointer">
              <input type="checkbox" checked={newIsPublic} onChange={(e) => setNewIsPublic(e.target.checked)} className="accent-violet-500" />
              {t('groups.is_public_label')}
            </label>
            <Button type="submit">{t('common.create')}</Button>
          </form>
        )}

        {myGroups.length === 0 ? (
          <p className="text-zinc-500 text-sm">{t('groups.no_groups')}</p>
        ) : (
          <ul className="divide-y divide-zinc-800 bg-ui-surface border border-ui-border rounded-xl overflow-hidden">
            {myGroups.map((g) => (
              <li key={g.id} className="px-4 py-3">
                <div className="flex items-center justify-between">
                  <Link to={`/groups/${g.id}`} className="text-sm font-medium text-zinc-100 hover:text-violet-400 transition-colors">
                    {g.name}
                  </Link>
                  <Badge variant={g.is_public ? 'public' : 'private'} />
                </div>
                {(sessionsByGroup.get(g.id) ?? []).length > 0 && (
                  <ul className="mt-2 space-y-1">
                    {(sessionsByGroup.get(g.id) ?? []).map((s) => (
                      <li key={s.id} className="flex items-center gap-2 text-xs text-zinc-500">
                        <span className="w-1 h-1 rounded-full bg-violet-500 shrink-0" />
                        {fmtSession(s)}
                      </li>
                    ))}
                  </ul>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>

      <hr className="border-ui-border" />

      {/* Suche */}
      <section className="space-y-3">
        <SectionLabel>{isAdmin() ? t('groups.search_title_admin') : t('groups.search_title')}</SectionLabel>
        <SearchBar value={searchQuery} onChange={setSearchQuery} onSubmit={handleSearch} placeholder={t('groups.search_placeholder')} />

        {searchResults.length > 0 && (
          <ul className="divide-y divide-zinc-800 bg-ui-surface border border-ui-border rounded-xl overflow-hidden">
            {searchResults.map((g) => {
              const isMember = myGroupIds.has(g.id) || joined.has(g.id);
              return (
                <li key={g.id} className="flex items-center justify-between px-4 py-3 gap-4">
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <Link to={`/groups/${g.id}`} className="text-sm font-medium text-zinc-100 hover:text-violet-400 transition-colors truncate">{g.name}</Link>
                      <Badge variant={g.is_public ? 'public' : 'private'} />
                    </div>
                    <p className="text-xs text-zinc-500 mt-0.5">
                      {t('groups.member_count', { count: g.member_count })}
                    </p>
                  </div>
                  {isMember
                    ? <Badge variant="joined" />
                    : <Button size="sm" onClick={() => handleJoin(g)}>{t('common.join')}</Button>
                  }
                </li>
              );
            })}
          </ul>
        )}
        {searchResults.length === 0 && searchQuery && <p className="text-zinc-500 text-sm">{t('groups.no_results')}</p>}
      </section>
    </PageLayout>
  );
}
