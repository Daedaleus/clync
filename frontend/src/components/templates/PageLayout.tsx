import AppHeader from '../organisms/AppHeader';

interface Props {
  children: React.ReactNode;
  /** 'md' = max-w-2xl (default), 'lg' = max-w-3xl */
  size?: 'md' | 'lg';
}

export default function PageLayout({ children, size = 'md' }: Props) {
  const maxW = size === 'lg' ? 'max-w-3xl' : 'max-w-2xl';
  return (
    <div className="min-h-dvh bg-zinc-950">
      <AppHeader />
      <main className={`mx-auto ${maxW} px-4 py-8 space-y-8`}>
        {children}
      </main>
    </div>
  );
}
