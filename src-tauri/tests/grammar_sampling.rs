//! Integration tests for grammar sampling

use oxide_lib::generate::grammar::{GrammarSampler, OutputFormat, validate_json};

#[test]
fn test_output_format_none() {
    let fmt = OutputFormat::None;
    assert!(!fmt.requires_grammar());
    assert!(!fmt.is_json_mode());
}

#[test]
fn test_output_format_json_mode() {
    let fmt = OutputFormat::Json;
    assert!(fmt.requires_grammar());
    assert!(fmt.is_json_mode());
}

#[test]
fn test_output_format_json_schema() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        }
    });
    let fmt = OutputFormat::JsonSchema(schema);
    assert!(fmt.requires_grammar());
    assert!(!fmt.is_json_mode());
}

#[test]
fn test_grammar_sampler_empty() {
    let sampler = GrammarSampler::new();
    assert!(!sampler.is_complete());
    assert_eq!(sampler.depth(), 0);
}

#[test]
fn test_grammar_sampler_simple_object() {
    let mut sampler = GrammarSampler::new();
    sampler.update("{\"key\": \"value\"}");
    assert!(sampler.is_complete());
    assert_eq!(sampler.depth(), 0);
}

#[test]
fn test_grammar_sampler_nested_object() {
    let mut sampler = GrammarSampler::new();
    sampler.update("{\"outer\": {\"inner\": 42}}");
    assert!(sampler.is_complete());
}

#[test]
fn test_grammar_sampler_array() {
    let mut sampler = GrammarSampler::new();
    sampler.update("[1, 2, 3]");
    assert!(sampler.is_complete());
}

#[test]
fn test_grammar_sampler_incomplete() {
    let mut sampler = GrammarSampler::new();
    sampler.update("{\"key\":");
    assert!(!sampler.is_complete());
    assert!(sampler.depth() > 0);
}

#[test]
fn test_validate_json_valid() {
    let result = validate_json("{\"test\": 123}");
    assert!(result.is_ok());
}

#[test]
fn test_validate_json_invalid() {
    let result = validate_json("not json at all");
    assert!(result.is_err());
}

#[test]
fn test_validate_json_array() {
    let result = validate_json("[1, \"two\", true]");
    assert!(result.is_ok());
}
