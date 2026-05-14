import { Link } from 'react-router-dom';
import Avatar from '../atoms/Avatar';

interface Props {
  keycloak_id: string;
  username: string;
  right?: React.ReactNode;
}

export default function UserRow({ keycloak_id, username, right }: Props) {
  return (
    <div className="flex items-center justify-between px-4 py-3 gap-4">
      <div className="flex items-center gap-3 min-w-0">
        <Avatar name={username} />
        <Link
          to={`/users/${keycloak_id}`}
          className="text-sm font-medium text-zinc-100 hover:text-violet-400 transition-colors truncate"
        >
          {username}
        </Link>
      </div>
      {right && <div className="shrink-0">{right}</div>}
    </div>
  );
}
