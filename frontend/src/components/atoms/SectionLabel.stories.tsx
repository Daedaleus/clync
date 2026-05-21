import type { Meta, StoryObj } from '@storybook/react-vite';
import SectionLabel from './SectionLabel';

const meta: Meta<typeof SectionLabel> = {
  component: SectionLabel,
  title: 'Atoms/SectionLabel',
};
export default meta;
type Story = StoryObj<typeof SectionLabel>;

export const Default: Story = { args: { children: 'Nächste Sessions' } };
export const WithCount: Story = { args: { children: 'Gruppen', count: 4 } };
export const ZeroCount: Story = { args: { children: 'Freunde', count: 0 } };
