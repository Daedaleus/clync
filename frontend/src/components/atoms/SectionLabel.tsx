interface Props {
  children: React.ReactNode;
  count?: number;
  'data-testid'?: string;
}

export default function SectionLabel({ children, count, 'data-testid': testId }: Props) {
  return (
    <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wider" data-testid={testId}>
      {children}
      {count !== undefined && (
        <span className="ml-1.5 text-zinc-600 normal-case font-normal">({count})</span>
      )}
    </p>
  );
}
