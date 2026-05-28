import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useParams } from 'react-router-dom';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import ErrorBanner from '../components/molecules/ErrorBanner';
import { config } from '../config';

export default function JoinPage() {
  const { t } = useTranslation();
  const { token } = useParams<{ token: string }>();

  const [valid, setValid] = useState<boolean | null>(null);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [done, setDone] = useState(false);

  useEffect(() => {
    if (!token) return;
    const controller = new AbortController();
    fetch(`${config.apiUrl}/api/v1/invites/${token}`, { signal: controller.signal })
      .then((r) => r.json())
      .then((d: { valid: boolean }) => setValid(d.valid))
      .catch((err: unknown) => {
        if (err instanceof Error && err.name === 'AbortError') return;
        setValid(false);
      });
    return () => controller.abort();
  }, [token]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (password !== confirm) { setError(t('join.error_password_mismatch')); return; }
    if (password.length < 6) { setError(t('join.error_password_short')); return; }

    setLoading(true);
    try {
      const resp = await fetch(`${config.apiUrl}/api/v1/invites/${token}/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      });

      if (!resp.ok) {
        try {
          const body = await resp.json();
          setError(typeof body.error === 'string' ? body.error : t('join.error_registration_failed'));
        } catch {
          setError(t('join.error_registration_failed'));
        }
        return;
      }

      setDone(true);
    } catch { setError(t('join.error_network')); }
    finally { setLoading(false); }
  };

  if (valid === null) {
    return (
      <div className="min-h-screen bg-zinc-950 flex items-center justify-center">
        <p className="text-zinc-500 text-sm">{t('join.checking')}</p>
      </div>
    );
  }

  if (!valid) {
    return (
      <div className="min-h-screen bg-zinc-950 flex items-center justify-center p-4">
        <div className="w-full max-w-sm text-center space-y-4">
          <p className="text-4xl">🔗</p>
          <h1 className="text-xl font-bold text-zinc-100">{t('join.invalid_title')}</h1>
          <p className="text-sm text-zinc-500">{t('join.invalid_subtitle')}</p>
        </div>
      </div>
    );
  }

  if (done) {
    return (
      <div className="min-h-screen bg-zinc-950 flex items-center justify-center p-4">
        <div className="w-full max-w-sm text-center space-y-4">
          <p className="text-4xl">🎉</p>
          <h1 className="text-xl font-bold text-zinc-100">{t('join.done_title')}</h1>
          <p className="text-sm text-zinc-500">{t('join.done_subtitle')}</p>
          <Button onClick={() => { window.location.href = '/'; }} className="w-full">{t('join.done_cta')}</Button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950 flex items-center justify-center p-4">
      <div className="w-full max-w-sm space-y-6">
        <div className="text-center space-y-2">
          <p className="text-4xl">👋</p>
          <h1 className="text-2xl font-bold text-zinc-100">{t('join.welcome_title')}</h1>
          <p className="text-sm text-zinc-500">{t('join.welcome_subtitle')}</p>
        </div>

        {error && <ErrorBanner message={error} />}

        <form onSubmit={handleSubmit} className="space-y-3">
          <Input
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder={t('join.username_placeholder')}
            required
            className="w-full"
          />
          <Input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder={t('join.password_placeholder')}
            required
            className="w-full"
          />
          <Input
            type="password"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            placeholder={t('join.confirm_placeholder')}
            required
            className="w-full"
          />
          <Button type="submit" disabled={loading} className="w-full">
            {loading ? t('join.submitting') : t('join.submit')}
          </Button>
        </form>
      </div>
    </div>
  );
}
