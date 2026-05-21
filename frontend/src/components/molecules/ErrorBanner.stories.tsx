import type { Meta, StoryObj } from '@storybook/react-vite';
import ErrorBanner from './ErrorBanner';

const meta: Meta<typeof ErrorBanner> = {
  component: ErrorBanner,
  title: 'Molecules/ErrorBanner',
};
export default meta;
type Story = StoryObj<typeof ErrorBanner>;

export const Short: Story = { args: { message: 'Fehler beim Laden.' } };
export const Long: Story = {
  args: {
    message:
      'Die Session konnte nicht erstellt werden. Bitte stelle sicher, dass das Datum in der Zukunft liegt und ein Spiel ausgewählt wurde.',
  },
};
