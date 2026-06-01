import { useTranslation } from 'react-i18next';
import Button from '../atoms/Button';
import type { AutofillCandidate } from './AutofillPicker';

interface Props {
  candidates: AutofillCandidate[];
  loading: boolean;
  addedNames: Set<string>;
  busyName: string | null;
  onAdd: (candidate: AutofillCandidate) => void;
  onClose: () => void;
}

export default function GameSearchResults({
  candidates,
  loading,
  addedNames,
  busyName,
  onAdd,
  onClose,
}: Props) {
  const { t } = useTranslation();

  if (loading) {
    return (
      <div className="bg-ui-surface border border-ui-border rounded-xl p-4 text-center text-sm text-zinc-500">
        {t('autofill.searching')}
      </div>
    );
  }

  if (candidates.length === 0) {
    return (
      <div className="bg-ui-surface border border-ui-border rounded-xl p-4 flex items-center justify-between gap-3">
        <p className="text-sm text-zinc-500">{t('autofill.no_results')}</p>
        <Button size="sm" variant="secondary" onClick={onClose}>{t('autofill.close')}</Button>
      </div>
    );
  }

  return (
    <ul className="flex flex-col gap-2">
      {candidates.map((c) => {
        const added = addedNames.has(c.rawg_name);
        const busy = busyName === c.rawg_name;
        return (
          <li
            key={c.rawg_id}
            className="flex items-center gap-3 rounded-xl border border-ui-border bg-ui-surface p-2"
          >
            {c.thumbnail_url ? (
              <img
                src={c.thumbnail_url}
                alt={c.rawg_name}
                className="h-16 w-12 shrink-0 rounded-md object-cover"
                onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
              />
            ) : (
              <div className="h-16 w-12 shrink-0 rounded-md bg-ui-raised flex items-center justify-center text-xl text-zinc-500">
                🎮
              </div>
            )}
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium text-zinc-200">{c.rawg_name}</p>
              {c.genre && (
                <p className="truncate text-xs text-zinc-500">{c.genre}</p>
              )}
            </div>
            <Button
              size="sm"
              variant={added ? 'secondary' : 'primary'}
              disabled={added || busy}
              onClick={() => onAdd(c)}
            >
              {busy ? t('common.loading') : added ? t('library.game_added') : t('library.add_game')}
            </Button>
          </li>
        );
      })}
    </ul>
  );
}
