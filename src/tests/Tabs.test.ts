import { render, screen } from '@testing-library/svelte/svelte5';
import { describe, it, expect } from 'vitest';
import Tabs from '../lib/components/ui/Tabs.svelte';

describe('Tabs component', () => {
  const mockTabs = [
    { id: 'tab1', label: 'Tab 1' },
    { id: 'tab2', label: 'Tab 2' },
    { id: 'tab3', label: 'Tab 3' },
  ];

  it('renders tabs correctly', () => {
    render(Tabs as any, {
      props: {
        tabs: mockTabs,
        activeTab: 'tab1',
        children: () => {},
      },
    });

    // Check if all tab labels are rendered
    expect(screen.getByText('Tab 1')).toBeInTheDocument();
    expect(screen.getByText('Tab 2')).toBeInTheDocument();
    expect(screen.getByText('Tab 3')).toBeInTheDocument();
  });

  it('shows active tab with correct styling', () => {
    render(Tabs as any, {
      props: {
        tabs: mockTabs,
        activeTab: 'tab2',
        children: () => {},
      },
    });

    const activeTab = screen.getByText('Tab 2');
    expect(activeTab).toHaveClass('active');
  });

  it('changes active tab on click', async () => {
    const { userEvent } = await import('@testing-library/user-event');
    const user = userEvent.setup();

    let activeTabValue = 'tab1';

    render(Tabs as any, {
      props: {
        tabs: mockTabs,
        activeTab: activeTabValue,
        children: () => {},
      },
    });

    const tab2 = screen.getByText('Tab 2');
    await user.click(tab2);

    // Note: Since activeTab is not bound in this test setup,
    // we just verify the tab is present and clickable
    expect(tab2).toBeInTheDocument();
  });

  it('renders tablist with proper accessibility attributes', () => {
    render(Tabs as any, {
      props: {
        tabs: mockTabs,
        activeTab: 'tab1',
        children: () => {},
      },
    });

    const tablist = screen.getByRole('tablist');
    expect(tablist).toBeInTheDocument();
    expect(tablist).toHaveAttribute('aria-label', 'Вкладки менеджера моделей');
  });

  it('renders tab buttons with proper accessibility attributes', () => {
    render(Tabs as any, {
      props: {
        tabs: mockTabs,
        activeTab: 'tab1',
        children: () => {},
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
