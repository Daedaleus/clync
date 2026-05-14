import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import Button from '../components/atoms/Button';
import Input from '../components/atoms/Input';
import ErrorBanner from '../components/molecules/ErrorBanner';
import { config } from '../config';

export default function JoinPage() {
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
    fetch(`${config.apiUrl}/api/v1/invites/${token}`)
      .then((r) => r.json())
      .then((d: { valid: boolean }) => setValid(d.valid))
      .catch(() => setValid(false));
  }, [token]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (password !== confirm) { setError('Passwörter stimmen nicht überein.'); return; }
    if (password.length < 6) { setError('Passwort muss mindestens 6 Zeichen haben.'); return; }

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
          setError(typeof body.error === 'string' ? body.error : 'Registrierung fehlgeschlagen.');
        } catch {
          setError('Registrierung fehlgeschlagen.');
        }
        return;
      }

      setDone(true);
    } catch { setError('Netzwerkfehler. Bitte versuche es erneut.'); }
    finally { setLoading(false); }
  };

  if (valid === null) {
    return (
      <div className="min-h-screen bg-zinc-950 flex items-center justify-center">
        <p className="text-zinc-500 text-sm">Überprüfe Einladungslink…</p>
      </div>
    );
  }

  if (!valid) {
    return (
      <div className="min-h-screen bg-zinc-950 flex items-center justify-center p-4">
        <div className="w-full max-w-sm text-center space-y-4">
          <p className="text-4xl">🔗</p>
          <h1 className="text-xl font-bold text-zinc-100">Link ungültig</h1>
          <p className="text-sm text-zinc-500">Dieser Einladungslink ist abgelaufen oder wurde bereits verwendet.</p>
        </div>
      </div>
    );
  }

  if (done) {
    return (
      <div className="min-h-screen bg-zinc-950 flex items-center justify-center p-4">
        <div className="w-full max-w-sm text-center space-y-4">
          <p className="text-4xl">🎉</p>
          <h1 className="text-xl font-bold text-zinc-100">Konto erstellt!</h1>
          <p className="text-sm text-zinc-500">Du kannst dich jetzt mit deinen Zugangsdaten anmelden.</p>
          <Button onClick={() => { window.location.href = '/'; }} className="w-full">Zur Anmeldung</Button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950 flex items-center justify-center p-4">
      <div className="w-full max-w-sm space-y-6">
        <div className="text-center space-y-2">
          <p className="text-4xl">👋</p>
          <h1 className="text-2xl font-bold text-zinc-100">Du wurdest eingeladen!</h1>
          <p className="text-sm text-zinc-500">Wähle einen Nutzernamen und Passwort um loszulegen.</p>
        </div>

        {error && <ErrorBanner message={error} />}

        <form onSubmit={handleSubmit} className="space-y-3">
          <Input
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="Nutzername"
            required
            className="w-full"
          />
          <Input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Passwort (min. 6 Zeichen)"
            required
            className="w-full"
          />
          <Input
            type="password"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            placeholder="Passwort wiederholen"
            required
            className="w-full"
          />
          <Button type="submit" disabled={loading} className="w-full">
            {loading ? 'Konto wird erstellt…' : 'Konto erstellen'}
          </Button>
        </form>
      </div>
    </div>
  );
}
