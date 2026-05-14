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
  placeholder = 'Suchen…',
  buttonLabel = 'Suchen',
}: Props) {
  return (
    <form onSubmit={onSubmit} className="flex gap-2">
      <Input
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="flex-1"
      />
      <Button type="submit">{buttonLabel}</Button>
    </form>
  );
}
