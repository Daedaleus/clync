import { useTranslation } from 'react-i18next';
import Button from '../atoms/Button';

export interface AutofillCandidate {
  rawg_id: number;
  rawg_name: string;
  thumbnail_url: string | null;
  genre: string | null;
}

interface Props {
  candidates: AutofillCandidate[];
  loading: boolean;
  onSelect: (candidate: AutofillCandidate) => void;
  onCancel: () => void;
}

export default function AutofillPicker({ candidates, loading, onSelect, onCancel }: Props) {
  const { t } = useTranslation();

  if (loading) {
    return (
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 text-center text-sm text-zinc-500">
        {t('autofill.searching')}
      </div>
    );
  }

  if (candidates.length === 0) {
    return (
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 flex items-center justify-between gap-3">
        <p className="text-sm text-zinc-500">{t('autofill.no_results')}</p>
        <Button size="sm" variant="secondary" onClick={onCancel}>{t('autofill.close')}</Button>
      </div>
    );
  }

  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
      <div className="flex items-center justify-between">
        <p className="text-sm font-medium text-zinc-300">{t('autofill.which_game')}</p>
        <button onClick={onCancel} className="text-zinc-600 hover:text-zinc-400 text-xs transition-colors">
          {t('autofill.cancel')}
        </button>
      </div>

      <div className="grid grid-cols-3 gap-2">
        {candidates.map((c) => (
          <button
            key={c.rawg_id}
            onClick={() => onSelect(c)}
            className="group text-left bg-zinc-800 border border-zinc-700 rounded-lg overflow-hidden hover:border-violet-500 transition-colors focus:outline-none focus:ring-1 focus:ring-violet-500"
          >
            {c.thumbnail_url ? (
              <img
                src={c.thumbnail_url}
                alt={c.rawg_name}
                className="w-full h-24 object-cover"
                onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
              />
            ) : (
              <div className="w-full h-24 bg-zinc-700 flex items-center justify-center text-2xl text-zinc-500">
                🎮
              </div>
            )}
            <div className="px-2 py-1.5">
              <p className="text-xs font-medium text-zinc-200 line-clamp-1 group-hover:text-violet-300 transition-colors">
                {c.rawg_name}
              </p>
              {c.genre && (
                <p className="text-xs text-zinc-600 truncate">{c.genre}</p>
              )}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
