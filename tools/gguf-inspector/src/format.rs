use candle::quantized::gguf_file;

pub fn readable_value(v: &gguf_file::Value) -> String {
    // Prefer the library-provided string representation when available
    if let Ok(s) = v.to_string() {
        return s.to_string();
    }

    // If it's an array, try to produce a compact textual representation.
    // Special-case: arrays of U8 should be printed as hex (blob-like).
    match v {
        gguf_file::Value::Array(arr) => {
            // If array of bytes (U8) -> hex preview or length
            if !arr.is_empty() && arr.iter().all(|el| matches!(el, gguf_file::Value::U8(_))) {
                let bytes_len = arr.len();
                if bytes_len <= 64 {
                    let hex: String = arr.iter().map(|el| {
                        if let gguf_file::Value::U8(b) = el { format!("{:02x}", b) } else { String::new() }
                    }).collect::<Vec<_>>().join("");
                    return hex;
                } else {
                    return format!("Blob(len={})", bytes_len);
                }
            }

            // Map small arrays to joined string, large arrays -> debug
            if arr.len() <= 64 {
                let parts: Vec<String> = arr.iter().map(|el| format!("{:?}", el)).collect();
                return format!("Array([{}])", parts.join(", "));
            } else {
                return format!("Array(len={})", arr.len());
            }
        }
        _ => format!("{:?}", v),
    }
}


