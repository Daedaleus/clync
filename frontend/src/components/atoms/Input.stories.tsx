import type { Meta, StoryObj } from '@storybook/react-vite';
import Input from './Input';

const meta: Meta<typeof Input> = {
  component: Input,
  title: 'Atoms/Input',
  argTypes: {
    disabled: { control: 'boolean' },
    placeholder: { control: 'text' },
  },
};
export default meta;
type Story = StoryObj<typeof Input>;

export const Default: Story = { args: { placeholder: 'Spielname eingeben…' } };
export const WithValue: Story = { args: { defaultValue: 'Counter-Strike 2' } };
export const Disabled: Story = { args: { placeholder: 'Nicht bearbeitbar', disabled: true } };
export const FullWidth: Story = { args: { placeholder: 'Suchen…', className: 'w-full' } };
