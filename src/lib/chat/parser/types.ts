export type StreamSegment = { kind: 'html' | 'text'; data: string };
export type ParseResult = { segments: StreamSegment[]; remainder: string };


