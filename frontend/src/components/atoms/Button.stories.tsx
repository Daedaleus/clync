import type { Meta, StoryObj } from '@storybook/react-vite';
import Button from './Button';

const meta: Meta<typeof Button> = {
  component: Button,
  title: 'Atoms/Button',
  argTypes: {
    variant: { control: 'select', options: ['primary', 'secondary', 'ghost', 'danger'] },
    size: { control: 'radio', options: ['sm', 'md'] },
    disabled: { control: 'boolean' },
  },
};
export default meta;
type Story = StoryObj<typeof Button>;

export const Primary: Story = { args: { children: 'Speichern', variant: 'primary' } };
export const Secondary: Story = { args: { children: 'Abbrechen', variant: 'secondary' } };
export const Ghost: Story = { args: { children: 'Mehr anzeigen', variant: 'ghost' } };
export const Danger: Story = { args: { children: 'Löschen', variant: 'danger' } };
export const Small: Story = { args: { children: 'Einladen', variant: 'primary', size: 'sm' } };
export const Disabled: Story = { args: { children: 'Eintragen', variant: 'primary', disabled: true } };
