import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import FriendsPage from './pages/FriendsPage';
import SessionsPage from './pages/SessionsPage';
import GroupDetailPage from './pages/GroupDetailPage';
import GroupEditPage from './pages/GroupEditPage';
import GroupsPage from './pages/GroupsPage';
import GameDetailPage from './pages/GameDetailPage';
import LibraryPage from './pages/LibraryPage';
import MePage from './pages/MePage';
import ProfilePage from './pages/ProfilePage';
import SessionDetailPage from './pages/SessionDetailPage';
import UserPage from './pages/UserPage';

export default function App() {
  return (
    <BrowserRouter>
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
    </BrowserRouter>
  );
}
