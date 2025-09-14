//! Test file for the ModelBuilder implementation

#[cfg(test)]
mod tests {
    use crate::models::common::builder::ModelFactory;
    use crate::models::qwen3::builder::Qwen3ModelBuilder;
    use crate::models::registry::ArchKind;
    use std::collections::HashMap;

    #[test]
    fn test_model_factory_creation() {
        let factory = ModelFactory::new();
        // We can't directly access the private builders field, but we can test the factory's behavior
        let metadata = HashMap::new();
        let result = factory.detect_gguf_arch(&metadata);
        assert_eq!(result, None);
    }

    #[test]
    fn test_qwen3_builder_arch_kind() {
        let builder = Qwen3ModelBuilder::new();
        assert_eq!(builder.arch_kind(), ArchKind::Qwen3);
    }
}
