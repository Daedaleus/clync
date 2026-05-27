import { Suspense, lazy } from 'react';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import PageLoader from './components/PageLoader';

const FriendsPage = lazy(() => import('./pages/FriendsPage'));
const SessionsPage = lazy(() => import('./pages/SessionsPage'));
const GroupDetailPage = lazy(() => import('./pages/GroupDetailPage'));
const GroupEditPage = lazy(() => import('./pages/GroupEditPage'));
const GroupsPage = lazy(() => import('./pages/GroupsPage'));
const GameDetailPage = lazy(() => import('./pages/GameDetailPage'));
const LibraryPage = lazy(() => import('./pages/LibraryPage'));
const MePage = lazy(() => import('./pages/MePage'));
const ProfilePage = lazy(() => import('./pages/ProfilePage'));
const SessionDetailPage = lazy(() => import('./pages/SessionDetailPage'));
const UserPage = lazy(() => import('./pages/UserPage'));

export default function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={<PageLoader />}>
        <Routes>
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
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
