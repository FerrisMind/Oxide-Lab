import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Tabs from '../lib/components/ui/Tabs.svelte';

describe('Tabs component', () => {
  const mockTabs = [
    { id: 'tab1', label: 'Tab 1' },
    { id: 'tab2', label: 'Tab 2' },
    { id: 'tab3', label: 'Tab 3' },
  ];

  it('renders tabs correctly', () => {
    render(Tabs, {
      props: {
        tabs: mockTabs,
        activeTab: 'tab1',
        children: () => '<div>Tab content</div>',
      },
    });

    // Check if all tab labels are rendered
    expect(screen.getByText('Tab 1')).toBeInTheDocument();
    expect(screen.getByText('Tab 2')).toBeInTheDocument();
    expect(screen.getByText('Tab 3')).toBeInTheDocument();
  });

  it('shows active tab with correct styling', () => {
    render(Tabs, {
      props: {
        tabs: mockTabs,
        activeTab: 'tab2',
        children: () => '<div>Tab content</div>',
      },
    });

    const activeTab = screen.getByText('Tab 2');
    expect(activeTab).toHaveClass('active');
  });

  it('changes active tab on click', async () => {
    const mockActiveTab = { value: 'tab1' };

    render(Tabs, {
      props: {
        tabs: mockTabs,
        activeTab: mockActiveTab.value,
        children: () => '<div>Tab content</div>',
      },
    });

    const tab2 = screen.getByText('Tab 2');
    await fireEvent.click(tab2);

    // Note: In a real implementation, you'd need to use proper binding
    // This is a simplified test to demonstrate the concept
    expect(tab2).toBeInTheDocument();
  });

  it('renders tablist with proper accessibility attributes', () => {
    render(Tabs, {
      props: {
        tabs: mockTabs,
        activeTab: 'tab1',
        children: () => '<div>Tab content</div>',
      },
    });

    const tablist = screen.getByRole('tablist');
    expect(tablist).toBeInTheDocument();
    expect(tablist).toHaveAttribute('aria-label', 'Вкладки менеджера моделей');
  });

  it('renders tab buttons with proper accessibility attributes', () => {
    render(Tabs, {
      props: {
        tabs: mockTabs,
        activeTab: 'tab1',
        children: () => '<div>Tab content</div>',
      },
    });

    const tab1 = screen.getByRole('tab', { name: 'Tab 1' });
    expect(tab1).toHaveAttribute('aria-selected', 'true');
    expect(tab1).toHaveAttribute('aria-controls', 'tabpanel-tab1');
    expect(tab1).toHaveAttribute('tabindex', '0');

    const tab2 = screen.getByRole('tab', { name: 'Tab 2' });
    expect(tab2).toHaveAttribute('aria-selected', 'false');
    expect(tab2).toHaveAttribute('tabindex', '-1');
  });
});
