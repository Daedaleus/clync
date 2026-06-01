import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Input from './Input';

describe('Input', () => {
  it('renders an input element', () => {
    render(<Input />);
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('forwards value and placeholder props', () => {
    render(<Input value="hello" placeholder="Enter text" onChange={() => {}} />);
    const input = screen.getByRole('textbox');
    expect(input).toHaveValue('hello');
    expect(input).toHaveAttribute('placeholder', 'Enter text');
  });

  it('calls onChange when user types', async () => {
    const onChange = vi.fn();
    render(<Input onChange={onChange} />);
    await userEvent.type(screen.getByRole('textbox'), 'a');
    expect(onChange).toHaveBeenCalled();
  });

  it('applies extra className alongside base classes', () => {
    render(<Input className="extra-class" />);
    expect(screen.getByRole('textbox').className).toContain('extra-class');
  });

  it('forwards type attribute', () => {
    render(<Input type="password" />);
    expect(document.querySelector('input[type="password"]')).toBeInTheDocument();
  });
});
