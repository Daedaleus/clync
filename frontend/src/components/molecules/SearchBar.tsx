import { useTranslation } from 'react-i18next';
import Button from '../atoms/Button';
import Input from '../atoms/Input';

interface Props {
  value: string;
  onChange: (v: string) => void;
  onSubmit: (e: React.FormEvent) => void;
  placeholder?: string;
  buttonLabel?: string;
}

export default function SearchBar({
  value, onChange, onSubmit,
  placeholder,
  buttonLabel,
}: Props) {
  const { t } = useTranslation();
  return (
    <form onSubmit={onSubmit} className="flex gap-2">
      <Input
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder ?? t('search_bar.placeholder')}
        className="flex-1"
      />
      <Button type="submit">{buttonLabel ?? t('search_bar.button')}</Button>
    </form>
  );
}
