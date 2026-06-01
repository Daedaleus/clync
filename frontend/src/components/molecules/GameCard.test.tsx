import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import GameCard from './GameCard';
import type { Game } from '../../types';

const game: Game = { name: 'CS2', genre: 'Shooter', thumbnail_url: null };

const renderCard = (props: Partial<Parameters<typeof GameCard>[0]> = {}) =>
  render(
    <MemoryRouter>
      <GameCard game={game} {...props} />
    </MemoryRouter>
  );

describe('GameCard', () => {
  it('renders the game name', () => {
    renderCard();
    expect(screen.getAllByText('CS2').length).toBeGreaterThan(0);
  });

  it('renders the genre when provided', () => {
    renderCard();
    expect(screen.getByText('Shooter')).toBeInTheDocument();
  });

  it('does not render genre when omitted', () => {
    render(
      <MemoryRouter>
        <GameCard game={{ name: 'CS2', genre: null }} />
      </MemoryRouter>
    );
    expect(screen.queryByText('Shooter')).not.toBeInTheDocument();
  });

  it('links to the library detail page', () => {
    renderCard();
    const links = screen.getAllByRole('link');
    expect(links[0]).toHaveAttribute('href', '/library/CS2');
  });

  it('shows the fallback game icon when no thumbnail', () => {
    renderCard();
    expect(screen.getByText('🎮')).toBeInTheDocument();
  });

  it('renders add-to-list button when onToggle is provided', () => {
    renderCard({ onToggle: vi.fn() });
    expect(screen.getByRole('button', { name: '+ Zur Liste' })).toBeInTheDocument();
  });

  it('renders in-list button when inWishlist is true', () => {
    renderCard({ inWishlist: true, onToggle: vi.fn() });
    expect(screen.getByRole('button', { name: '✓ In meiner Liste' })).toBeInTheDocument();
  });

  it('calls onToggle when the wishlist button is clicked', async () => {
    const onToggle = vi.fn();
    renderCard({ onToggle });
    await userEvent.click(screen.getByRole('button'));
    expect(onToggle).toHaveBeenCalledOnce();
  });

  it('does not render a button when onToggle is not provided', () => {
    renderCard();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});
