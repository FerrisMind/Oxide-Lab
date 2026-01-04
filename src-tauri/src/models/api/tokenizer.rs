//! Wrapper over tokenizer

use tokenizers::Tokenizer;

pub struct TokenizerWrapper {
    tokenizer: Tokenizer,
    eos_token_id: Option<u32>,
    bos_token_id: Option<u32>,
    stop_token_ids: Vec<u32>,
}

impl TokenizerWrapper {
    pub fn new(tokenizer: Tokenizer) -> Self {
        let mut wrapper = Self {
            tokenizer,
            eos_token_id: None,
            bos_token_id: None,
            stop_token_ids: Vec::new(),
        };
        wrapper.detect_special_tokens();
        wrapper
    }

    pub fn from_file(path: impl AsRef<std::path::Path>) -> super::error::Result<Self> {
        let tokenizer = Tokenizer::from_file(path)
            .map_err(|e| super::error::Error::Tokenizer(e.to_string()))?;
        Ok(Self::new(tokenizer))
    }

    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }

    pub fn encode(&self, text: &str, add_special_tokens: bool) -> super::error::Result<Vec<u32>> {
        let encoding = self
            .tokenizer
            .encode(text, add_special_tokens)
            .map_err(|e| super::error::Error::Tokenizer(e.to_string()))?;
        Ok(encoding.get_ids().to_vec())
    }

    pub fn decode(
        &self,
        tokens: &[u32],
        skip_special_tokens: bool,
    ) -> super::error::Result<String> {
        self.tokenizer
            .decode(tokens, skip_special_tokens)
            .map_err(|e| super::error::Error::Tokenizer(e.to_string()))
    }

    pub fn get_token_id(&self, token: &str) -> Option<u32> {
        self.tokenizer.get_vocab(true).get(token).copied()
    }

    pub fn eos_token_id(&self) -> Option<u32> {
        self.eos_token_id
    }
    pub fn bos_token_id(&self) -> Option<u32> {
        self.bos_token_id
    }

    pub fn stop_token_ids(&self) -> Vec<u32> {
        let mut ids = self.stop_token_ids.clone();
        if let Some(eos) = self.eos_token_id
            && !ids.contains(&eos)
        {
            ids.push(eos);
        }
        ids
    }

    pub fn add_stop_token(&mut self, token: &str) {
        if let Some(id) = self.get_token_id(token)
            && !self.stop_token_ids.contains(&id)
        {
            self.stop_token_ids.push(id);
        }
    }

    pub fn is_stop_token(&self, token_id: u32) -> bool {
        self.eos_token_id == Some(token_id) || self.stop_token_ids.contains(&token_id)
    }

    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(true)
    }

    fn detect_special_tokens(&mut self) {
        let vocab = self.tokenizer.get_vocab(true);

        // Try common EOS token patterns (using raw string to avoid XML issues)
        let eos_str = "\x3c/s\x3e"; // This is the same as the tag end
        if let Some(&id) = vocab.get(eos_str) {
            self.eos_token_id = Some(id);
            return;
        }

        // Try other patterns
        if let Some(&id) = vocab.get("\x3c|endoftext|\x3e") {
            self.eos_token_id = Some(id);
            return;
        }

        if let Some(&id) = vocab.get("\x3c|im_end|\x3e") {
            self.eos_token_id = Some(id);
            return;
        }

        if let Some(&id) = vocab.get("[EOS]") {
            self.eos_token_id = Some(id);
        }
    }
}
