pub struct ContextSlice {
    pub encoded_len: usize,
    pub base_context_len: usize,
    pub effective_context_tokens: Vec<u32>,
}

impl ContextSlice {
    pub fn new(full_context_tokens: Vec<u32>, limit: usize) -> Self {
        let encoded_len = full_context_tokens.len();
        let effective_context_tokens = if encoded_len > limit && limit > 0 {
            let start = encoded_len - limit;
            full_context_tokens[start..].to_vec()
        } else {
            full_context_tokens.clone()
        };
        let base_context_len = effective_context_tokens.len();
        Self { encoded_len, base_context_len, effective_context_tokens }
    }
}


