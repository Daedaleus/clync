import { Link } from 'react-router-dom';
import Button from '../atoms/Button';
import type { Game } from '../../types';
import { config } from '../../config';

interface Props {
  game: Game;
  inWishlist?: boolean;
  onToggle?: () => void;
  className?: string;
}

function thumbnailSrc(name: string, url: string | null | undefined): string | undefined {
  if (!url) return undefined;
  if (url.startsWith('http')) return url;
  return `${config.apiUrl}/api/v1/library/${encodeURIComponent(name)}/thumbnail`;
}

export default function GameCard({ game, inWishlist, onToggle, className = '' }: Props) {
  const src = thumbnailSrc(game.name, game.thumbnail_url);
  return (
    <div className={`bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden hover:border-zinc-700 transition-colors flex flex-col ${className}`}>
      <Link to={`/library/${encodeURIComponent(game.name)}`} className="block shrink-0">
        {src ? (
          <img
            src={src}
            alt={game.name}
            className="w-full h-32 object-cover"
            onError={(e) => {
              const el = e.target as HTMLImageElement;
              el.style.display = 'none';
              el.nextElementSibling?.removeAttribute('style');
            }}
          />
        ) : null}
        <div
          className="w-full h-32 bg-zinc-800 items-center justify-center text-3xl text-zinc-600"
          style={{ display: src ? 'none' : 'flex' }}
        >
          🎮
        </div>
      </Link>

      <div className="p-3 flex flex-col gap-2 flex-1">
        <div className="flex-1 min-w-0">
          <Link
            to={`/library/${encodeURIComponent(game.name)}`}
            className="text-sm font-semibold text-zinc-100 hover:text-violet-300 transition-colors line-clamp-1 block"
          >
            {game.name}
          </Link>
          {game.genre && (
            <p className="text-xs text-zinc-500 mt-0.5 truncate">{game.genre}</p>
          )}
        </div>
        {onToggle !== undefined && (
          <Button
            size="sm"
            variant={inWishlist ? 'secondary' : 'primary'}
            onClick={onToggle}
            className="w-full"
          >
            {inWishlist ? '✓ In meiner Liste' : '+ Zur Liste'}
          </Button>
        )}
      </div>
    </div>
  );
}
