import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import SectionLabel from './SectionLabel';

describe('SectionLabel', () => {
  it('renders the label text', () => {
    render(<SectionLabel>Meine Spiele</SectionLabel>);
    expect(screen.getByText('Meine Spiele')).toBeInTheDocument();
  });

  it('shows the count in parentheses when provided', () => {
    render(<SectionLabel count={5}>Gruppen</SectionLabel>);
    expect(screen.getByText('(5)')).toBeInTheDocument();
  });

  it('shows zero count correctly', () => {
    render(<SectionLabel count={0}>Gruppen</SectionLabel>);
    expect(screen.getByText('(0)')).toBeInTheDocument();
  });

  it('does not render parentheses when count is not provided', () => {
    render(<SectionLabel>Gruppen</SectionLabel>);
    expect(screen.queryByText(/\(/)).not.toBeInTheDocument();
  });
});
