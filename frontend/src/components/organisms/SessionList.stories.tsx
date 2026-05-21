import type { Meta, StoryObj } from '@storybook/react-vite';
import SessionList from './SessionList';
import type { Session } from '../../types';

const meta: Meta<typeof SessionList> = {
  component: SessionList,
  title: 'Organisms/SessionList',
};
export default meta;
type Story = StoryObj<typeof SessionList>;

const now = Date.now();

const sessions: Session[] = [
  {
    id: 's1',
    user_id: 'u1',
    username: 'Anna Schmidt',
    game: 'Counter-Strike 2',
    scheduled_at: new Date(now + 1 * 60 * 60 * 1000).toISOString(),
    scope: 'global',
    participant_count: 4,
    is_mine: true,
    is_participant: false,
    group_ids: [],
    group_names: [],
  },
  {
    id: 's2',
    user_id: 'u2',
    username: 'Ben Müller',
    game: 'Minecraft',
    scheduled_at: new Date(now + 3 * 60 * 60 * 1000).toISOString(),
    scope: 'groups',
    participant_count: 2,
    is_mine: false,
    is_participant: true,
    group_ids: ['g1'],
    group_names: ['Stammtisch'],
  },
  {
    id: 's3',
    user_id: 'u3',
    username: 'Clara Weber',
    game: 'Elden Ring',
    scheduled_at: new Date(now + 6 * 60 * 60 * 1000).toISOString(),
    scope: 'global',
    participant_count: 1,
    is_mine: false,
    is_participant: false,
    group_ids: [],
    group_names: [],
  },
];

export const WithSessions: Story = {
  args: { sessions, onJoin: () => {}, onDelete: () => {} },
};

export const Empty: Story = {
  args: { sessions: [] },
};

export const CustomEmptyMessage: Story = {
  args: { sessions: [], emptyMessage: 'Noch keine Sessions für diese Gruppe.' },
};
