use std::fs::File;
use std::io::Read;

#[test]
fn test_read_qwen3_metadata_and_tokenizer() {
    let md_path = std::path::Path::new("models/metadatas/qwen3.yaml");
    assert!(md_path.exists(), "qwen3 metadata not found");
    let mut f = File::open(md_path).expect("open qwen3.yaml");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("read qwen3.yaml");
    assert!(
        s.contains("tokenizer.ggml.tokens"),
        "qwen3 metadata missing tokenizer tokens"
    );
}

#[test]
fn test_read_gemma3_metadata_and_tokenizer() {
    let md_path = std::path::Path::new("models/metadatas/gemma3.yaml");
    assert!(md_path.exists(), "gemma3 metadata not found");
    let mut f = File::open(md_path).expect("open gemma3.yaml");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("read gemma3.yaml");
    assert!(
        s.contains("tokenizer.ggml.tokens"),
        "gemma3 metadata missing tokenizer tokens"
    );
}
