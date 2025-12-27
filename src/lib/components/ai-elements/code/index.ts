import { tv } from 'tailwind-variants';
import Root from './Code.svelte';
import CopyButton from './CodeCopyButton.svelte';
import type { CodeCopyButtonProps, CodeRootProps, CodeVariant } from './types';

export const codeVariants = tv({
  base: 'not-prose relative h-full overflow-auto rounded-lg border',
  variants: {
    variant: {
      default: 'border-border bg-card',
      secondary: 'bg-secondary/50 border-transparent',
    },
  },
});

export type { CodeVariant };

export {
  Root,
  CopyButton,
  Root as Code,
  type CodeRootProps as RootProps,
  type CodeCopyButtonProps as CopyButtonProps,
};
