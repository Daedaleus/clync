import { StrictMode } from 'react';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App.tsx';
import JoinPage from './pages/JoinPage.tsx';
import keycloak from './services/auth.ts';

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch((err) => {
      console.warn('Service Worker registration failed:', err);
    });
  });
}

// /join/:token is public — render it before Keycloak initialises so the
// invitee never gets bounced to the login screen.
if (window.location.pathname.startsWith('/join/')) {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <BrowserRouter>
        <Routes>
          <Route path="/join/:token" element={<JoinPage />} />
        </Routes>
      </BrowserRouter>
    </StrictMode>,
  );
} else {
  keycloak
    .init({ onLoad: 'login-required' })
    .then(() => {
      keycloak.onTokenExpired = () => {
        keycloak.updateToken(30).catch(() => keycloak.login());
      };

      createRoot(document.getElementById('root')!).render(
        <StrictMode>
          <App />
        </StrictMode>,
      );
    })
    .catch((err) => {
      console.error('Keycloak initialization failed', err);
    });
}
