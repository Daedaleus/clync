interface Props {
  name: string;
  onRemove?: () => void;
}

export default function GameChip({ name, onRemove }: Props) {
  return (
    <span className="flex items-center gap-1.5 bg-ui-raised border border-ui-border rounded-full px-3 py-1 text-sm text-zinc-200">
      {name}
      {onRemove && (
        <button
          onClick={onRemove}
          aria-label={`${name} entfernen`}
          className="text-zinc-500 hover:text-zinc-200 transition-colors cursor-pointer leading-none"
        >
          ×
        </button>
      )}
    </span>
  );
}
