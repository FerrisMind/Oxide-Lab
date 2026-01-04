fn main() {
    // Add path to stdc++.lib stub for Windows MSVC compatibility
    // This is needed because candle-kernels MOE requires libstdc++ which doesn't exist on MSVC
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rustc-link-search=native={}", manifest_dir);

        // Add cuDNN library path from environment variable or auto-detect
        println!("cargo:rerun-if-env-changed=CUDNN_PATH");
        if let Ok(cudnn_path) = std::env::var("CUDNN_PATH") {
            // Check if CUDNN_PATH already points to directory with cudnn.lib
            let cudnn_lib = format!("{}\\cudnn.lib", cudnn_path);
            if std::path::Path::new(&cudnn_lib).exists() {
                // CUDNN_PATH already points to lib directory
                println!("cargo:rustc-link-search=native={}", cudnn_path);
            } else {
                // Try both new (12.x/x64) and old (x64) directory structures
                let new_path = format!("{}\\lib\\12.9\\x64", cudnn_path);
                let old_path = format!("{}\\lib\\x64", cudnn_path);
                if std::path::Path::new(&new_path).exists() {
                    println!("cargo:rustc-link-search=native={}", new_path);
                } else if std::path::Path::new(&old_path).exists() {
                    println!("cargo:rustc-link-search=native={}", old_path);
                } else {
                    // Fallback: use CUDNN_PATH as-is
                    println!("cargo:rustc-link-search=native={}", cudnn_path);
                }
            }
        } else {
            // Try common cuDNN installation paths (new structure with CUDA version subdirs)
            let possible_paths = [
                // cuDNN 9.x+ uses lib/{cuda_version}/x64 structure
                "C:\\Program Files\\NVIDIA\\CUDNN\\v9.17\\lib\\12.9\\x64",
                "C:\\Program Files\\NVIDIA\\CUDNN\\v9.6\\lib\\12.6\\x64",
                "C:\\Program Files\\NVIDIA\\CUDNN\\v9.5\\lib\\12.6\\x64",
                // Older cuDNN uses lib/x64 structure
                "C:\\Program Files\\NVIDIA\\CUDNN\\v9.17\\lib\\x64",
                "C:\\Program Files\\NVIDIA\\CUDNN\\v9.0\\lib\\x64",
                "C:\\Program Files\\NVIDIA\\CUDNN\\v8.9\\lib\\x64",
            ];
            for path in possible_paths {
                if std::path::Path::new(path).exists() {
                    println!("cargo:rustc-link-search=native={}", path);
                    break;
                }
            }
        }
    }

    tauri_build::build()
}
