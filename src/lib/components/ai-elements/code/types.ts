import type { WithChildren, WithoutChildren } from 'bits-ui';
import type { SupportedLanguage } from './shiki';
import type { HTMLAttributes } from 'svelte/elements';

export type CodeVariant = 'default' | 'secondary';

export type CodeRootPropsWithoutHTML = WithChildren<{
  ref?: HTMLDivElement | null;
  variant?: CodeVariant;
  lang?: SupportedLanguage;
  code: string;
  class?: string;
  hideLines?: boolean;
  highlight?: (number | [number, number])[];
}>;

export type CodeRootProps = CodeRootPropsWithoutHTML &
  WithoutChildren<HTMLAttributes<HTMLDivElement>>;

export type CodeCopyButtonPropsWithoutHTML = {
  variant?: 'default' | 'ghost' | 'outline' | 'secondary' | 'destructive' | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  class?: string;
};

export type CodeCopyButtonProps = CodeCopyButtonPropsWithoutHTML &
  WithoutChildren<HTMLAttributes<HTMLButtonElement>>;

export type CodeOverflowPropsWithoutHTML = WithChildren<{
  collapsed?: boolean;
}>;

export type CodeOverflowProps = CodeOverflowPropsWithoutHTML &
  WithoutChildren<HTMLAttributes<HTMLDivElement>>;
