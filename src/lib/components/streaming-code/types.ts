export interface StreamingCodeBlockProps {
  code: string;
  language: string;
  isStreaming: boolean;
  theme?: 'light' | 'dark' | 'auto';
  showLineNumbers?: boolean;
  readonly?: boolean;
}

export interface StreamingCodeBlockEvents {
  change: { code: string };
  toggle: { expanded: boolean };
  streamingTimeout: void;
}

export interface ProgressBarProps {
  language: string;
  isStreaming: boolean;
  isExpanded: boolean;
}

export interface ProgressBarEvents {
  click: void;
}

export type StreamingState = 'idle' | 'streaming' | 'completed';

export interface CodeStreamingContext {
  state: StreamingState;
  code: string;
  language: string;
  isExpanded: boolean;
  lastUpdateTime: number;
}
