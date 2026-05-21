import { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react-vite';
import VisibilitySelect, { type Visibility } from './VisibilitySelect';

const meta: Meta<typeof VisibilitySelect> = {
  component: VisibilitySelect,
  title: 'Atoms/VisibilitySelect',
};
export default meta;
type Story = StoryObj<typeof VisibilitySelect>;

export const Public: Story = {
  render: () => {
    const [v, setV] = useState<Visibility>('public');
    return <VisibilitySelect value={v} onChange={setV} />;
  },
};

export const Friends: Story = {
  render: () => {
    const [v, setV] = useState<Visibility>('friends');
    return <VisibilitySelect value={v} onChange={setV} />;
  },
};
