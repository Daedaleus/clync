import type { Meta, StoryObj } from '@storybook/react-vite';
import GameChip from './GameChip';

const meta: Meta<typeof GameChip> = {
  component: GameChip,
  title: 'Molecules/GameChip',
  argTypes: { name: { control: 'text' } },
};
export default meta;
type Story = StoryObj<typeof GameChip>;

export const Default: Story = { args: { name: 'Counter-Strike 2' } };
export const WithRemove: Story = { args: { name: 'Minecraft', onRemove: () => {} } };

export const List: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      {['Counter-Strike 2', 'Minecraft', 'Elden Ring', 'Stardew Valley'].map((n) => (
        <GameChip key={n} name={n} onRemove={() => {}} />
      ))}
    </div>
  ),
};
