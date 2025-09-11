import { test, expect } from 'vitest';
import { createStreamParser } from './impl';

test('createStreamParser - empty <think> produces no segments', () => {
  const p = createStreamParser();
  const { segments } = p.parse('<think></think>');
  expect(segments).toHaveLength(0);
});

test('createStreamParser - whitespace-only <think> produces no segments', () => {
  const p = createStreamParser();
  const { segments } = p.parse('<think>   \n\t  \n</think>');
  expect(segments).toHaveLength(0);
});

test('createStreamParser - non-empty <think> renders details and content', () => {
  const p = createStreamParser();
  const { segments } = p.parse('<think>hello world</think>');
  // Expect opening HTML, content HTML, and closing HTML
  expect(segments.length).toBeGreaterThanOrEqual(1);
  const joined = segments.map((s) => s.data).join('');
  expect(joined).toContain('Рассуждения');
  expect(joined).toContain('hello world');
});

test('createStreamParser - streaming think opens only after non-empty content arrives', () => {
  const p = createStreamParser();
  let buf = '';
  buf += '<think>';
  let r = p.parse(buf);
  buf = r.remainder;
  expect(r.segments).toHaveLength(0);

  buf += '\n\n';
  r = p.parse(buf);
  buf = r.remainder;
  // still whitespace only -> no opening
  expect(r.segments).toHaveLength(0);

  buf += 'first';
  r = p.parse(buf);
  buf = r.remainder;
  // after non-whitespace arrives, parser should emit opening + content (html)
  const joined = r.segments.map((s) => s.data).join('');
  expect(joined).toContain('Рассуждения');
  expect(joined).toContain('first');

  buf += '</think>';
  r = p.parse(buf);
  buf = r.remainder;
  // closing may produce closing html in segments
  const joined2 = r.segments.map((s) => s.data).join('');
  expect(joined2).toContain('</pre></details>');
});
