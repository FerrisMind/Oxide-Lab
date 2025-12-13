use candle::quantized::gguf_file::Value;
use oxide_lib::models::registry::{ArchKind, detect_arch};
use std::collections::HashMap;

#[test]
fn test_supported_models_detection() {
    let mut metadata = HashMap::new();

    // Test Qwen detection (only Qwen3 is currently implemented)
    metadata.insert(
        "general.architecture".to_string(),
        Value::String("qwen3".to_string()),
    );
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Qwen3));
    metadata.clear();

    // Other architectures are not implemented yet, so they should return None
    metadata.insert(
        "general.architecture".to_string(),
        Value::String("llama".to_string()),
    );
    assert_eq!(detect_arch(&metadata), None); // Not implemented yet
    metadata.clear();

    metadata.insert(
        "general.architecture".to_string(),
        Value::String("mistral".to_string()),
    );
    assert_eq!(detect_arch(&metadata), None); // Not implemented yet
    metadata.clear();
}

#[test]
fn test_fallback_detection() {
    let mut metadata = HashMap::new();

    // Test Qwen fallback detection (only Qwen3 is currently implemented)
    metadata.insert(
        "model.name".to_string(),
        Value::String("qwen3-7b".to_string()),
    );
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Qwen3));
    metadata.clear();

    // Other architectures are not implemented yet, so they should return None
    metadata.insert(
        "model.name".to_string(),
        Value::String("meta-llama-3-8b".to_string()),
    );
    assert_eq!(detect_arch(&metadata), None); // Not implemented yet
    metadata.clear();
}
