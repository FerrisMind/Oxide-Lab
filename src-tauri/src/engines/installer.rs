//! Engine installer - downloads and extracts engine binaries.
//!
//! Engines are downloaded as zip archives, extracted into
//! `{app_data}/engines/{engine_id}/`, and the binary path is
//! resolved relative to that directory.

use log::{error, info};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};

use super::definition::EngineDefinition;

/// Status of an engine installation
#[derive(Debug, Clone, serde::Serialize)]
pub struct EngineInstallInfo {
    pub engine_id: String,
    pub name: String,
    pub description: Option<String>,
    pub installed: bool,
    pub binary_path: Option<String>,
    pub download_url: Option<String>,
    pub capabilities: Vec<String>,
}

/// Progress event emitted during installation
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallProgress {
    pub engine_id: String,
    pub stage: String, // "downloading" | "extracting" | "done" | "error"
    pub progress: f64, // 0.0 - 100.0
    pub message: String,
}

/// Get the engines install directory
pub fn engines_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    let engines_path = dir.join("engines");
    fs::create_dir_all(&engines_path).map_err(|e| format!("Failed to create engines dir: {e}"))?;
    Ok(engines_path)
}

/// Check if an engine is installed (binary exists)
pub fn is_engine_installed(app: &AppHandle, def: &EngineDefinition) -> bool {
    if let Ok(path) = resolve_engine_binary(app, def) {
        path.exists()
    } else {
        false
    }
}

/// Resolve the full binary path for an engine
pub fn resolve_engine_binary(app: &AppHandle, def: &EngineDefinition) -> Result<PathBuf, String> {
    validate_engine_id(&def.id)?;
    let variant = def
        .find_variant_for_current_os()
        .ok_or_else(|| "No binary variant for current OS".to_string())?;

    let dir = engines_dir(app)?;
    let engine_dir = dir.join(&def.id);
    Ok(engine_dir.join(&variant.path))
}

/// Install an engine by downloading and extracting it
pub async fn install_engine(app: AppHandle, def: &EngineDefinition) -> Result<PathBuf, String> {
    validate_engine_id(&def.id)?;
    let variant = def
        .find_variant_for_current_os()
        .ok_or_else(|| "No binary variant for current OS".to_string())?;

    let download_url = variant
        .download_url
        .as_ref()
        .ok_or_else(|| "No download URL for this engine variant".to_string())?;

    let dir = engines_dir(&app)?;
    let engine_dir = dir.join(&def.id);
    fs::create_dir_all(&engine_dir).map_err(|e| format!("Failed to create engine dir: {e}"))?;

    // Emit: downloading
    emit_progress(&app, &def.id, "downloading", 0.0, "Starting download...");

    info!("Downloading engine {} from {}", def.id, download_url);

    // Download the archive
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let archive_path = engine_dir.join("_download.zip");

    // Stream download to file with progress
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::File::create(&archive_path)
        .await
        .map_err(|e| format!("Failed to create archive file: {e}"))?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| format!("Download stream error: {e}"))?;
        file.write_all(&bytes)
            .await
            .map_err(|e| format!("Write error: {e}"))?;
        downloaded += bytes.len() as u64;

        // Throttle progress emission
        if last_emit.elapsed() > std::time::Duration::from_millis(250) {
            let pct = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 80.0 // 0-80% for download
            } else {
                50.0
            };
            emit_progress(
                &app,
                &def.id,
                "downloading",
                pct,
                &format!(
                    "{:.1} MB / {:.1} MB",
                    downloaded as f64 / 1_048_576.0,
                    total_size as f64 / 1_048_576.0
                ),
            );
            last_emit = std::time::Instant::now();
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("Flush error: {e}"))?;
    drop(file);

    if let Some(expected_sha256) = variant.sha256.as_deref() {
        let actual = compute_sha256_hex(&archive_path)?;
        let expected = expected_sha256.trim().to_ascii_lowercase();
        if actual != expected {
            let _ = fs::remove_file(&archive_path);
            return Err(format!(
                "SHA256 mismatch for engine archive. expected={expected}, actual={actual}"
            ));
        }
    }

    info!("Downloaded {} bytes for engine {}", downloaded, def.id);

    // Emit: extracting
    emit_progress(&app, &def.id, "extracting", 85.0, "Extracting archive...");

    // Extract zip
    extract_zip(&archive_path, &engine_dir)?;

    // Clean up archive
    let _ = fs::remove_file(&archive_path);

    // Verify binary exists
    let binary_path = engine_dir.join(&variant.path);
    if !binary_path.exists() {
        error!(
            "Binary not found after extraction: {}",
            binary_path.display()
        );
        return Err(format!(
            "Binary not found after extraction: {}. Check that the archive structure matches the path in engines.json.",
            variant.path
        ));
    }

    // Make binary executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&binary_path, fs::Permissions::from_mode(0o755));
    }

    emit_progress(&app, &def.id, "done", 100.0, "Installation complete");

    info!("Engine {} installed at {}", def.id, binary_path.display());
    Ok(binary_path)
}

/// Uninstall an engine by removing its directory
pub fn uninstall_engine(app: &AppHandle, engine_id: &str) -> Result<(), String> {
    validate_engine_id(engine_id)?;
    let dir = engines_dir(app)?;
    let canonical_root = fs::canonicalize(&dir)
        .map_err(|e| format!("Failed to canonicalize engines dir {}: {e}", dir.display()))?;
    let engine_dir = canonical_root.join(engine_id);
    if engine_dir.exists() {
        let canonical_target = fs::canonicalize(&engine_dir).map_err(|e| {
            format!(
                "Failed to canonicalize engine directory {}: {e}",
                engine_dir.display()
            )
        })?;
        if !canonical_target.starts_with(&canonical_root) {
            return Err(format!(
                "Refusing to uninstall path outside engine root: {}",
                canonical_target.display()
            ));
        }
        fs::remove_dir_all(&canonical_target)
            .map_err(|e| format!("Failed to remove engine dir: {e}"))?;
        info!("Engine {} uninstalled", engine_id);
    }
    Ok(())
}

fn emit_progress(app: &AppHandle, engine_id: &str, stage: &str, progress: f64, message: &str) {
    let _ = app.emit(
        "engine_install_progress",
        InstallProgress {
            engine_id: engine_id.to_string(),
            stage: stage.to_string(),
            progress,
            message: message.to_string(),
        },
    );
}

fn extract_zip(archive_path: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(archive_path).map_err(|e| format!("Failed to open archive: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {e}"))?;

        let name = entry.name().to_string();
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| format!("Unsafe zip entry path: {name}"))?
            .to_path_buf();
        let out_path = dest.join(&enclosed);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Failed to create dir {}: {e}", name))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {e}"))?;
            }
            let mut outfile = fs::File::create(&out_path)
                .map_err(|e| format!("Failed to create file {}: {e}", name))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Failed to extract {}: {e}", name))?;
        }
    }

    Ok(())
}

fn validate_engine_id(engine_id: &str) -> Result<(), String> {
    if engine_id.is_empty() {
        return Err("Engine id cannot be empty".to_string());
    }
    if engine_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
    {
        return Ok(());
    }
    Err("Engine id contains unsupported characters".to_string())
}

fn compute_sha256_hex(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("Failed to open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
