use std::collections::HashMap;
use llm_chat_lib::models::registry::{ArchKind, detect_arch};
use candle::quantized::gguf_file::Value;

#[test]
fn test_supported_models_detection() {
    let mut metadata = HashMap::new();
    
    // Test Llama detection
    metadata.insert("general.architecture".to_string(), Value::String("llama".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Llama));
    metadata.clear();
    
    // Test Mistral detection
    metadata.insert("general.architecture".to_string(), Value::String("mistral".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Mistral));
    metadata.clear();
    
    // Test Mixtral detection
    metadata.insert("general.architecture".to_string(), Value::String("mixtral".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Mixtral));
    metadata.clear();
    
    // Test Gemma detection
    metadata.insert("general.architecture".to_string(), Value::String("gemma".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Gemma));
    metadata.clear();
    
    // Test Qwen detection
    metadata.insert("general.architecture".to_string(), Value::String("qwen3".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Qwen3));
    metadata.clear();
    
    // Test Yi detection
    metadata.insert("general.architecture".to_string(), Value::String("yi".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Yi));
    metadata.clear();
    
    // Test Phi3 detection
    metadata.insert("general.architecture".to_string(), Value::String("phi3".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Phi3));
    metadata.clear();
    
    // Test DeepSeek detection
    metadata.insert("general.architecture".to_string(), Value::String("deepseek".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::DeepSeek));
    metadata.clear();
    
    // Test Pixtral detection
    metadata.insert("general.architecture".to_string(), Value::String("pixtral".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Pixtral));
    metadata.clear();
    
    // Test SmolLM2 detection
    metadata.insert("general.architecture".to_string(), Value::String("smollm2".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::SmolLM2));
    metadata.clear();
}

#[test]
fn test_fallback_detection() {
    let mut metadata = HashMap::new();
    
    // Test Llama fallback detection
    metadata.insert("model.name".to_string(), Value::String("meta-llama-3-8b".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Llama));
    metadata.clear();
    
    // Test Mistral fallback detection
    metadata.insert("model.name".to_string(), Value::String("mistral-7b-instruct".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Mistral));
    metadata.clear();
    
    // Test Qwen fallback detection
    metadata.insert("model.name".to_string(), Value::String("qwen3-7b".to_string()));
    assert_eq!(detect_arch(&metadata), Some(ArchKind::Qwen3));
    metadata.clear();
}