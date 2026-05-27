import type { Meta, StoryObj } from '@storybook/react-vite';
import SessionRow from './SessionRow';
import type { Session } from '../../types';

const meta: Meta<typeof SessionRow> = {
  component: SessionRow,
  title: 'Molecules/SessionRow',
  decorators: [
    (Story) => (
      <ul className="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden">
        <Story />
      </ul>
    ),
  ],
};
export default meta;
type Story = StoryObj<typeof SessionRow>;

const base: Session = {
  id: 's1',
  user_id: 'u1',
  username: 'Anna Schmidt',
  game: 'Counter-Strike 2',
  scheduled_at: new Date(Date.now() + 2 * 60 * 60 * 1000).toISOString(),
  scope: 'global',
  participant_count: 3,
  is_mine: false,
  is_participant: false,
  my_rsvp: null,
  group_ids: [],
  group_names: [],
};

export const Default: Story = {
  args: { session: base, onRsvp: () => {} },
};

export const IsMine: Story = {
  args: {
    session: { ...base, is_mine: true },
    onDelete: () => {},
  },
};

export const IsAccepted: Story = {
  args: { session: { ...base, is_participant: true, my_rsvp: 'accepted' }, onRsvp: () => {} },
};

export const IsMaybe: Story = {
  args: { session: { ...base, my_rsvp: 'maybe' }, onRsvp: () => {} },
};

export const IsDeclined: Story = {
  args: { session: { ...base, my_rsvp: 'declined' }, onRsvp: () => {} },
};

export const GroupScope: Story = {
  args: {
    session: {
      ...base,
      scope: 'groups',
      group_ids: ['g1'],
      group_names: ['Stammtisch'],
    },
    onRsvp: () => {},
  },
};

export const LateJoinable: Story = {
  args: {
    session: {
      ...base,
      scheduled_at: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
    },
    onRsvp: () => {},
  },
};

export const Past: Story = {
  args: {
    session: {
      ...base,
      scheduled_at: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
    },
  },
};

export const List: Story = {
  decorators: [
    (Story) => (
      <ul className="bg-zinc-900 border border-zinc-800 rounded-xl divide-y divide-zinc-800 overflow-hidden">
        <Story />
      </ul>
    ),
  ],
  render: () => (
    <>
      <SessionRow
        session={base}
        onRsvp={() => {}}
      />
      <SessionRow
        session={{ ...base, id: 's2', game: 'Minecraft', scope: 'groups', group_names: ['Stammtisch'], is_mine: true }}
        onDelete={() => {}}
      />
      <SessionRow
        session={{ ...base, id: 's3', game: 'Elden Ring', is_participant: true, my_rsvp: 'accepted' }}
        onRsvp={() => {}}
      />
    </>
  ),
};
