import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ErrorBoundary } from './ErrorBoundary';

const ThrowingComponent = () => {
  throw new Error('boom');
};

describe('ErrorBoundary', () => {
  beforeEach(() => {
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  it('renders children when there is no error', () => {
    render(
      <ErrorBoundary>
        <p>all good</p>
      </ErrorBoundary>
    );
    expect(screen.getByText('all good')).toBeInTheDocument();
  });

  it('shows fallback UI when a child throws', () => {
    render(
      <ErrorBoundary>
        <ThrowingComponent />
      </ErrorBoundary>
    );
    expect(screen.getByText('Etwas ist schiefgelaufen')).toBeInTheDocument();
  });

  it('shows the error message in the details', () => {
    render(
      <ErrorBoundary>
        <ThrowingComponent />
      </ErrorBoundary>
    );
    expect(screen.getByText('boom')).toBeInTheDocument();
  });

  it('renders a reload button', () => {
    render(
      <ErrorBoundary>
        <ThrowingComponent />
      </ErrorBoundary>
    );
    expect(screen.getByRole('button', { name: 'Seite neu laden' })).toBeInTheDocument();
  });

  it('calls window.location.reload when reload button is clicked', async () => {
    const reload = vi.fn();
    Object.defineProperty(window, 'location', { value: { reload }, writable: true });
    render(
      <ErrorBoundary>
        <ThrowingComponent />
      </ErrorBoundary>
    );
    await userEvent.click(screen.getByRole('button', { name: 'Seite neu laden' }));
    expect(reload).toHaveBeenCalledOnce();
  });
});
