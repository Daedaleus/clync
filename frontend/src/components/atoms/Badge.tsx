import { useTranslation } from 'react-i18next';

export type BadgeVariant = 'public' | 'private' | 'friend' | 'global' | 'groups' | 'joined';

const styles: Record<BadgeVariant, string> = {
  public:  'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
  private: 'bg-zinc-800 text-zinc-400 border-zinc-700',
  friend:  'bg-violet-500/10 text-violet-400 border-violet-500/20',
  global:  'bg-blue-500/10 text-blue-400 border-blue-500/20',
  groups:  'bg-violet-500/10 text-violet-400 border-violet-500/20',
  joined:  'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
};

interface Props {
  variant: BadgeVariant;
  label?: string;
}

export default function Badge({ variant, label }: Props) {
  const { t } = useTranslation();
  return (
    <span className={`text-xs px-2 py-0.5 rounded-full border ${styles[variant]}`}>
      {label ?? t(`badge.${variant}`)}
    </span>
  );
}
