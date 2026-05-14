import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Badge from './Badge';
import type { BadgeVariant } from './Badge';

describe('Badge', () => {
  const cases: Array<[BadgeVariant, string]> = [
    ['public',  'Öffentlich'],
    ['private', 'Privat'],
    ['friend',  'Freund'],
    ['global',  'Global'],
    ['groups',  'Gruppen'],
    ['joined',  'Beigetreten'],
  ];

  it.each(cases)('variant "%s" renders default label "%s"', (variant, label) => {
    render(<Badge variant={variant} />);
    expect(screen.getByText(label)).toBeInTheDocument();
  });

  it('renders a custom label when provided', () => {
    render(<Badge variant="public" label="My Custom Label" />);
    expect(screen.getByText('My Custom Label')).toBeInTheDocument();
  });

  it('custom label replaces the default', () => {
    render(<Badge variant="public" label="Custom" />);
    expect(screen.queryByText('Öffentlich')).not.toBeInTheDocument();
  });
});
