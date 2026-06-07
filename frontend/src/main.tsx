import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { AuthProvider } from 'react-oidc-context';
import './index.css';
import './i18n';
import App from './App.tsx';
import { ErrorBoundary } from './components/ErrorBoundary.tsx';
import { userManager } from './services/auth.ts';

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch((err) => {
      console.warn('Service Worker registration failed:', err);
    });
  });
}

// Strips the OIDC code/state params Keycloak appends to the redirect URI —
// without this, subsequent silent token renewals fail.
const onSigninCallback = (): void => {
  window.history.replaceState({}, document.title, window.location.pathname);
};

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary>
      <AuthProvider userManager={userManager} onSigninCallback={onSigninCallback}>
        <App />
      </AuthProvider>
    </ErrorBoundary>
  </StrictMode>,
);
