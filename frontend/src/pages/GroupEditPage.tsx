import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate, useParams } from 'react-router-dom';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import ErrorBanner from '../components/molecules/ErrorBanner';
import PageLayout from '../components/templates/PageLayout';
import SectionLabel from '../components/atoms/SectionLabel';
import { api } from '../services/api';
import { isAbortError } from '../utils/abort';
import { useAuth } from 'react-oidc-context';
import { isAdmin } from '../utils/auth';

interface GroupDetail {
  id: string;
  name: string;
  is_public: boolean;
  creator_id: string | null;
  discord_invite: string | null;
}

export default function GroupEditPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [group, setGroup] = useState<GroupDetail | null>(null);
  const [discordInvite, setDiscordInvite] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const auth = useAuth();
  const myId = auth.user?.profile.sub;

  useEffect(() => {
    if (!id) return;
    const controller = new AbortController();
    api.get<GroupDetail>(`/api/v1/groups/${id}`, controller.signal)
      .then((g) => {
        // Redirect away if not creator or admin
        if (g.creator_id !== myId && !isAdmin(auth.user)) {
          navigate(`/groups/${id}`);
          return;
        }
        setGroup(g);
        setDiscordInvite(g.discord_invite ?? '');
      })
      .catch((err: unknown) => { if (!isAbortError(err)) setError(t('group_edit.not_found')); });
    return () => controller.abort();
  }, [id, myId, navigate, t, auth.user]);

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
      setError(err instanceof Error ? err.message : t('group_edit.save_error'));
    } finally {
      setSaving(false);
    }
  };

  if (!group && !error) {
    return (
      <PageLayout>
        <p className="text-zinc-500 text-sm">{t('group_edit.loading')}</p>
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
            {t('group_edit.back_to_group')}
          </button>
          <h1 className="text-xl font-bold text-zinc-100">
            {t('group_edit.title', { name: group?.name })}
          </h1>
        </div>

        {error && <ErrorBanner message={error} />}

        {saved && (
          <p className="text-sm text-green-400">{t('group_edit.saved')}</p>
        )}

        <section className="bg-ui-surface border border-ui-border rounded-xl p-5 space-y-4">
          <SectionLabel>{t('group_edit.discord_section')}</SectionLabel>

          <div className="space-y-2">
            <label className="text-xs text-zinc-400">
              {t('group_edit.discord_invite_label')}
            </label>
            <Input
              value={discordInvite}
              onChange={(e) => { setDiscordInvite(e.target.value); setSaved(false); }}
              placeholder={t('group_edit.discord_invite_placeholder')}
              className="w-full font-mono text-sm"
            />
            <p className="text-xs text-zinc-600">
              {t('group_edit.discord_invite_hint')}
            </p>
          </div>

          <div className="flex gap-3">
            <Button onClick={handleSave} disabled={saving}>
              {saving ? t('group_edit.saving') : t('group_edit.save')}
            </Button>
            <Button variant="secondary" onClick={() => navigate(`/groups/${id}`)}>
              {t('group_edit.cancel')}
            </Button>
          </div>
        </section>
      </div>
    </PageLayout>
  );
}
