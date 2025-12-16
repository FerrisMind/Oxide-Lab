import { render } from '@testing-library/svelte/svelte5';

import { vi } from 'vitest';

const MockIcon = () => {};

vi.mock('phosphor-svelte/lib/Stop', () => ({ default: MockIcon }));
vi.mock('phosphor-svelte/lib/CheckCircle', () => ({ default: MockIcon }));
vi.mock('phosphor-svelte/lib/Lightbulb', () => ({ default: MockIcon }));

const { default: LoadingStatus } = await import('./LoadingStatus.svelte');

describe('LoadingStatus', () => {
  it('renders a spinner while model is loading', () => {
    const { getByTestId } = render(LoadingStatus, {
      props: {
        isLoadingModel: true,
        isCancelling: false,
        loadingStage: 'start',
        loadingProgress: 10,
        errorText: '',
      },
    });

    expect(getByTestId('model-loading-spinner')).toBeInTheDocument();
  });
});
