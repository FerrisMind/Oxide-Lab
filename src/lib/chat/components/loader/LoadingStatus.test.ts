import { render } from '@testing-library/svelte';
import { asClassComponent } from 'svelte/legacy';

import { describe, expect, it, vi } from 'vitest';

const MockIcon = () => {};

vi.mock('phosphor-svelte/lib/Stop', () => ({ default: MockIcon }));
vi.mock('phosphor-svelte/lib/CheckCircle', () => ({ default: MockIcon }));
vi.mock('phosphor-svelte/lib/Lightbulb', () => ({ default: MockIcon }));

const { default: LoadingStatus } = await import('./LoadingStatus.svelte');

describe('LoadingStatus', () => {
  it('renders a spinner while model is loading', () => {
    const { getByTestId } = render(asClassComponent(LoadingStatus), {
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
