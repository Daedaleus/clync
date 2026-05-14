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

  it('shows join button when not mine, not participant, not past, and onJoin provided', () => {
    const onJoin = vi.fn();
    wrap(<ul><SessionRow session={session()} onJoin={onJoin} /></ul>);
    expect(screen.getByRole('button', { name: 'Beitreten' })).toBeInTheDocument();
  });

  it('calls onJoin with session id when join button clicked', async () => {
    const onJoin = vi.fn();
    wrap(<ul><SessionRow session={session({ id: 'sess-42' })} onJoin={onJoin} /></ul>);
    await userEvent.click(screen.getByRole('button', { name: 'Beitreten' }));
    expect(onJoin).toHaveBeenCalledWith('sess-42');
  });

  it('does not show join button for past sessions', () => {
    wrap(<ul><SessionRow session={session({ scheduled_at: PAST })} onJoin={vi.fn()} /></ul>);
    expect(screen.queryByRole('button', { name: 'Beitreten' })).not.toBeInTheDocument();
  });

  it('does not show join button when already a participant', () => {
    wrap(<ul><SessionRow session={session({ is_participant: true })} onJoin={vi.fn()} /></ul>);
    expect(screen.queryByRole('button', { name: 'Beitreten' })).not.toBeInTheDocument();
  });

  it('shows "Zugesagt" when already a participant', () => {
    wrap(<ul><SessionRow session={session({ is_participant: true })} /></ul>);
    expect(screen.getByText('Zugesagt')).toBeInTheDocument();
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
