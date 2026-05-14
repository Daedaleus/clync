import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Avatar from './Avatar';

describe('Avatar', () => {
  it('renders the uppercase initial of the name', () => {
    render(<Avatar name="alice" />);
    expect(screen.getByText('A')).toBeInTheDocument();
  });

  it('uppercases a lowercase initial', () => {
    render(<Avatar name="bob" />);
    expect(screen.getByText('B')).toBeInTheDocument();
  });

  it('uses only the first character', () => {
    render(<Avatar name="Charlie" />);
    expect(screen.getByText('C')).toBeInTheDocument();
  });
});
