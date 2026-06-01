import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SearchBar from './SearchBar';

describe('SearchBar', () => {
  it('renders input and submit button', () => {
    render(<SearchBar value="" onChange={() => {}} onSubmit={() => {}} />);
    expect(screen.getByRole('textbox')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Suchen' })).toBeInTheDocument();
  });

  it('shows custom placeholder', () => {
    render(<SearchBar value="" onChange={() => {}} onSubmit={() => {}} placeholder="Spiel suchen…" />);
    expect(screen.getByPlaceholderText('Spiel suchen…')).toBeInTheDocument();
  });

  it('shows custom button label', () => {
    render(<SearchBar value="" onChange={() => {}} onSubmit={() => {}} buttonLabel="Los" />);
    expect(screen.getByRole('button', { name: 'Los' })).toBeInTheDocument();
  });

  it('reflects the current value', () => {
    render(<SearchBar value="CS2" onChange={() => {}} onSubmit={() => {}} />);
    expect(screen.getByRole('textbox')).toHaveValue('CS2');
  });

  it('calls onChange when user types', async () => {
    const onChange = vi.fn();
    render(<SearchBar value="" onChange={onChange} onSubmit={() => {}} />);
    await userEvent.type(screen.getByRole('textbox'), 'a');
    expect(onChange).toHaveBeenCalledWith('a');
  });

  it('calls onSubmit when form is submitted', async () => {
    const onSubmit = vi.fn((e: React.FormEvent) => e.preventDefault());
    render(<SearchBar value="CS2" onChange={() => {}} onSubmit={onSubmit} />);
    await userEvent.click(screen.getByRole('button'));
    expect(onSubmit).toHaveBeenCalledOnce();
  });
});
