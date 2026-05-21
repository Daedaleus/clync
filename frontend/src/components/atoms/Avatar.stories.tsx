import type { Meta, StoryObj } from '@storybook/react-vite';
import Avatar from './Avatar';

const meta: Meta<typeof Avatar> = {
  component: Avatar,
  title: 'Atoms/Avatar',
  argTypes: {
    size: { control: 'radio', options: ['sm', 'md', 'lg'] },
    name: { control: 'text' },
  },
};
export default meta;
type Story = StoryObj<typeof Avatar>;

export const Small: Story = { args: { name: 'Anna', size: 'sm' } };
export const Medium: Story = { args: { name: 'Ben', size: 'md' } };
export const Large: Story = { args: { name: 'Clara', size: 'lg' } };

export const AllSizes: Story = {
  render: () => (
    <div className="flex items-end gap-4">
      <Avatar name="Anna" size="sm" />
      <Avatar name="Ben" size="md" />
      <Avatar name="Clara" size="lg" />
    </div>
  ),
};
