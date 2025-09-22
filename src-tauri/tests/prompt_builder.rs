//! Test for the prompt builder functionality

use llm_chat_lib::core::prompt::{ChatMessage, PromptBuilder};

#[test]
fn test_prompt_builder_no_template() {
    let builder = PromptBuilder::new(None);
    assert!(!builder.has_template());

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

    let prompt = builder.build_fallback_prompt(messages);
    assert!(prompt.contains("user"));
    assert!(prompt.contains("assistant"));
    assert!(prompt.ends_with("assistant\n"));
}

#[test]
fn test_prompt_builder_with_template() {
    let template = "{% for message in messages %}{{ message.role }}: {{ message.content }}\n{% endfor %}{% if add_generation_prompt %}assistant:{% endif %}";
    let builder = PromptBuilder::new(Some(template.to_string()));
    assert!(builder.has_template());

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

    let result = builder.render_prompt(messages);
    assert!(result.is_ok());
    let prompt = result.unwrap();
    assert!(prompt.contains("user: Hello, how are you?"));
    assert!(prompt.contains("assistant: I'm doing well, thank you!"));
    assert!(prompt.ends_with("assistant:"));
}
