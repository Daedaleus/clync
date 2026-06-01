import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import AppHeader from '../organisms/AppHeader';

interface Props {
  children: React.ReactNode;
  /** 'md' = max-w-2xl (default), 'lg' = max-w-3xl */
  size?: 'md' | 'lg';
}

export default function PageLayout({ children, size = 'md' }: Props) {
  const { t } = useTranslation();
  const maxW = size === 'lg' ? 'max-w-3xl' : 'max-w-2xl';
  return (
    <div className="min-h-dvh bg-app flex flex-col">
      <AppHeader />
      <main className={`mx-auto w-full ${maxW} px-4 py-8 space-y-8 flex-1`}>
        {children}
      </main>
      <footer className={`mx-auto w-full ${maxW} px-4 py-4 flex items-center justify-between border-t border-ui-border/60`}>
        <span className="text-xs text-zinc-600">
          {t('footer.version', { version: __APP_VERSION__ })}
        </span>
        <Link to="/about" className="text-xs text-zinc-600 hover:text-zinc-400 transition-colors">
          {t('footer.about')}
        </Link>
      </footer>
    </div>
  );
}
