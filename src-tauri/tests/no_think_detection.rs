use llm_chat_lib::generate::stream::detect_no_think;
use llm_chat_lib::core::types::GenerateRequest;

#[test]
fn detect_no_think_simple() {
    let r = GenerateRequest { 
        prompt: "Hello <think>hidden</think> world".to_string(),
        messages: None, use_custom_params: false, temperature: None, top_p: None, top_k: None, min_p: None, repeat_last_n: 0, repeat_penalty: None, seed: None };
    assert!(detect_no_think(&r));
}

#[test]
fn detect_no_think_case_insensitive() {
    let r = GenerateRequest { prompt: "Hello <THINK>secret</THINK>".to_string(),
        messages: None, use_custom_params: false, temperature: None, top_p: None, top_k: None, min_p: None, repeat_last_n: 0, repeat_penalty: None, seed: None };
    assert!(detect_no_think(&r));
}

#[test]
fn detect_no_think_absent() {
    let r = GenerateRequest { prompt: "No tags here".to_string(),
        messages: None, use_custom_params: false, temperature: None, top_p: None, top_k: None, min_p: None, repeat_last_n: 0, repeat_penalty: None, seed: None };
    assert!(!detect_no_think(&r));
}


