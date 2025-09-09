use std::collections::HashMap;
use llm_chat_lib::models::registry::{detect_arch, ArchKind};
use candle::quantized::gguf_file::Value;

#[test]
fn test_architecture_detection() {
    // Test Qwen3 detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("qwen3".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Qwen3));
    
    // Test Llama detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("llama".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Llama));
    
    // Test Mistral detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("mistral".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Mistral));
    
    // Test Mixtral detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("mixtral".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Mixtral));
    
    // Test Gemma detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("gemma".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Gemma));
    
    // Test Phi3 detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("phi3".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Phi3));
    
    // Test Yi detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("yi".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Yi));
    
    // Test DeepSeek detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("deepseek".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::DeepSeek));
    
    // Test Pixtral detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("pixtral".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Pixtral));
    
    // Test SmolLM2 detection
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("smollm2".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::SmolLM2));
    
    // Test fallback detection for Qwen
    let mut metadata = HashMap::new();
    metadata.insert("some.key".to_string(), Value::String("qwen3-model-v1".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Qwen3));
    
    // Test fallback detection for Llama
    let mut metadata = HashMap::new();
    metadata.insert("some.key".to_string(), Value::String("llama-3-8b".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Llama));
    
    // Test unknown architecture
    let mut metadata = HashMap::new();
    metadata.insert("general.architecture".to_string(), Value::String("unknown".to_string()));
    assert_eq!(detect_arch(&metadata), None);
    
    // Test empty metadata
    let metadata = HashMap::new();
    assert_eq!(detect_arch(&metadata), None);
}