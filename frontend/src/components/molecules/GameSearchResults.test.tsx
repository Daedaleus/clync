import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import GameSearchResults from './GameSearchResults';
import type { AutofillCandidate } from './AutofillPicker';

const candidate: AutofillCandidate = {
  rawg_id: 1,
  rawg_name: 'Counter-Strike 2',
  thumbnail_url: null,
  genre: 'Shooter',
};

const defaults = {
  loading: false,
  addedNames: new Set<string>(),
  busyName: null,
  onAdd: vi.fn(),
  onClose: vi.fn(),
};

describe('GameSearchResults', () => {
  it('shows loading message while searching', () => {
    render(<GameSearchResults {...defaults} candidates={[]} loading={true} />);
    expect(screen.getByText('Suche auf RAWG…')).toBeInTheDocument();
  });

  it('shows no-results message with close button when empty', () => {
    render(<GameSearchResults {...defaults} candidates={[]} />);
    expect(screen.getByText('Kein Spiel auf RAWG gefunden.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Schließen' })).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', async () => {
    const onClose = vi.fn();
    render(<GameSearchResults {...defaults} candidates={[]} onClose={onClose} />);
    await userEvent.click(screen.getByRole('button', { name: 'Schließen' }));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it('renders a candidate with name and genre', () => {
    render(<GameSearchResults {...defaults} candidates={[candidate]} />);
    expect(screen.getByText('Counter-Strike 2')).toBeInTheDocument();
    expect(screen.getByText('Shooter')).toBeInTheDocument();
  });

  it('calls onAdd when the add button is clicked', async () => {
    const onAdd = vi.fn();
    render(<GameSearchResults {...defaults} candidates={[candidate]} onAdd={onAdd} />);
    await userEvent.click(screen.getByRole('button', { name: 'Hinzufügen' }));
    expect(onAdd).toHaveBeenCalledWith(candidate);
  });

  it('shows added state when game is already in wishlist', () => {
    render(
      <GameSearchResults
        {...defaults}
        candidates={[candidate]}
        addedNames={new Set(['Counter-Strike 2'])}
      />
    );
    expect(screen.getByRole('button', { name: /hinzugefügt/i })).toBeDisabled();
  });

  it('shows fallback icon when candidate has no thumbnail', () => {
    render(<GameSearchResults {...defaults} candidates={[candidate]} />);
    expect(screen.getByText('🎮')).toBeInTheDocument();
  });
});
