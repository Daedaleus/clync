import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import userEvent from '@testing-library/user-event';
import SessionRow from './SessionRow';
import type { Session } from '../../types';

const FUTURE = '2099-12-31T23:59:59.000Z';
const PAST   = '2020-01-15T10:30:00.000Z';

function session(overrides: Partial<Session> = {}): Session {
  return {
    id: 's1',
    user_id: 'user-alice',
    username: 'alice',
    game: 'Counter-Strike 2',
    scheduled_at: FUTURE,
    scope: 'global',
    participant_count: 3,
    is_mine: false,
    is_participant: false,
    my_rsvp: null,
    group_ids: [], group_names: [],
    ...overrides,
  };
}

const wrap = (ui: React.ReactElement) =>
  render(<MemoryRouter>{ui}</MemoryRouter>);

describe('SessionRow', () => {
  it('renders the game name', () => {
    wrap(<ul><SessionRow session={session()} /></ul>);
    expect(screen.getByText('Counter-Strike 2')).toBeInTheDocument();
  });

  it('renders the username', () => {
    wrap(<ul><SessionRow session={session()} /></ul>);
    expect(screen.getByText('alice')).toBeInTheDocument();
  });

  it('shows RSVP buttons when not mine, not past, and onRsvp provided', () => {
    const onRsvp = vi.fn();
    wrap(<ul><SessionRow session={session()} onRsvp={onRsvp} /></ul>);
    expect(screen.getByRole('button', { name: 'Zusagen' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Vielleicht' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Absagen' })).toBeInTheDocument();
  });

  it('calls onRsvp with session id and accepted status when clicked', async () => {
    const onRsvp = vi.fn();
    wrap(<ul><SessionRow session={session({ id: 'sess-42' })} onRsvp={onRsvp} /></ul>);
    await userEvent.click(screen.getByRole('button', { name: 'Zusagen' }));
    expect(onRsvp).toHaveBeenCalledWith('sess-42', 'accepted');
  });

  it('calls onRsvp with null when toggling off the active status', async () => {
    const onRsvp = vi.fn();
    wrap(<ul><SessionRow session={session({ id: 'sess-1', my_rsvp: 'accepted' })} onRsvp={onRsvp} /></ul>);
    await userEvent.click(screen.getByRole('button', { name: 'Zusagen' }));
    expect(onRsvp).toHaveBeenCalledWith('sess-1', null);
  });

  it('does not show RSVP buttons for past sessions', () => {
    wrap(<ul><SessionRow session={session({ scheduled_at: PAST })} onRsvp={vi.fn()} /></ul>);
    expect(screen.queryByRole('button', { name: 'Zusagen' })).not.toBeInTheDocument();
  });

  it('does not show RSVP buttons for own sessions', () => {
    wrap(<ul><SessionRow session={session({ is_mine: true })} onRsvp={vi.fn()} /></ul>);
    expect(screen.queryByRole('button', { name: 'Zusagen' })).not.toBeInTheDocument();
  });

  it('shows delete button when is_mine and onDelete provided', () => {
    const onDelete = vi.fn();
    wrap(<ul><SessionRow session={session({ is_mine: true })} onDelete={onDelete} /></ul>);
    expect(screen.getByRole('button', { name: 'Löschen' })).toBeInTheDocument();
  });

  it('calls onDelete with session id when delete clicked', async () => {
    const onDelete = vi.fn();
    wrap(<ul><SessionRow session={session({ id: 'sess-99', is_mine: true })} onDelete={onDelete} /></ul>);
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }));
    expect(onDelete).toHaveBeenCalledWith('sess-99');
  });

  it('shows "vergangen" label for past sessions', () => {
    wrap(<ul><SessionRow session={session({ scheduled_at: PAST })} /></ul>);
    expect(screen.getByText('vergangen')).toBeInTheDocument();
  });
});
