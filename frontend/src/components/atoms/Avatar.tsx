type Size = 'sm' | 'md' | 'lg';

const sizes: Record<Size, string> = {
  sm: 'w-7 h-7 text-xs',
  md: 'w-9 h-9 text-sm',
  lg: 'w-14 h-14 text-2xl',
};

interface Props {
  name: string;
  size?: Size;
}

export default function Avatar({ name, size = 'md' }: Props) {
  return (
    <div className={`${sizes[size]} rounded-full bg-violet-600 flex items-center justify-center font-semibold text-white shrink-0`}>
      {name.charAt(0).toUpperCase()}
    </div>
  );
}
