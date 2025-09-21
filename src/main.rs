use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use candle::quantized::gguf_file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace this path with the actual path to your Gemma 3 GGUF file
    let path = std::env::args().nth(1).expect("Usage: test_gemma3_tokenizer <gguf-file-path>");
    
    println!("Testing Gemma 3 GGUF file: {}", path);
    
    let mut f = File::open(&path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let mut cursor = std::io::Cursor::new(&buf);
    let content = gguf_file::Content::read(&mut cursor)?;
    
    println!("Successfully read GGUF file");
    println!("Total metadata keys: {}", content.metadata.len());
    
    // Look for tokenizer-related keys
    let tokenizer_keys: Vec<&String> = content.metadata.keys()
        .filter(|k| k.contains("tokenizer"))
        .collect();
    
    println!("Tokenizer-related keys found: {}", tokenizer_keys.len());
    for key in &tokenizer_keys {
        println!("  - {}", key);
    }
    
    // Try to extract and display the tokenizer JSON
    if let Some(tokenizer_json) = find_tokenizer_json_in_metadata(&content.metadata) {
        println!("\nFound tokenizer JSON (length: {} chars)", tokenizer_json.len());
        
        // Try to parse with the tokenizers library
        match tokenizers::Tokenizer::from_bytes(tokenizer_json.as_bytes()) {
            Ok(tokenizer) => {
                println!("Tokenizer parsed successfully!");
                println!("Tokenizer model type: {:?}", tokenizer.get_model().get_type());
                println!("Vocabulary size: {}", tokenizer.get_vocab_size(true));
                
                // Test encoding a simple string
                let encoding = tokenizer.encode("Hello, world!", false)?;
                println!("Test encoding successful, token count: {}", encoding.get_ids().len());
            },
            Err(e) => {
                println!("Error parsing tokenizer: {}", e);
                
                // Let's try to see what the decoder field looks like
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&tokenizer_json) {
                    if let Some(decoder) = json_value.get("decoder") {
                        println!("Decoder field: {:?}", decoder);
                    }
                }
            },
        }
    } else {
        println!("\nNo tokenizer JSON found in metadata");
        
        // Check if we have tokenizer.ggml.tokens which might allow reconstruction
        if content.metadata.contains_key("tokenizer.ggml.tokens") {
            println!("Found tokenizer.ggml.tokens - attempting reconstruction...");
            if let Some(reconstructed) = try_reconstruct_tokenizer(&content.metadata) {
                match tokenizers::Tokenizer::from_bytes(reconstructed.as_bytes()) {
                    Ok(tokenizer) => {
                        println!("Tokenizer reconstructed successfully!");
                        println!("Tokenizer model type: {:?}", tokenizer.get_model().get_type());
                        println!("Vocabulary size: {}", tokenizer.get_vocab_size(true));
                    },
                    Err(e) => println!("Error parsing reconstructed tokenizer: {}", e),
                }
            } else {
                println!("Could not reconstruct tokenizer from available data");
            }
        }
    }
    
    Ok(())
}

fn find_tokenizer_json_in_metadata(md: &HashMap<String, gguf_file::Value>) -> Option<String> {
    // Known keys that might contain tokenizer JSON
    for key in [
        "tokenizer.json",
        "general.tokenizer_json",
        "qwen3.tokenizer_json",
        "llama.tokenizer_json",
        "gemma.tokenizer_json",
        "tokenizer.ggml.json",
        "tokenizer_json",
        "tokenizer",
    ] {
        if let Some(v) = md.get(key) {
            if let Ok(s) = v.to_string() { 
                return Some(s.clone()); 
            }
        }
    }
    
    // Heuristic: find any string value that looks like JSON and parses successfully
    for (_k, v) in md.iter() {
        if let Ok(s) = v.to_string() {
            let st = s.trim();
            if st.starts_with('{') && st.ends_with('}')
                && tokenizers::Tokenizer::from_bytes(st.as_bytes()).is_ok()
            {
                return Some(s.clone());
            }
        }
    }
    None
}

fn get_string_array(md: &HashMap<String, gguf_file::Value>, key: &str) -> Option<Vec<String>> {
    match md.get(key) {
        Some(gguf_file::Value::Array(vs)) => {
            let mut out: Vec<String> = Vec::with_capacity(vs.len());
            for v in vs { if let Ok(s) = v.to_string() { out.push(s.clone()); } }
            Some(out)
        }
        _ => None,
    }
}

fn try_reconstruct_tokenizer(md: &HashMap<String, gguf_file::Value>) -> Option<String> {
    // Try to reconstruct BPE tokenizer if we have both tokens and merges
    let vocab_list = get_string_array(md, "tokenizer.ggml.tokens").or_else(|| get_string_array(md, "tokenizer.vocab"))?;
    let merges_list_opt = get_string_array(md, "tokenizer.ggml.merges")
        .or_else(|| get_string_array(md, "tokenizer.ggml.bpe_merges"))
        .or_else(|| get_string_array(md, "tokenizer.merges"));
    
    if let Some(merges_list) = merges_list_opt {
        if !merges_list.is_empty() {
            let mut vocab_obj: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            for (i, tok) in vocab_list.iter().enumerate() { 
                vocab_obj.insert(tok.clone(), serde_json::json!(i as u32)); 
            }
            let json = serde_json::json!({
                "version": "1.0",
                "pre_tokenizer": { "type": "ByteLevel", "add_prefix_space": true, "trim_offsets": true },
                "decoder": { "type": "ByteLevel", "add_prefix_space": true, "trim_offsets": true },
                "model": { "type": "BPE", "vocab": vocab_obj, "merges": merges_list },
            });
            return Some(json.to_string());
        }
    }
    
    // Try to build WordLevel tokenizer from tokens only
    let mut vocab_obj: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (i, tok) in vocab_list.iter().enumerate() {
        vocab_obj.insert(tok.clone(), serde_json::json!(i as u32));
    }

    let unk = md.get("tokenizer.ggml.unknown_token")
        .and_then(|v| v.to_string().ok().map(|s| s.clone()))
        .unwrap_or_else(|| "<unk>".to_string());

    let mut root_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    root_map.insert("version".to_string(), serde_json::Value::String("1.0".to_string()));

    let mut model_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    model_map.insert("type".to_string(), serde_json::Value::String("WordLevel".to_string()));
    model_map.insert("vocab".to_string(), serde_json::Value::Object(vocab_obj));
    model_map.insert("unk_token".to_string(), serde_json::Value::String(unk.clone()));
    root_map.insert("model".to_string(), serde_json::Value::Object(model_map));

    root_map.insert("pre_tokenizer".to_string(), serde_json::json!({ "type": "Whitespace" }));
    root_map.insert("decoder".to_string(), serde_json::json!({ "type": "WordLevel" }));

    Some(serde_json::Value::Object(root_map).to_string())
}