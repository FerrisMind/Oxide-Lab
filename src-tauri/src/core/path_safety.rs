use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

fn parse_absolute_path(raw: &str) -> Result<PathBuf, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err("Path must be absolute".to_string());
    }
    Ok(path)
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(path)
        .map_err(|e| format!("Failed to canonicalize path {}: {e}", path.display()))
}

fn resolve_with_existing_ancestor(path: &Path) -> Result<PathBuf, String> {
    let mut ancestor = path;
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or_else(|| {
            format!(
                "Failed to resolve path {}: no existing ancestor",
                path.display()
            )
        })?;
    }
    let canonical_ancestor = canonicalize_existing(ancestor)?;
    let suffix = path.strip_prefix(ancestor).map_err(|e| {
        format!(
            "Failed to resolve path {} relative to ancestor {}: {e}",
            path.display(),
            ancestor.display()
        )
    })?;
    Ok(canonical_ancestor.join(suffix))
}

pub fn app_allowed_roots(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();

    let app_data = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {e}"))?;
    fs::create_dir_all(&app_data).map_err(|e| {
        format!(
            "Failed to create app data directory {}: {e}",
            app_data.display()
        )
    })?;
    roots.push(canonicalize_existing(&app_data).unwrap_or(app_data));

    if let Ok(download) = app.path().download_dir()
        && download.exists()
    {
        roots.push(canonicalize_existing(&download).unwrap_or(download));
    }

    if let Ok(document) = app.path().document_dir()
        && document.exists()
    {
        roots.push(canonicalize_existing(&document).unwrap_or(document));
    }

    roots.sort();
    roots.dedup();
    Ok(roots)
}

fn ensure_path_in_allowed_roots(path: &Path, roots: &[PathBuf]) -> Result<PathBuf, String> {
    if roots.iter().any(|root| path.starts_with(root)) {
        return Ok(path.to_path_buf());
    }
    Err(format!(
        "Path {} is outside allowed directories",
        path.display()
    ))
}

pub fn ensure_scoped_existing_path(app: &AppHandle, raw: &str) -> Result<PathBuf, String> {
    let raw_path = parse_absolute_path(raw)?;
    if !raw_path.exists() {
        return Err(format!("Path does not exist: {}", raw_path.display()));
    }
    let canonical = canonicalize_existing(&raw_path)?;
    let roots = app_allowed_roots(app)?;
    ensure_path_in_allowed_roots(&canonical, &roots)
}

pub fn ensure_scoped_existing_pathbuf(app: &AppHandle, path: &Path) -> Result<PathBuf, String> {
    let canonical = canonicalize_existing(path)?;
    let roots = app_allowed_roots(app)?;
    ensure_path_in_allowed_roots(&canonical, &roots)
}

pub fn ensure_scoped_path(
    app: &AppHandle,
    raw: &str,
    require_exists: bool,
) -> Result<PathBuf, String> {
    let raw_path = parse_absolute_path(raw)?;
    let resolved = if raw_path.exists() || require_exists {
        if !raw_path.exists() {
            return Err(format!("Path does not exist: {}", raw_path.display()));
        }
        canonicalize_existing(&raw_path)?
    } else {
        resolve_with_existing_ancestor(&raw_path)?
    };
    let roots = app_allowed_roots(app)?;
    ensure_path_in_allowed_roots(&resolved, &roots)
}

pub fn sanitize_file_name(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("Filename cannot be empty".to_string());
    }
    let path = Path::new(value);
    let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
        return Err("Invalid filename".to_string());
    };
    if file_name != value || file_name == "." || file_name == ".." {
        return Err("Filename must not contain path components".to_string());
    }
    if file_name.contains('/') || file_name.contains('\\') {
        return Err("Filename must not contain path separators".to_string());
    }
    Ok(file_name.to_string())
}

pub fn sanitize_path_component(raw: &str, label: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!("{label} cannot be empty"));
    }
    if value == "." || value == ".." {
        return Err(format!("{label} contains invalid value"));
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Ok(value.to_string());
    }
    Err(format!(
        "{label} contains unsupported characters (allowed: a-z A-Z 0-9 _ - .)"
    ))
}
