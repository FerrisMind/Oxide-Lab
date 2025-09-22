//! Integration test for chat template functionality in the generation pipeline

use llm_chat_lib::core::types::{ChatMessage, GenerateRequest};

#[test]
fn test_generate_request_with_chat_messages() {
    // Test that GenerateRequest can be created with chat messages
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "I'm doing well, thank you!".to_string(),
        },
    ];

    let req = GenerateRequest {
        prompt: "Direct prompt".to_string(),
        messages: Some(messages),
        max_new_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        min_p: None,
        repeat_penalty: None,
        repeat_last_n: 64,
        use_custom_params: false,
        seed: None,
    };

    assert_eq!(req.prompt, "Direct prompt");
    assert!(req.messages.is_some());
    assert_eq!(req.messages.as_ref().unwrap().len(), 2);
}

#[test]
fn test_generate_request_without_chat_messages() {
    // Test that GenerateRequest can be created without chat messages (backward compatibility)
    let req = GenerateRequest {
        prompt: "Direct prompt".to_string(),
        messages: None,
        max_new_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        min_p: None,
        repeat_penalty: None,
        repeat_last_n: 64,
        use_custom_params: false,
        seed: None,
    };

    assert_eq!(req.prompt, "Direct prompt");
    assert!(req.messages.is_none());
}
