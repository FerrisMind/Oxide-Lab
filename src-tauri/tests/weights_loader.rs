//! Tests for the universal weights loader functionality

#[cfg(test)]
mod tests {
    use oxide_lib::core::weights;

    #[test]
    fn test_local_list_safetensors_empty() {
        // Test with a directory that doesn't exist or has no safetensors files
        let result = weights::local_list_safetensors("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_hub_list_safetensors_function_exists() {
        // Just test that the function exists and has the right signature
        // We can't test actual Hub functionality without network access
        let _func = weights::hub_list_safetensors;
    }

    #[test]
    fn test_local_list_safetensors_function_exists() {
        // Just test that the function exists and has the right signature
        let _func = weights::local_list_safetensors::<&str>;
    }

    #[test]
    fn test_build_varbuilder_function_exists() {
        // Just test that the function exists and has the right signature
        let _func = weights::build_varbuilder;
    }

    #[test]
    fn test_validate_safetensors_files_function_exists() {
        // Just test that the function exists and has the right signature
        let _func = weights::validate_safetensors_files;
    }

    #[test]
    fn test_validate_safetensors_files_empty() {
        // Test validation with empty list
        let result = weights::validate_safetensors_files(&[]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("No safetensors paths provided")
        );
    }
}
