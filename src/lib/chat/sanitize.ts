export function sanitizeForPrompt(input: string): string {
  let r = input.replace(/<think>[\s\S]*?<\/think>/gi, "");
  r = r
    .replace(/<\|im_start\|>/g, "")
    .replace(/<\|im_end\|>/g, "")
    .replace(/<\|user\|>/g, "")
    .replace(/<\|assistant\|>/g, "")
    .replace(/<\|system\|>/g, "")
    .replace(/<\|start_header_id\|>[\s\S]*?<\|end_header_id\|>/g, "")
    .replace(/<\|eot_id\|>/g, "")
    .replace(/<\|eom_id\|>/g, "")
    .replace(/<\|endoftext\|>/g, "")
    .replace(/<\|end_of_text\|>/g, "")
    .replace(/<\|begin_of_sentence\|>/g, "")
    .replace(/<\|end_of_sentence\|>/g, "")
    .replace(/<\|reserved_[^|]*\|>/g, "")
    // tool calling
    .replace(/<tool_call>[\s\S]*?<\/tool_call>/gi, "")
    .replace(/<tool_response>[\s\S]*?<\/tool_response>/gi, "")
    // code blocks
    .replace(/<\|code\|>[\s\S]*?<\|endcode\|>/gi, "")
    .replace(/<\|python\|>[\s\S]*?<\|\/python\|>/gi, "")
    // media
    .replace(/<\|image\|>[\s\S]*?<\|\/image\|>/gi, "")
    .replace(/<\|audio\|>[\s\S]*?<\|\/audio\|>/gi, "")
    .replace(/<\|video\|>[\s\S]*?<\|\/video\|>/gi, "")
    .replace(/<s>/g, "")
    .replace(/<\/s>/g, "");
  return r;
}


