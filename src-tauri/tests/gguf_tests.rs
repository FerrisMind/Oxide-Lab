use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent directory")
        .to_path_buf()
}

fn metadata_dir() -> PathBuf {
    repo_root().join("models").join("metadatas")
}

fn read_to_string(path: &Path) -> String {
    let mut f = File::open(path).expect("open metadata file");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("read metadata file");
    s
}

fn find_qwen3_metadata_file() -> PathBuf {
    let dir = metadata_dir();
    let preferred = dir.join("qwen3.yaml");
    if preferred.exists() {
        return preferred;
    }

    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read models/metadatas directory")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("qwen3") && n.ends_with(".yaml"))
        })
        .collect();
    candidates.sort();
    candidates
        .into_iter()
        .next()
        .expect("no qwen3*.yaml metadata found")
}

#[test]
fn test_read_qwen3_metadata_and_tokenizer() {
    let md_path = find_qwen3_metadata_file();
    assert!(md_path.exists(), "qwen3 metadata not found");
    let s = read_to_string(&md_path);
    assert!(
        s.contains("tokenizer.ggml.tokens"),
        "qwen3 metadata missing tokenizer tokens"
    );
}

#[test]
fn test_read_gemma3_metadata_and_tokenizer() {
    let md_path = metadata_dir().join("gemma3.yaml");
    assert!(md_path.exists(), "gemma3 metadata not found");
    let s = read_to_string(&md_path);
    assert!(
        s.contains("tokenizer.ggml.tokens"),
        "gemma3 metadata missing tokenizer tokens"
    );
}
