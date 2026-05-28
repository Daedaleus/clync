import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import logo from '../../assets/logo.svg';
import { Link, NavLink } from 'react-router-dom';
import keycloak from '../../services/auth';
import { api } from '../../services/api';
import { useNotifications } from '../../hooks/useNotifications';
import InviteModal from '../molecules/InviteModal';

// ── Icons ────────────────────────────────────────────────────────────────────

function BellIcon({ active }: { active: boolean }) {
  return (
    <svg
      width="17" height="17" viewBox="0 0 24 24"
      fill={active ? 'rgba(167,139,250,0.18)' : 'none'}
      stroke="currentColor" strokeWidth="2"
      strokeLinecap="round" strokeLinejoin="round"
      className={active ? 'text-violet-400' : 'text-zinc-500'}
    >
      <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
      <path d="M13.73 21a2 2 0 0 1-3.46 0" />
    </svg>
  );
}

function ChevronIcon({ open }: { open: boolean }) {
  return (
    <svg
      width="11" height="11" viewBox="0 0 24 24"
      fill="none" stroke="currentColor" strokeWidth="2.5"
      strokeLinecap="round" strokeLinejoin="round"
      className={`transition-transform duration-150 ${open ? 'rotate-180' : ''}`}
    >
      <polyline points="6 9 12 15 18 9" />
    </svg>
  );
}

// ── Sub-components ────────────────────────────────────────────────────────────

const navLink = ({ isActive }: { isActive: boolean }) =>
  `text-sm transition-colors ${isActive ? 'text-zinc-100 font-medium' : 'text-zinc-400 hover:text-zinc-200'}`;

const dropdownItem =
  'flex items-center gap-2.5 w-full text-left px-4 py-2 text-sm text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100 transition-colors cursor-pointer';

// ── AppHeader ─────────────────────────────────────────────────────────────────

export default function AppHeader() {
  const { t, i18n } = useTranslation();
  const username = keycloak.tokenParsed?.preferred_username as string | undefined;
  const { permission, subscribed, enable, disable } = useNotifications();
  const [menuOpen, setMenuOpen] = useState(false);
  const [inviteUrl, setInviteUrl] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const showBell = permission !== 'unsupported' && permission !== 'denied';

  // Close dropdown on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  const handleBellToggle = () => {
    if (subscribed) { disable(); } else { enable(); }
    setMenuOpen(false);
  };

  const close = () => setMenuOpen(false);

  const handleInvite = async () => {
    close();
    try {
      const { token } = await api.post<{ token: string }>('/api/v1/invites');
      setInviteUrl(`${window.location.origin}/join/${token}`);
    } catch { /* silently ignore */ }
  };

  return (
    <>
    <header className="sticky top-0 z-10 bg-zinc-950/90 backdrop-blur-sm border-b border-zinc-800">
      <div className="mx-auto max-w-2xl px-4 h-14 flex items-center justify-between gap-4">

        {/* Left: logo + desktop nav */}
        <div className="flex items-center gap-5 min-w-0">
          <Link to="/" className="shrink-0">
            <img src={logo} alt="WhatsUp" className="h-7 w-auto" />
          </Link>
          <nav className="hidden sm:flex items-center gap-4">
            <NavLink to="/sessions" className={navLink}>{t('nav.sessions')}</NavLink>
            <NavLink to="/groups" className={navLink}>{t('nav.groups')}</NavLink>
            <NavLink to="/library" className={navLink}>{t('nav.library')}</NavLink>
            <NavLink to="/friends" className={navLink}>{t('nav.friends')}</NavLink>
          </nav>
        </div>

        {/* Right: bell (desktop) + username dropdown */}
        <div className="flex items-center gap-3 shrink-0">

          {/* Bell – desktop only, inline */}
          {showBell && (
            <button
              onClick={handleBellToggle}
              title={subscribed ? t('header.notifications_disable') : t('header.notifications_enable')}
              className="hidden sm:flex items-center justify-center w-8 h-8 rounded-lg hover:bg-zinc-800 transition-colors cursor-pointer"
            >
              <BellIcon active={subscribed} />
            </button>
          )}

          {/* Username + dropdown */}
          <div className="relative" ref={menuRef}>
            <button
              onClick={() => setMenuOpen((v) => !v)}
              className="flex items-center gap-1.5 text-sm text-zinc-300 hover:text-zinc-100 transition-colors cursor-pointer"
            >
              <span className="truncate max-w-24">{username ?? '...'}</span>
              <ChevronIcon open={menuOpen} />
            </button>

            {menuOpen && (
              <div className="absolute right-0 top-full mt-2 bg-zinc-900 border border-zinc-800 rounded-xl shadow-xl min-w-44 py-1 z-20">

                {/* Mobile-only nav items */}
                <div className="sm:hidden">
                  <Link to="/sessions" className={dropdownItem} onClick={close}>{t('nav.sessions')}</Link>
                  <Link to="/groups" className={dropdownItem} onClick={close}>{t('nav.groups')}</Link>
                  <Link to="/library" className={dropdownItem} onClick={close}>{t('nav.library')}</Link>
                  <Link to="/friends" className={dropdownItem} onClick={close}>{t('nav.friends')}</Link>
                  {showBell && (
                    <button className={dropdownItem} onClick={handleBellToggle}>
                      <BellIcon active={subscribed} />
                      <span>{subscribed ? t('header.notifications_on') : t('header.notifications_off')}</span>
                    </button>
                  )}
                  <hr className="border-zinc-800 my-1" />
                </div>

                {/* Always: Profil + logout */}
                <Link to="/profile" className={dropdownItem} onClick={close}>{t('header.profile')}</Link>
                <button className={dropdownItem} onClick={handleInvite}>
                  {t('header.invite_friend')}
                </button>
                <hr className="border-zinc-800 my-1" />
                <button
                  className={dropdownItem}
                  onClick={() => { i18n.changeLanguage((i18n.resolvedLanguage ?? i18n.language).startsWith('de') ? 'en' : 'de'); close(); }}
                >
                  {(i18n.resolvedLanguage ?? i18n.language).startsWith('de') ? '🇬🇧 English' : '🇩🇪 Deutsch'}
                </button>
                <hr className="border-zinc-800 my-1" />
                <button
                  className={`${dropdownItem} text-zinc-400`}
                  onClick={() => keycloak.logout()}
                >
                  {t('header.logout')}
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>

    {inviteUrl && (
      <InviteModal url={inviteUrl} onClose={() => setInviteUrl(null)} />
    )}
  </>
  );
}
