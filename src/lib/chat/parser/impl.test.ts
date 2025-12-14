import { test, expect } from 'vitest';
import { createStreamParser } from './impl';

test('createStreamParser - empty <think> produces no segments', () => {
  const p = createStreamParser();
  const { segments } = p.parse('<think></think>');
  const joined = segments.map((s) => s.data).join('');
  expect(joined).toContain('<think></think>');
});

test('createStreamParser - whitespace-only <think> produces no segments', () => {
  const p = createStreamParser();
  const { segments } = p.parse('<think>   \n\t  \n</think>');
  const joined = segments.map((s) => s.data).join('');
  expect(joined).toContain('<think>');
  expect(joined).toContain('</think>');
});

test('createStreamParser - non-empty <think> renders details and content', () => {
  const p = createStreamParser();
  const { segments } = p.parse('<think>hello world</think>');
  const joined = segments.map((s) => s.data).join('');
  expect(joined).toContain('<think>');
  expect(joined).toContain('</think>');
  expect(joined).toContain('hello world');
});

test('createStreamParser - streaming think opens only after non-empty content arrives', () => {
  const p = createStreamParser();
  let buf = '';
  const collected: string[] = [];
  buf += '<think>';
  let r = p.parse(buf);
  buf = r.remainder;
  collected.push(...r.segments.map((s) => s.data));

  buf += '\n\n';
  r = p.parse(buf);
  buf = r.remainder;
  collected.push(...r.segments.map((s) => s.data));

  buf += 'first';
  r = p.parse(buf);
  buf = r.remainder;
  collected.push(...r.segments.map((s) => s.data));
  const joined = collected.join('');
  expect(joined).toContain('<think');
  expect(joined).toContain('first');

  buf += '</think>';
  r = p.parse(buf);
  buf = r.remainder;
  const joined2 = r.segments.map((s) => s.data).join('');
  expect(joined2).toContain('</think>');
});
