import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import GameSection from './GameSection';

// GameInput is mocked to avoid pulling in the real api service
vi.mock('../molecules/GameInput', () => ({
  default: ({ onAdd }: { onAdd: (name: string) => void }) => (
    <button onClick={() => onAdd('Mocked Game')}>Spiel hinzufügen</button>
  ),
}));

describe('GameSection', () => {
  it('renders all provided game chips', () => {
    render(<GameSection games={['CS2', 'Minecraft']} />);
    expect(screen.getByText('CS2')).toBeInTheDocument();
    expect(screen.getByText('Minecraft')).toBeInTheDocument();
  });

  it('shows empty state message when there are no games', () => {
    render(<GameSection games={[]} />);
    expect(screen.getByText('Noch keine Spiele.')).toBeInTheDocument();
  });

  it('renders GameInput when onAdd is provided', () => {
    render(<GameSection games={[]} onAdd={vi.fn()} />);
    expect(screen.getByRole('button', { name: 'Spiel hinzufügen' })).toBeInTheDocument();
  });

  it('does not render GameInput when onAdd is not provided (read-only mode)', () => {
    render(<GameSection games={[]} />);
    expect(screen.queryByRole('button', { name: 'Spiel hinzufügen' })).not.toBeInTheDocument();
  });

  it('calls onAdd with the game name when adding a game', async () => {
    const onAdd = vi.fn();
    render(<GameSection games={[]} onAdd={onAdd} />);
    await userEvent.click(screen.getByRole('button', { name: 'Spiel hinzufügen' }));
    expect(onAdd).toHaveBeenCalledWith('Mocked Game');
  });

  it('chips have remove buttons when onRemove is provided', () => {
    render(<GameSection games={['CS2']} onRemove={vi.fn()} />);
    expect(screen.getByRole('button', { name: 'CS2 entfernen' })).toBeInTheDocument();
  });

  it('chips do not have remove buttons when onRemove is not provided', () => {
    render(<GameSection games={['CS2']} />);
    expect(screen.queryByRole('button', { name: 'CS2 entfernen' })).not.toBeInTheDocument();
  });

  it('calls onRemove with the correct game name when chip is removed', async () => {
    const onRemove = vi.fn();
    render(<GameSection games={['CS2', 'Minecraft']} onRemove={onRemove} />);
    await userEvent.click(screen.getByRole('button', { name: 'CS2 entfernen' }));
    expect(onRemove).toHaveBeenCalledWith('CS2');
    expect(onRemove).not.toHaveBeenCalledWith('Minecraft');
  });
});
