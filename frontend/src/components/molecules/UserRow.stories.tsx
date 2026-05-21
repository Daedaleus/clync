import type { Meta, StoryObj } from '@storybook/react-vite';
import UserRow from './UserRow';
import Button from '../atoms/Button';

const meta: Meta<typeof UserRow> = {
  component: UserRow,
  title: 'Molecules/UserRow',
};
export default meta;
type Story = StoryObj<typeof UserRow>;

export const Default: Story = {
  args: { keycloak_id: 'user-1', username: 'Anna Schmidt' },
};

export const WithAction: Story = {
  args: {
    keycloak_id: 'user-2',
    username: 'Ben Müller',
    right: <Button size="sm">Einladen</Button>,
  },
};

export const WithSentLabel: Story = {
  args: {
    keycloak_id: 'user-3',
    username: 'Clara Weber',
    right: <span className="text-xs text-zinc-500">Eingeladen ✓</span>,
  },
};

export const List: Story = {
  render: () => (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl divide-y divide-zinc-800">
      {[
        { id: 'u1', name: 'Anna Schmidt' },
        { id: 'u2', name: 'Ben Müller' },
        { id: 'u3', name: 'Clara Weber' },
      ].map((u) => (
        <UserRow
          key={u.id}
          keycloak_id={u.id}
          username={u.name}
          right={<Button size="sm">Einladen</Button>}
        />
      ))}
    </div>
  ),
};
