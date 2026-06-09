import { Suspense, lazy, useEffect } from 'react';
import { BrowserRouter, Navigate, Outlet, Route, Routes } from 'react-router-dom';
import { useAuth } from 'react-oidc-context';
import PageLoader from './components/PageLoader';

const FriendsPage = lazy(() => import('./pages/FriendsPage'));
const SessionsPage = lazy(() => import('./pages/SessionsPage'));
const GroupDetailPage = lazy(() => import('./pages/GroupDetailPage'));
const GroupEditPage = lazy(() => import('./pages/GroupEditPage'));
const GroupsPage = lazy(() => import('./pages/GroupsPage'));
const GameDetailPage = lazy(() => import('./pages/GameDetailPage'));
const JoinPage = lazy(() => import('./pages/JoinPage'));
const LibraryPage = lazy(() => import('./pages/LibraryPage'));
const MePage = lazy(() => import('./pages/MePage'));
const ProfilePage = lazy(() => import('./pages/ProfilePage'));
const SessionDetailPage = lazy(() => import('./pages/SessionDetailPage'));
const UserPage = lazy(() => import('./pages/UserPage'));
const AboutPage = lazy(() => import('./pages/AboutPage'));

/**
 * Layout route guard — tries a silent token refresh when the access token is
 * expired before falling back to a full login redirect.
 *
 * `useAutoSignin` would redirect immediately whenever `isAuthenticated` is
 * false — which includes the case where the access token has simply expired
 * while the app was backgrounded (common on mobile and inactive tabs).
 * Instead we attempt `signinSilent()` first; only if that also fails (no
 * refresh token, or the offline session itself has expired) do we send the
 * user to the login page.
 */
function RequireAuth() {
  const auth = useAuth();

  useEffect(() => {
    if (auth.isLoading || auth.isAuthenticated) return;

    if (auth.user) {
      // Token expired but session may still be recoverable via refresh token.
      void auth.signinSilent().catch(() => {
        void auth.signinRedirect({ redirect_uri: window.location.href });
      });
    } else {
      void auth.signinRedirect({ redirect_uri: window.location.href });
    }
  }, [auth]);

  return auth.isAuthenticated ? <Outlet /> : <PageLoader />;
}

export default function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={<PageLoader />}>
        <Routes>
          <Route path="/join/:token" element={<JoinPage />} />
          <Route element={<RequireAuth />}>
            <Route path="/" element={<Navigate to="/me" replace />} />
            <Route path="/me" element={<MePage />} />
            <Route path="/profile" element={<ProfilePage />} />
            <Route path="/groups" element={<GroupsPage />} />
            <Route path="/groups/:id" element={<GroupDetailPage />} />
            <Route path="/groups/:id/edit" element={<GroupEditPage />} />
            <Route path="/friends" element={<FriendsPage />} />
            <Route path="/users/:id" element={<UserPage />} />
            <Route path="/sessions" element={<SessionsPage />} />
            <Route path="/sessions/:id" element={<SessionDetailPage />} />
            <Route path="/library" element={<LibraryPage />} />
            <Route path="/library/:name" element={<GameDetailPage />} />
            <Route path="/about" element={<AboutPage />} />
          </Route>
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
