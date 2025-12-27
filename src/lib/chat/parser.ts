/**
 * Stream Parser
 * 
 * Parses streaming tokens into segments for rendering.
 * Handles cases where model may not emit opening <think> tag.
 */

export type SegmentKind = 'text' | 'think_start' | 'think_end' | 'think_content';

export type Segment = {
    kind: SegmentKind;
    data: string;
};

export type ParseResult = {
    segments: Segment[];
    remainder: string;
};

/**
 * Creates a stream parser for handling token streams.
 */
export function createStreamParser() {
    let inThinkBlock = false;

    function parse(buffer: string): ParseResult {
        const segments: Segment[] = [];
        let current = buffer;
        let remainder = '';

        while (current.length > 0) {
            if (!inThinkBlock) {
                // First check if there's a closing </think> without opening tag
                // Some models start with thinking content without explicit <think>
                const thinkEnd = current.indexOf('</think>');
                const thinkStart = current.indexOf('<think>');

                // If we find </think> before or without <think>, treat content before it as thinking
                if (thinkEnd !== -1 && (thinkStart === -1 || thinkEnd < thinkStart)) {
                    if (thinkEnd > 0) {
                        // Implicit think block - all content before </think> is thinking
                        segments.push({ kind: 'think_start', data: '' });
                        segments.push({ kind: 'think_content', data: current.slice(0, thinkEnd) });
                    }
                    segments.push({ kind: 'think_end', data: '' });
                    current = current.slice(thinkEnd + 8); // Skip </think>
                    continue;
                }

                // Look for <think> tag
                if (thinkStart === -1) {
                    // No think tag, check if we might have a partial tag at the end
                    if (current.endsWith('<') ||
                        current.endsWith('<t') ||
                        current.endsWith('<th') ||
                        current.endsWith('<thi') ||
                        current.endsWith('<thin') ||
                        current.endsWith('<think') ||
                        current.endsWith('</') ||
                        current.endsWith('</t') ||
                        current.endsWith('</th') ||
                        current.endsWith('</thi') ||
                        current.endsWith('</thin') ||
                        current.endsWith('</think')) {
                        const lastLt = current.lastIndexOf('<');
                        if (lastLt > 0) {
                            segments.push({ kind: 'text', data: current.slice(0, lastLt) });
                        }
                        remainder = current.slice(lastLt);
                        break;
                    }
                    segments.push({ kind: 'text', data: current });
                    break;
                }

                if (thinkStart > 0) {
                    segments.push({ kind: 'text', data: current.slice(0, thinkStart) });
                }
                segments.push({ kind: 'think_start', data: '' });
                inThinkBlock = true;
                current = current.slice(thinkStart + 7); // Skip <think>
            } else {
                // Inside think block, look for </think>
                const thinkEnd = current.indexOf('</think>');
                if (thinkEnd === -1) {
                    // Check for partial closing tag
                    if (current.endsWith('<') ||
                        current.endsWith('</') ||
                        current.endsWith('</t') ||
                        current.endsWith('</th') ||
                        current.endsWith('</thi') ||
                        current.endsWith('</thin') ||
                        current.endsWith('</think')) {
                        const lastLt = current.lastIndexOf('<');
                        if (lastLt > 0) {
                            segments.push({ kind: 'think_content', data: current.slice(0, lastLt) });
                        }
                        remainder = current.slice(lastLt);
                        break;
                    }
                    segments.push({ kind: 'think_content', data: current });
                    break;
                }

                if (thinkEnd > 0) {
                    segments.push({ kind: 'think_content', data: current.slice(0, thinkEnd) });
                }
                segments.push({ kind: 'think_end', data: '' });
                inThinkBlock = false;
                current = current.slice(thinkEnd + 8); // Skip </think>
            }
        }

        return { segments, remainder };
    }

    function reset() {
        inThinkBlock = false;
    }

    function setInThinkBlock(value: boolean) {
        inThinkBlock = value;
    }

    function isInThinkBlock() {
        return inThinkBlock;
    }

    return { parse, reset, setInThinkBlock, isInThinkBlock };
}
