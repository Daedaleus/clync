import { useTranslation } from 'react-i18next';

export type Visibility = 'public' | 'group' | 'friends';

interface Props {
  value: Visibility;
  onChange: (v: Visibility) => void;
  id?: string;
}

export default function VisibilitySelect({ value, onChange, id }: Props) {
  const { t } = useTranslation();
  return (
    <select
      id={id}
      value={value}
      onChange={(e) => onChange(e.target.value as Visibility)}
      className="bg-zinc-800 border border-zinc-700 text-xs text-zinc-300 rounded-lg px-2 py-1.5 focus:outline-none focus:ring-1 focus:ring-violet-500 cursor-pointer"
    >
      <option value="public">{t('visibility.public')}</option>
      <option value="group">{t('visibility.group')}</option>
      <option value="friends">{t('visibility.friends')}</option>
    </select>
  );
}
