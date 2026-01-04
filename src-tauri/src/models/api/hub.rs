//! Загрузка моделей с HuggingFace Hub

use std::path::PathBuf;

/// Загрузчик моделей с HuggingFace Hub
pub struct HubDownloader {
    api: hf_hub::api::sync::Api,
}

impl HubDownloader {
    /// Создаёт новый загрузчик
    pub fn new() -> super::error::Result<Self> {
        let api = hf_hub::api::sync::Api::new()?;
        Ok(Self { api })
    }

    /// Загружает файл из репозитория
    pub fn get_file(
        &self,
        repo_id: &str,
        filename: &str,
        revision: Option<&str>,
    ) -> super::error::Result<PathBuf> {
        let repo = self.api.repo(hf_hub::Repo::with_revision(
            repo_id.to_string(),
            hf_hub::RepoType::Model,
            revision.unwrap_or("main").to_string(),
        ));

        let path = repo.get(filename)?;
        Ok(path)
    }

    /// Загружает tokenizer.json
    pub fn get_tokenizer(
        &self,
        repo_id: &str,
        revision: Option<&str>,
    ) -> super::error::Result<PathBuf> {
        self.get_file(repo_id, "tokenizer.json", revision)
    }

    /// Загружает config.json
    pub fn get_config(
        &self,
        repo_id: &str,
        revision: Option<&str>,
    ) -> super::error::Result<PathBuf> {
        self.get_file(repo_id, "config.json", revision)
    }

    /// Загружает SafeTensors файлы модели
    pub fn get_safetensors_files(
        &self,
        repo_id: &str,
        revision: Option<&str>,
    ) -> super::error::Result<Vec<PathBuf>> {
        let repo = self.api.repo(hf_hub::Repo::with_revision(
            repo_id.to_string(),
            hf_hub::RepoType::Model,
            revision.unwrap_or("main").to_string(),
        ));

        // Пробуем загрузить index файл
        match repo.get("model.safetensors.index.json") {
            Ok(index_path) => {
                // Читаем index и загружаем все части
                let index_content = std::fs::read_to_string(&index_path)?;
                let index: serde_json::Value = serde_json::from_str(&index_content)?;

                let mut files = std::collections::HashSet::new();
                if let Some(weight_map) = index.get("weight_map").and_then(|v| v.as_object()) {
                    for filename in weight_map.values() {
                        if let Some(f) = filename.as_str() {
                            files.insert(f.to_string());
                        }
                    }
                }

                let mut paths = Vec::new();
                for filename in files {
                    paths.push(repo.get(&filename)?);
                }
                Ok(paths)
            }
            Err(_) => {
                // Пробуем загрузить один файл
                let path = repo.get("model.safetensors")?;
                Ok(vec![path])
            }
        }
    }

    /// Загружает GGUF файл модели
    pub fn get_gguf_file(
        &self,
        repo_id: &str,
        filename: &str,
        revision: Option<&str>,
    ) -> super::error::Result<PathBuf> {
        self.get_file(repo_id, filename, revision)
    }
}

impl Default for HubDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create HubDownloader")
    }
}

/// Информация о модели из config.json
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModelConfigJson {
    pub model_type: Option<String>,
    pub vocab_size: Option<usize>,
    pub hidden_size: Option<usize>,
    pub num_hidden_layers: Option<usize>,
    pub num_attention_heads: Option<usize>,
    pub max_position_embeddings: Option<usize>,
    #[serde(default)]
    pub torch_dtype: Option<String>,
}

impl ModelConfigJson {
    /// Загружает конфиг из файла
    pub fn from_file(path: &std::path::Path) -> super::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }
}
