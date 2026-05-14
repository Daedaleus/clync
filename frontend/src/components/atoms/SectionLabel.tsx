interface Props {
  children: React.ReactNode;
  count?: number;
}

export default function SectionLabel({ children, count }: Props) {
  return (
    <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wider">
      {children}
      {count !== undefined && (
        <span className="ml-1.5 text-zinc-600 normal-case font-normal">({count})</span>
      )}
    </p>
  );
}
