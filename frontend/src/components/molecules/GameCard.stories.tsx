import type { Meta, StoryObj } from '@storybook/react-vite';
import GameCard from './GameCard';

const meta: Meta<typeof GameCard> = {
  component: GameCard,
  title: 'Molecules/GameCard',
};
export default meta;
type Story = StoryObj<typeof GameCard>;

const gameBase = { name: 'Counter-Strike 2', genre: 'Shooter' };

export const NoThumbnail: Story = {
  args: { game: gameBase },
};

export const WithWishlistButton: Story = {
  args: { game: gameBase, inWishlist: false, onToggle: () => {} },
};

export const InWishlist: Story = {
  args: { game: gameBase, inWishlist: true, onToggle: () => {} },
};

export const NoGenre: Story = {
  args: { game: { name: 'Minecraft' }, inWishlist: false, onToggle: () => {} },
};

export const Grid: Story = {
  render: () => (
    <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 max-w-xl">
      {[
        { name: 'Counter-Strike 2', genre: 'Shooter' },
        { name: 'Minecraft', genre: 'Sandbox' },
        { name: 'Elden Ring', genre: 'Action-RPG' },
        { name: 'Stardew Valley', genre: 'Simulation' },
      ].map((g) => (
        <GameCard key={g.name} game={g} inWishlist={false} onToggle={() => {}} />
      ))}
    </div>
  ),
};
