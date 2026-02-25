use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineDefinition {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub capabilities: Vec<Capability>,
    pub binary: BinarySpec,
    pub args_template: Vec<String>,
    pub health_endpoint: String,
    pub api_style: ApiStyle,
    pub env_vars: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Chat,
    Completion,
    Vision,
    ImageGeneration,
    AudioTranscription,
    AudioSpeech,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiStyle {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "openai_images")]
    OpenAIImages,
    Ollama,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinarySpec {
    pub variants: Vec<BinaryVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryVariant {
    pub os: Os,
    pub arch: Arch,
    pub gpu: Option<GpuRequirement>,
    /// Relative path to binary within install dir
    pub path: String,
    /// Download URL for this variant (zip/tar.gz archive)
    pub download_url: Option<String>,
    /// SHA256 hash for verification
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Windows,
    Macos,
    Linux,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    X64,
    Arm64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GpuRequirement {
    Cuda,
    Vulkan,
    Metal,
    Rocm,
}

impl EngineDefinition {
    /// Find the matching binary variant for the current platform
    pub fn find_variant_for_current_os(&self) -> Option<&BinaryVariant> {
        let target_os = current_os();
        let target_arch = current_arch();

        self.binary
            .variants
            .iter()
            .find(|v| v.os == target_os && v.arch == target_arch)
    }
}

pub fn current_os() -> Os {
    if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        Os::Linux
    }
}

pub fn current_arch() -> Arch {
    if cfg!(target_arch = "aarch64") {
        Arch::Arm64
    } else {
        Arch::X64
    }
}
