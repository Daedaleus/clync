import type { Meta, StoryObj } from '@storybook/react-vite';
import Badge from './Badge';

const meta: Meta<typeof Badge> = {
  component: Badge,
  title: 'Atoms/Badge',
  argTypes: {
    variant: {
      control: 'select',
      options: ['public', 'private', 'friend', 'global', 'groups', 'joined'],
    },
  },
};
export default meta;
type Story = StoryObj<typeof Badge>;

export const Public: Story = { args: { variant: 'public' } };
export const Private: Story = { args: { variant: 'private' } };
export const Friend: Story = { args: { variant: 'friend' } };
export const Global: Story = { args: { variant: 'global' } };
export const Groups: Story = { args: { variant: 'groups' } };
export const Joined: Story = { args: { variant: 'joined' } };
export const CustomLabel: Story = { args: { variant: 'private', label: 'Vergangen' } };

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      {(['public', 'private', 'friend', 'global', 'groups', 'joined'] as const).map((v) => (
        <Badge key={v} variant={v} />
      ))}
    </div>
  ),
};
