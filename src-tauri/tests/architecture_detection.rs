use candle::quantized::gguf_file::Value;
use llm_chat_lib::models::registry::{ArchKind, detect_arch};
use std::collections::HashMap;

#[test]
fn test_architecture_detection() {
    // Test Qwen3 detection
    let mut metadata = HashMap::new();
    metadata.insert(
        "general.architecture".to_string(),
        Value::String("qwen3".to_string()),
    );
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Qwen3));

    // Test fallback detection for Qwen
    let mut metadata = HashMap::new();
    metadata.insert(
        "some.key".to_string(),
        Value::String("qwen3-model-v1".to_string()),
    );
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Qwen3));

    // Test unknown architecture (Llama is not implemented yet)
    let mut metadata = HashMap::new();
    metadata.insert(
        "general.architecture".to_string(),
        Value::String("llama".to_string()),
    );
    assert_eq!(detect_arch(&metadata), None); // Llama is not registered yet

    // Test unknown architecture
    let mut metadata = HashMap::new();
    metadata.insert(
        "general.architecture".to_string(),
        Value::String("unknown".to_string()),
    );
    assert_eq!(detect_arch(&metadata), None);

    // Test empty metadata
    let metadata = HashMap::new();
    assert_eq!(detect_arch(&metadata), None);
}
