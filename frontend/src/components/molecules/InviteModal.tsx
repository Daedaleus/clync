import { useState } from 'react';
import Button from '../atoms/Button';

interface Props {
  url: string;
  onClose: () => void;
}

export default function InviteModal({ url, onClose }: Props) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(url);
    } catch {
      // Fallback: select the input text
      const input = document.getElementById('invite-url-input') as HTMLInputElement | null;
      input?.select();
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 2500);
  };

  return (
    /* Backdrop */
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-zinc-950/80 p-4"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="w-full max-w-md bg-zinc-900 border border-zinc-800 rounded-2xl p-6 space-y-5 shadow-2xl">
        {/* Header */}
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold text-zinc-100">Freund einladen</h2>
            <p className="text-xs text-zinc-500 mt-0.5">Link ist 7 Tage gültig und kann nur einmal benutzt werden.</p>
          </div>
          <button
            onClick={onClose}
            className="text-zinc-600 hover:text-zinc-400 transition-colors text-lg leading-none mt-0.5"
          >
            ✕
          </button>
        </div>

        {/* URL display */}
        <div className="flex gap-2">
          <input
            id="invite-url-input"
            readOnly
            value={url}
            onClick={(e) => (e.target as HTMLInputElement).select()}
            className="flex-1 min-w-0 bg-zinc-800 border border-zinc-700 text-xs text-zinc-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-violet-500 cursor-text select-all"
          />
          <Button onClick={copy} variant={copied ? 'secondary' : 'primary'}>
            {copied ? '✓ Kopiert' : 'Kopieren'}
          </Button>
        </div>

        <p className="text-xs text-zinc-600 text-center">
          Schick diesen Link an die Person, die du einladen möchtest. Sie werden nach Nutzername und Passwort gefragt.
        </p>
      </div>
    </div>
  );
}
