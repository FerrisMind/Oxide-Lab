use candle::Result;

/// Потоковая декодирующая обёртка для токенов, позволяет выдавать прирост текста по мере генерации
pub struct TokenOutputStream {
    tokenizer: tokenizers::Tokenizer,
    tokens: Vec<u32>,
    prev_index: usize,
    current_index: usize,
}

impl TokenOutputStream {
    pub fn new(tokenizer: tokenizers::Tokenizer) -> Self {
        Self { tokenizer, tokens: Vec::new(), prev_index: 0, current_index: 0 }
    }

    fn decode(&self, tokens: &[u32]) -> Result<String> {
        match self.tokenizer.decode(tokens, true) {
            Ok(str_) => Ok(str_),
            Err(err) => candle::bail!("cannot decode: {err}"),
        }
    }

    pub fn next_token(&mut self, token: u32) -> Result<Option<String>> {
        let prev_text = if self.tokens.is_empty() {
            String::new()
        } else {
            let tokens = &self.tokens[self.prev_index..self.current_index];
            self.decode(tokens)?
        };
        self.tokens.push(token);
        let text = self.decode(&self.tokens[self.prev_index..])?;
        if text.len() > 0 {
            // Найти границу расхождения по символам (а не по байтам), чтобы избежать разреза внутри UTF-8
            let mut split_byte = 0usize;
            let mut prev_iter = prev_text.chars();
            for (byte_idx, ch) in text.char_indices() {
                match prev_iter.next() {
                    Some(pc) if pc == ch => {
                        // конец совпавшего символа
                        split_byte = byte_idx + ch.len_utf8();
                    }
                    _ => break,
                }
            }
            if split_byte < text.len() {
                let delta = text[split_byte..].to_string();
                // Если приращение заканчивается символами, которые часто бывают частью
                // составных графем (U+FFFD, ZWJ, variation selectors, skin tone modifiers),
                // подождём следующий токен, чтобы избежать артефактов вроде "�".
                let mut hold = false;
                if let Some(last) = delta.chars().last() {
                    if last == '\u{FFFD}' || last == '\u{200D}' || last == '\u{FE0F}'
                    || ('\u{1F3FB}'..='\u{1F3FF}').contains(&last) {
                        hold = true;
                    }
                }
                if hold {
                    return Ok(None);
                }
                self.prev_index = self.current_index;
                self.current_index = self.tokens.len();
                Ok(Some(delta))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn decode_rest(&self) -> Result<Option<String>> {
        let prev_text = if self.tokens.is_empty() {
            String::new()
        } else {
            let tokens = &self.tokens[self.prev_index..self.current_index];
            self.decode(tokens)?
        };
        let text = self.decode(&self.tokens[self.prev_index..])?;
        if text.is_empty() {
            return Ok(None);
        }
        // Найти общий префикс по символам
        let mut split_byte = 0usize;
        let mut prev_iter = prev_text.chars();
        for (byte_idx, ch) in text.char_indices() {
            match prev_iter.next() {
                Some(pc) if pc == ch => {
                    split_byte = byte_idx + ch.len_utf8();
                }
                _ => break,
            }
        }
        if split_byte < text.len() {
            Ok(Some(text[split_byte..].to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn tokenizer(&self) -> &tokenizers::Tokenizer { &self.tokenizer }

    #[allow(dead_code)]
    pub fn clear(&mut self) { self.tokens.clear(); self.prev_index = 0; self.current_index = 0; }
}


