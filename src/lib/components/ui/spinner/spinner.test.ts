import { render } from '@testing-library/svelte';
import { asClassComponent } from 'svelte/legacy';
import { describe, expect, it } from 'vitest';
import Spinner from './spinner.svelte';

describe('Spinner', () => {
  it('renders with provided size', () => {
    const { getByTestId } = render(asClassComponent(Spinner), {
      props: { size: 20, testId: 'spinner' },
    });
    expect(getByTestId('spinner')).toHaveStyle('--spinner-size: 20px');
  });
});
