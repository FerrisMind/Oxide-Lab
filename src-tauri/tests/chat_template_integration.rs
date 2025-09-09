//! Test for the chat template integration functionality

use llm_chat_lib::core::types::ChatMessage;
use llm_chat_lib::generate::stream::build_prompt_with_template;

#[test]
fn test_build_prompt_with_template() {
    let template = Some("{% for message in messages %}{{ message.role }}: {{ message.content }}\n{% endfor %}{% if add_generation_prompt %}assistant:{% endif %}".to_string());
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "I'm doing well, thank you!".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "That's great to hear!".to_string(),
        },
    ];
    
    let result = build_prompt_with_template(&template, messages);
    assert!(result.is_ok());
    let prompt = result.unwrap();
    assert!(prompt.contains("user: Hello, how are you?"));
    assert!(prompt.contains("assistant: I'm doing well, thank you!"));
    assert!(prompt.contains("user: That's great to hear!"));
    assert!(prompt.ends_with("assistant:"));
}

#[test]
fn test_build_prompt_without_template() {
    let template = None;
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
    
    let result = build_prompt_with_template(&template, messages);
    assert!(result.is_ok());
    let prompt = result.unwrap();
    assert!(prompt.contains("user"));
    assert!(prompt.contains("assistant"));
    assert!(prompt.ends_with("assistant\n"));
}