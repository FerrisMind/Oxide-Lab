import { render } from '@testing-library/svelte/svelte5';
import Spinner from './spinner.svelte';

describe('Spinner', () => {
  it('renders with provided size', () => {
    const { getByTestId } = render(Spinner, { props: { size: 20, testId: 'spinner' } });
    expect(getByTestId('spinner')).toHaveStyle('--spinner-size: 20px');
  });
});
