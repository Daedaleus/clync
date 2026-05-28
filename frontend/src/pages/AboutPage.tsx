import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import PageLayout from '../components/templates/PageLayout';

export default function AboutPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <PageLayout>
      <div className="space-y-6 max-w-prose">
        <div>
          <h1 className="text-2xl font-bold text-zinc-100">{t('about.title')}</h1>
          <p className="mt-2 text-zinc-400 text-sm italic">{t('about.tagline')}</p>
        </div>

        <section className="space-y-2">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-zinc-500">{t('about.section_what')}</h2>
          <p className="text-sm text-zinc-300 leading-relaxed">{t('about.what')}</p>
        </section>

        <section className="space-y-2">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-zinc-500">{t('about.section_features')}</h2>
          <ul className="space-y-1.5 text-sm text-zinc-300">
            {(['feature_sessions', 'feature_library', 'feature_groups', 'feature_friends', 'feature_notifications'] as const).map((key) => (
              <li key={key} className="flex items-start gap-2">
                <span className="text-violet-400 mt-0.5">·</span>
                {t(`about.${key}`)}
              </li>
            ))}
          </ul>
        </section>

        <section className="space-y-2">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-zinc-500">{t('about.section_status')}</h2>
          <p className="text-sm text-zinc-400 leading-relaxed">{t('about.status')}</p>
          <p className="text-xs text-zinc-600">
            {t('footer.version', { version: __APP_VERSION__ })}
          </p>
        </section>

        <button
          onClick={() => navigate(-1)}
          className="text-sm text-zinc-500 hover:text-zinc-300 transition-colors cursor-pointer"
        >
          {t('about.back')}
        </button>
      </div>
    </PageLayout>
  );
}
