mod gui;
use structopt::StructOpt;

use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(name = "gguf-inspector")]
struct Opt {
    /// Run GUI application
    #[structopt(long)]
    gui: bool,

    /// Directory with pre-extracted metadata YAML files to validate
    #[structopt(long, parse(from_os_str))]
    metadata_dir: Option<PathBuf>,

    /// Path to GGUF file for CLI export
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Output JSON file (CLI only)
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    if opt.gui {
        let native_options = eframe::NativeOptions::default();
        let _ = eframe::run_native("GGUF Inspector", native_options, Box::new(|_cc| Ok(Box::new(gui::GgufApp::default()))));
        return Ok(());
    }

    // Если не указаны входные аргументы, по умолчанию проверим каталог GGUF в корне проекта
    if opt.input.is_none() && opt.metadata_dir.is_none() {
        // Try to detect repository root by looking for README.md or .git at current or parent directories
        let mut cwd = std::env::current_dir()?;
        let mut repo_root: Option<PathBuf> = None;
        for _ in 0..6 {
            if cwd.join("README.md").exists() || cwd.join(".git").exists() {
                repo_root = Some(cwd.clone());
                break;
            }
            if !cwd.pop() { break; }
        }
        if let Some(root) = repo_root {
            let default_gguf_dir = root.join("models/gguf");
            if default_gguf_dir.exists() {
                check_gguf_dir(&default_gguf_dir)?;
                return Ok(());
            }
        }
    }

    // CLI mode: если указана директория с YAML метаданными — проверим её
    if let Some(ref dir) = opt.metadata_dir {
        check_metadata_dir(dir)?;
        return Ok(());
    }

    // CLI mode: fallback to previous behavior if input provided
    if let Some(input) = opt.input {
        let mut f = std::fs::File::open(&input)?;
        let mut buf = Vec::new();
        use std::io::Read;
        f.read_to_end(&mut buf)?;
        let mut cursor = std::io::Cursor::new(&buf);
        let content = candle::quantized::gguf_file::Content::read(&mut cursor)?;

        let mut map = serde_json::Map::new();
        let mut keys = Vec::new();
        for (k, _v) in content.metadata.iter() {
            keys.push(k.clone());
        }
        // Convert any variant to a readable string (second pass)
        for (k, v) in content.metadata.iter() {
            let s = gguf_inspector::format::readable_value(v);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&s) {
                map.insert(k.clone(), json);
            } else {
                map.insert(k.clone(), serde_json::Value::String(s));
            }
        }
        let exported = serde_json::json!({"keys": keys, "raw": serde_json::Value::Object(map)});
        let out_path = match opt.output {
            Some(p) => p,
            None => input.with_extension("gguf.metadata.json"),
        };
        std::fs::write(out_path, serde_json::to_string_pretty(&exported)?)?;
        println!("OK");
    } else {
        eprintln!("No input provided. Use --gui to run the GUI or provide input path for CLI.");
    }
    Ok(())
}

fn check_metadata_dir(dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() { return Err(format!("Not a directory: {}", dir.display()).into()); }
    for entry in std::fs::read_dir(dir)? {
        let ent = entry?;
        let path = ent.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
                println!("Checking {}...", path.display());
                let bytes = std::fs::read(&path)?;
                let yaml: serde_yaml::Value = serde_yaml::from_slice(&bytes)?;
                // Flatten to map of strings for basic validation
                if let Some(map) = yaml.as_mapping() {
                    let mut has_tokenizer = false;
                    let mut has_config = false;
                    for (k, _v) in map.iter() {
                        if let Some(kstr) = k.as_str() {
                            if kstr.contains("tokenizer") { has_tokenizer = true; }
                            if kstr.contains("config") { has_config = true; }
                        }
                    }
                    println!("  keys: {} entries, tokenizer_in_metadata={}, config_in_metadata={}", map.len(), has_tokenizer, has_config);
                } else {
                    println!("  not a mapping — skipping");
                }
            }
        }
    }
    Ok(())
}

fn check_gguf_dir(dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() { return Err(format!("Not a directory: {}", dir.display()).into()); }
    for entry in std::fs::read_dir(dir)? {
        let ent = entry?;
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("gguf")).unwrap_or(false) {
            println!("Reading {}...", path.display());
            let mut f = std::fs::File::open(&path)?;
            use std::io::Read;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            let mut cursor = std::io::Cursor::new(&buf);
            let content = candle::quantized::gguf_file::Content::read(&mut cursor)?;
            let keys: Vec<String> = content.metadata.keys().cloned().collect();
            let _joined = keys.join(", ");
            // Проверяем наличие токенизатора/конфига в метаданных
            let has_tokenizer = keys.iter().any(|k| k.contains("tokenizer") || k.contains("tokenizer.json") || k.contains("tokenizer.ggml"));
            let has_config = keys.iter().any(|k| k.contains("config") || k.contains("config.json") || k.contains("general.config_json"));
            println!("  keys_count={}, tokenizer_in_metadata={}, config_in_metadata={}", keys.len(), has_tokenizer, has_config);
            if !has_tokenizer {
                println!("  WARNING: tokenizer not found in GGUF metadata for {}", path.display());
            }
            if !has_config {
                println!("  WARNING: config not found in GGUF metadata for {}", path.display());
            }
        }
    }
    Ok(())
}


