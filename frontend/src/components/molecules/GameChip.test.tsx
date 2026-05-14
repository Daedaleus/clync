import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import GameChip from './GameChip';

describe('GameChip', () => {
  it('renders the game name', () => {
    render(<GameChip name="Counter-Strike 2" />);
    expect(screen.getByText('Counter-Strike 2')).toBeInTheDocument();
  });

  it('does not show a remove button when onRemove is not provided', () => {
    render(<GameChip name="CS2" />);
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('shows a remove button when onRemove is provided', () => {
    render(<GameChip name="CS2" onRemove={vi.fn()} />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('calls onRemove when the remove button is clicked', async () => {
    const onRemove = vi.fn();
    render(<GameChip name="CS2" onRemove={onRemove} />);
    await userEvent.click(screen.getByRole('button'));
    expect(onRemove).toHaveBeenCalledOnce();
  });

  it('remove button has an accessible aria-label', () => {
    render(<GameChip name="Minecraft" onRemove={vi.fn()} />);
    expect(screen.getByRole('button', { name: 'Minecraft entfernen' })).toBeInTheDocument();
  });
});
