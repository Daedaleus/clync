import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import VisibilitySelect from './VisibilitySelect';

describe('VisibilitySelect', () => {
  it('renders all three visibility options', () => {
    render(<VisibilitySelect value="public" onChange={() => {}} />);
    expect(screen.getByRole('combobox')).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Öffentlich' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'In Gruppe' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Nur Freunde' })).toBeInTheDocument();
  });

  it('shows the current value as selected', () => {
    render(<VisibilitySelect value="friends" onChange={() => {}} />);
    expect(screen.getByRole('combobox')).toHaveValue('friends');
  });

  it('calls onChange with the selected value', async () => {
    const onChange = vi.fn();
    render(<VisibilitySelect value="public" onChange={onChange} />);
    await userEvent.selectOptions(screen.getByRole('combobox'), 'group');
    expect(onChange).toHaveBeenCalledWith('group');
  });

  it('applies the id prop', () => {
    render(<VisibilitySelect value="public" onChange={() => {}} id="steam-vis" />);
    expect(screen.getByRole('combobox')).toHaveAttribute('id', 'steam-vis');
  });
});
