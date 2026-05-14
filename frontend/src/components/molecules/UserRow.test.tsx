import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import UserRow from './UserRow';

const renderWithRouter = (ui: React.ReactElement) =>
  render(ui, { wrapper: MemoryRouter });

describe('UserRow', () => {
  it('renders the username', () => {
    renderWithRouter(<UserRow keycloak_id="abc123" username="alice" />);
    expect(screen.getByText('alice')).toBeInTheDocument();
  });

  it('links to the user profile page', () => {
    renderWithRouter(<UserRow keycloak_id="abc123" username="alice" />);
    expect(screen.getByRole('link')).toHaveAttribute('href', '/users/abc123');
  });

  it('renders the right slot when provided', () => {
    renderWithRouter(
      <UserRow keycloak_id="abc123" username="alice" right={<span>Entfernen</span>} />,
    );
    expect(screen.getByText('Entfernen')).toBeInTheDocument();
  });

  it('does not render a right slot when not provided', () => {
    const { container } = renderWithRouter(
      <UserRow keycloak_id="abc123" username="alice" />,
    );
    // Only the link should be the interactive element
    expect(container.querySelectorAll('a')).toHaveLength(1);
  });

  it('renders the avatar initial', () => {
    renderWithRouter(<UserRow keycloak_id="abc123" username="alice" />);
    expect(screen.getByText('A')).toBeInTheDocument();
  });
});
