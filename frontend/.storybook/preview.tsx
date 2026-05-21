import type { Preview } from '@storybook/react-vite';
import { MemoryRouter } from 'react-router-dom';
import '../src/index.css';

const preview: Preview = {
  decorators: [
    (Story) => (
      <MemoryRouter>
        <div className="bg-zinc-950 p-6 min-h-screen">
          <Story />
        </div>
      </MemoryRouter>
    ),
  ],
  parameters: {
    backgrounds: { disable: true },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    a11y: { test: 'todo' },
  },
};

export default preview;
