import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import userEvent from '@testing-library/user-event';
import SessionList from './SessionList';
import type { Session } from '../../types';

const FUTURE = '2099-12-31T23:59:59.000Z';

function session(id: string, game: string, overrides: Partial<Session> = {}): Session {
  return {
    id,
    user_id: 'user-alice',
    username: 'alice',
    game,
    scheduled_at: FUTURE,
    scope: 'global',
    participant_count: 1,
    is_mine: false,
    is_participant: false,
    my_rsvp: null,
    group_ids: [], group_names: [],
    ...overrides,
  };
}

const wrap = (ui: React.ReactElement) =>
  render(<MemoryRouter>{ui}</MemoryRouter>);

describe('SessionList', () => {
  it('shows the default empty message when sessions is empty', () => {
    wrap(<SessionList sessions={[]} />);
    expect(screen.getByText('Keine Spielzeiten.')).toBeInTheDocument();
  });

  it('shows a custom empty message', () => {
    wrap(<SessionList sessions={[]} emptyMessage="Noch nichts geplant." />);
    expect(screen.getByText('Noch nichts geplant.')).toBeInTheDocument();
  });

  it('renders all session game names', () => {
    wrap(<SessionList sessions={[session('1', 'CS2'), session('2', 'Minecraft')]} />);
    expect(screen.getByText('CS2')).toBeInTheDocument();
    expect(screen.getByText('Minecraft')).toBeInTheDocument();
  });

  it('does not show empty message when sessions exist', () => {
    wrap(<SessionList sessions={[session('1', 'CS2')]} />);
    expect(screen.queryByText('Keine Spielzeiten.')).not.toBeInTheDocument();
  });

  it('calls onRsvp with the correct id and status', async () => {
    const onRsvp = vi.fn();
    wrap(<SessionList sessions={[session('sess-1', 'CS2')]} onRsvp={onRsvp} />);
    await userEvent.click(screen.getByRole('button', { name: 'Zusagen' }));
    expect(onRsvp).toHaveBeenCalledWith('sess-1', 'accepted');
  });

  it('calls onDelete with the correct id', async () => {
    const onDelete = vi.fn();
    wrap(
      <SessionList
        sessions={[session('sess-2', 'CS2', { is_mine: true })]}
        onDelete={onDelete}
      />,
    );
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }));
    expect(onDelete).toHaveBeenCalledWith('sess-2');
  });
});
