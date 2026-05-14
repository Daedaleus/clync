import GameChip from '../molecules/GameChip';
import GameInput from '../molecules/GameInput';
import SectionLabel from '../atoms/SectionLabel';

interface Props {
  games: string[];
  onAdd?: (name: string) => void;
  onRemove?: (name: string) => void;
}

export default function GameSection({ games, onAdd, onRemove }: Props) {
  return (
    <section className="space-y-3">
      <SectionLabel count={games.length}>Spiele</SectionLabel>
      <div className="flex flex-wrap gap-2">
        {games.map((g) => (
          <GameChip key={g} name={g} onRemove={onRemove ? () => onRemove(g) : undefined} />
        ))}
        {games.length === 0 && <p className="text-zinc-500 text-sm">Noch keine Spiele.</p>}
      </div>
      {onAdd && <GameInput onAdd={onAdd} />}
    </section>
  );
}
