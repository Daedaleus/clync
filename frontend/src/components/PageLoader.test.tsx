import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import PageLoader from './PageLoader';

describe('PageLoader', () => {
  it('renders without crashing', () => {
    const { container } = render(<PageLoader />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it('contains a spinning element', () => {
    const { container } = render(<PageLoader />);
    expect(container.querySelector('.animate-spin')).toBeInTheDocument();
  });
});
