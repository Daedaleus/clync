import { useEffect, useMemo, useState } from 'react';
import Button from '../components/atoms/Button';
import SectionLabel from '../components/atoms/SectionLabel';
import ErrorBanner from '../components/molecules/ErrorBanner';
import SessionCreateForm from '../components/organisms/SessionCreateForm';
import SessionList from '../components/organisms/SessionList';
import PageLayout from '../components/templates/PageLayout';
import type { GroupSummary, RsvpStatus, Session } from '../types';
import { api } from '../services/api';
import { isPast, isLateJoinable } from '../utils/date';

export default function SessionsPage() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [myGroups, setMyGroups] = useState<GroupSummary[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.get<Session[]>('/api/v1/sessions').then(setSessions).catch(console.error);
    api.get<GroupSummary[]>('/api/v1/groups/mine').then(setMyGroups).catch(console.error);
  }, []);

  const upcoming = useMemo(
    () => sessions.filter((s) => !isPast(s.scheduled_at) || isLateJoinable(s.scheduled_at)),
    [sessions],
  );
  const groupSessions = useMemo(() => upcoming.filter((s) => s.scope === 'groups'), [upcoming]);
  const globalSessions = useMemo(() => upcoming.filter((s) => s.scope === 'global'), [upcoming]);

  const handleRsvp = async (id: string, status: RsvpStatus | null) => {
    try {
      if (status === null) {
        await api.delete(`/api/v1/sessions/${id}/rsvp`);
        setSessions((p) => p.map((s) => s.id === id
          ? { ...s, my_rsvp: null, is_participant: false, participant_count: s.is_participant ? s.participant_count - 1 : s.participant_count }
          : s));
      } else {
        await api.put(`/api/v1/sessions/${id}/rsvp`, { status });
        const wasParticipant = sessions.find((s) => s.id === id)?.is_participant ?? false;
        const willBeParticipant = status === 'accepted';
        setSessions((p) => p.map((s) => s.id === id
          ? {
              ...s,
              my_rsvp: status,
              is_participant: willBeParticipant,
              participant_count:
                willBeParticipant && !wasParticipant ? s.participant_count + 1
                : !willBeParticipant && wasParticipant ? s.participant_count - 1
                : s.participant_count,
            }
          : s));
      }
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.delete(`/api/v1/sessions/${id}`);
      setSessions((p) => p.filter((s) => s.id !== id));
    } catch (err) { setError(err instanceof Error ? err.message : 'Fehler'); }
  };

  const handleSessionCreated = (session: Session) => {
    setSessions((p) => [...p, session].sort((a, b) => a.scheduled_at.localeCompare(b.scheduled_at)));
    setShowForm(false);
  };

  return (
    <PageLayout>
      <div className="flex items-center justify-between">
        <SectionLabel>Sessions</SectionLabel>
        <Button variant="secondary" onClick={() => setShowForm((v) => !v)}>
          {showForm ? 'Abbrechen' : '+ Neue Zeit'}
        </Button>
      </div>

      {error && <ErrorBanner message={error} />}

      {showForm && (
        <SessionCreateForm
          groups={myGroups}
          onCreate={handleSessionCreated}
          onError={setError}
          onCancel={() => setShowForm(false)}
        />
      )}

      <section className="space-y-2">
        <SectionLabel>Gruppen</SectionLabel>
        <SessionList
          sessions={groupSessions}
          onRsvp={handleRsvp}
          onDelete={handleDelete}
          emptyMessage="Keine bevorstehenden Gruppen-Sessions."
        />
      </section>

      <section className="space-y-2">
        <SectionLabel>Global</SectionLabel>
        <SessionList
          sessions={globalSessions}
          onRsvp={handleRsvp}
          onDelete={handleDelete}
          emptyMessage="Keine bevorstehenden globalen Sessions."
        />
      </section>
    </PageLayout>
  );
}
