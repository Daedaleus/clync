import { type ButtonHTMLAttributes } from 'react';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md';

const variants: Record<ButtonVariant, string> = {
  primary:   'bg-violet-600 hover:bg-violet-500 text-white rounded-lg',
  secondary: 'bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded-lg',
  ghost:     'text-zinc-500 hover:text-zinc-200',
  danger:    'text-zinc-600 hover:text-red-400',
};

const sizes: Record<ButtonSize, string> = {
  sm: 'text-xs px-3 py-1.5',
  md: 'text-sm px-4 py-2',
};

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

export default function Button({ variant = 'primary', size = 'md', className = '', ...props }: Props) {
  return (
    <button
      {...props}
      className={`font-medium transition-colors cursor-pointer disabled:opacity-50 ${variants[variant]} ${sizes[size]} ${className}`}
    />
  );
}
