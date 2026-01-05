#[cfg(test)]
mod tests {
    use oxide_lib::core::types::ChatMessage;
    use oxide_lib::generate::ctx::smart_truncate;
    use tokenizers::Tokenizer;

    // A minimal tokenizer JSON that uses WordLevel model and Whitespace pre-tokenizer.
    // This allows us to predict token counts: roughly 1 word = 1 token (if in vocab) or [UNK].
    const MINIMAL_TOKENIZER_JSON: &str = r#"{
      "version": "1.0",
      "truncation": null,
      "padding": null,
      "added_tokens": [
        { "id": 0, "content": "[UNK]", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true },
        { "id": 1, "content": "<|system|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true },
        { "id": 2, "content": "<|user|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true },
        { "id": 3, "content": "<|assistant|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true },
        { "id": 4, "content": "<|end|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true }
      ],
      "normalizer": null,
      "pre_tokenizer": {
        "type": "Whitespace"
      },
      "post_processor": null,
      "decoder": null,
      "model": {
        "type": "WordLevel",
        "vocab": {
          "[UNK]": 0,
          "<|system|>": 1,
          "<|user|>": 2,
          "<|assistant|>": 3,
          "<|end|>": 4,
          "hello": 5,
          "world": 6,
          "test": 7,
          "this": 8,
          "is": 9,
          "a": 10,
          "message": 11,
          "long": 12,
          "context": 13,
          "keep": 14,
          "me": 15,
          "system": 16,
          "prompt": 17
        },
        "unk_token": "[UNK]"
      }
    }"#;

    fn create_dummy_tokenizer() -> Tokenizer {
        Tokenizer::from_bytes(MINIMAL_TOKENIZER_JSON.as_bytes())
            .expect("Failed to create dummy tokenizer")
    }

    fn create_msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    // A simple chat template that adds minimal overhead
    // Format: <|role|> content <|end|>
    const SIMPLE_TEMPLATE: &str = "{{ bos_token }}{% for message in messages %}<|{{ message.role }}|> {{ message.content }} <|end|>{% endfor %}";

    #[test]
    fn test_smart_truncate_empty() {
        let tokenizer = create_dummy_tokenizer();
        let msgs = vec![];
        let res = smart_truncate(&tokenizer, &None, &msgs, None, 100);
        assert_eq!(res.unwrap(), "");
    }

    #[test]
    fn test_smart_truncate_fits_all() {
        let tokenizer = create_dummy_tokenizer();
        let msgs = vec![
            create_msg("system", "system prompt"),
            create_msg("user", "hello world"),
        ];
        let template = Some(SIMPLE_TEMPLATE.to_string());

        // "system prompt" -> <|system|> system prompt <|end|> ~= 1+2+1 = 4 tokens (system, [UNK], [UNK], end)?
        // Actually "system" is in vocab(16), "prompt" is in vocab(17). So 4 tokens.
        // "hello world" -> <|user|> hello world <|end|> ~= 1+1+1+1 = 4 tokens.
        // Total ~8 tokens. Limit 100.

        let res = smart_truncate(&tokenizer, &template, &msgs, None, 100).unwrap();

        assert!(res.contains("system prompt"));
        assert!(res.contains("hello world"));
    }

    #[test]
    fn test_smart_truncate_preserves_system_and_last() {
        let tokenizer = create_dummy_tokenizer();
        let template = Some(SIMPLE_TEMPLATE.to_string());

        // System: 4 tokens
        let sys_msg = create_msg("system", "system prompt");

        // Filler messages: "this is a message" (4 words -> 4 tokens + 2 overhead = 6 tokens each)
        // We add many fillers to overflow logic
        let mut msgs = vec![sys_msg];
        for i in 0..10 {
            msgs.push(create_msg("user", &format!("filler message {}", i)));
        }

        // Last message: "keep me" (2 words -> 2 tokens + 2 overhead = 4 tokens)
        msgs.push(create_msg("user", "keep me"));

        // Total messages: 1 system + 10 fillers + 1 last = 12 messages.
        // Approx tokens = 4 + 10*6 + 4 = 68 tokens.

        // Set limit very low.
        // System (4) + Last (4) = 8. + overheads.
        // Let's set limit to 15.
        // Should keep System, Last, and maybe 1 filler?

        let res = smart_truncate(&tokenizer, &template, &msgs, None, 15).unwrap();

        // Check System preservation
        assert!(
            res.contains("system prompt"),
            "System prompt must be preserved"
        );

        // Check Last Message preservation
        assert!(res.contains("keep me"), "Last message must be preserved");

        // Check Truncation (oldest fillers should be gone)
        // "filler message 0" is the oldest filler.
        assert!(
            !res.contains("filler message 0"),
            "Oldest filler should be truncated"
        );

        // "filler message 9" is newest filler. Might be kept if space allows.
        // 15 - 8 = 7 tokens left. "filler message 9" takes ~6. It might fit.
    }

    #[test]
    fn test_smart_truncate_hard_limit() {
        let tokenizer = create_dummy_tokenizer();
        let template = Some(SIMPLE_TEMPLATE.to_string());

        let msgs = vec![
            create_msg("system", "system prompt"), // ~4 len
            create_msg("user", "hello world"),     // ~4 len
        ];

        // Limit 5. System takes 4. Remaining 1. "hello world" takes 4.
        // Should return System + hello world (smart_truncate logic: prioritized last message even if it slightly exceeds?
        // Wait, the logic says: "If we are at the last message and it still doesn't fit... We must return it (System + Last)".
        // So expected result is valid string containing both, even if it exceeds limit slightly (ContextSlice handles hard clip later).

        let res = smart_truncate(&tokenizer, &template, &msgs, None, 5).unwrap();
        assert!(res.contains("system prompt"));
        assert!(res.contains("hello world"));
    }
}
