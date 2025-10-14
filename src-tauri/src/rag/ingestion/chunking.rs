use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentKind {
    Pdf,
    PlainText,
    Docx,
    Markdown,
    Code,
    Unknown,
}

impl DocumentKind {
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
        {
            Some(ext) if ext == "pdf" => Some(DocumentKind::Pdf),
            Some(ext) if ext == "txt" => Some(DocumentKind::PlainText),
            Some(ext) if ext == "docx" => Some(DocumentKind::Docx),
            Some(ext) if ext == "md" || ext == "markdown" => Some(DocumentKind::Markdown),
            Some(ext)
                if matches!(
                    ext.as_str(),
                    "rs" | "py"
                        | "js"
                        | "ts"
                        | "jsx"
                        | "tsx"
                        | "java"
                        | "go"
                        | "cpp"
                        | "c"
                        | "cs"
                        | "php"
                        | "rb"
                        | "swift"
                ) =>
            {
                Some(DocumentKind::Code)
            }
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentKind::Pdf => "pdf",
            DocumentKind::PlainText => "plain-text",
            DocumentKind::Docx => "docx",
            DocumentKind::Markdown => "markdown",
            DocumentKind::Code => "code",
            DocumentKind::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChunkingStrategy {
    LayoutAware,
    Semantic,
    RecursiveSyntax,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    pub min_tokens: usize,
    pub max_tokens: usize,
    pub overlap_ratio: f32,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            min_tokens: 50,
            max_tokens: 512,
            overlap_ratio: 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkCoordinate {
    pub section: usize,
    pub index: usize,
    pub start_token: usize,
    pub end_token: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: Uuid,
    pub coordinate: ChunkCoordinate,
    pub text: String,
    pub token_count: usize,
    pub metadata: HashMap<String, String>,
}

impl DocumentChunk {
    pub fn new(
        coordinate: ChunkCoordinate,
        text: String,
        token_count: usize,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            coordinate,
            text,
            token_count,
            metadata,
        }
    }
}

/// Adaptive chunker that selects a strategy based on document kind.
#[derive(Clone)]
pub struct SemanticChunker {
    config: ChunkingConfig,
    heading_regex: Regex,
    code_split_regex: Regex,
}

impl SemanticChunker {
    pub fn new(config: ChunkingConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            heading_regex: Regex::new(r"(?m)^(?P<level>#{1,6})\s+(?P<title>.+)$")?,
            code_split_regex: Regex::new(r"(?m)^(fn|pub\s+fn|class|struct|impl)\b")?,
        })
    }

    pub fn chunk(
        &self,
        doc_kind: DocumentKind,
        text: &str,
        origin_metadata: &HashMap<String, String>,
    ) -> Vec<DocumentChunk> {
        let strategy = self.select_strategy(doc_kind);
        match strategy {
            ChunkingStrategy::LayoutAware => self.chunk_layout_aware(text, origin_metadata),
            ChunkingStrategy::Semantic => self.chunk_semantic(text, origin_metadata),
            ChunkingStrategy::RecursiveSyntax => self.chunk_recursive(text, origin_metadata),
        }
    }

    fn select_strategy(&self, kind: DocumentKind) -> ChunkingStrategy {
        match kind {
            DocumentKind::Pdf | DocumentKind::Docx => ChunkingStrategy::LayoutAware,
            DocumentKind::Markdown | DocumentKind::PlainText => ChunkingStrategy::Semantic,
            DocumentKind::Code => ChunkingStrategy::RecursiveSyntax,
            DocumentKind::Unknown => ChunkingStrategy::Semantic,
        }
    }

    fn chunk_layout_aware(
        &self,
        text: &str,
        origin_metadata: &HashMap<String, String>,
    ) -> Vec<DocumentChunk> {
        let mut sections: Vec<&str> = text
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .collect();
        if sections.is_empty() {
            sections.push(text);
        }
        self.chunk_sections(
            sections.iter().map(|s| s.to_string()).collect(),
            origin_metadata,
        )
    }

    fn chunk_semantic(
        &self,
        text: &str,
        origin_metadata: &HashMap<String, String>,
    ) -> Vec<DocumentChunk> {
        let mut blocks = Vec::new();
        let mut current_block = String::new();

        for line in text.lines() {
            if self.heading_regex.is_match(line) && !current_block.trim().is_empty() {
                blocks.push(current_block.clone());
                current_block.clear();
            }
            current_block.push_str(line);
            current_block.push('\n');
        }
        if !current_block.trim().is_empty() {
            blocks.push(current_block);
        }
        if blocks.is_empty() {
            blocks.push(text.to_string());
        }
        self.chunk_sections(blocks, origin_metadata)
    }

    fn chunk_recursive(
        &self,
        text: &str,
        origin_metadata: &HashMap<String, String>,
    ) -> Vec<DocumentChunk> {
        let mut blocks = Vec::new();
        let mut current = String::new();

        for line in text.lines() {
            if self.code_split_regex.is_match(line) && !current.trim().is_empty() {
                blocks.push(current.clone());
                current.clear();
            }
            current.push_str(line);
            current.push('\n');
        }
        if !current.trim().is_empty() {
            blocks.push(current);
        }
        if blocks.is_empty() {
            blocks.push(text.to_string());
        }

        self.chunk_sections(blocks, origin_metadata)
    }

    fn chunk_sections(
        &self,
        sections: Vec<String>,
        origin_metadata: &HashMap<String, String>,
    ) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let overlap_tokens =
            ((self.config.max_tokens as f32) * self.config.overlap_ratio).round() as usize;

        let mut section_index = 0usize;
        for section in sections {
            let sentences = split_into_sentences(&section);
            let mut start_token = 0usize;
            let mut buffer: Vec<String> = Vec::new();
            let mut token_sum = 0usize;

            for sentence in sentences {
                let token_estimate = estimate_token_count(&sentence);
                if token_sum + token_estimate > self.config.max_tokens && !buffer.is_empty() {
                    let combined = buffer.join(" ");
                    let token_count = estimate_token_count(&combined);
                    let metadata = build_metadata(origin_metadata, section_index, chunks.len());
                    chunks.push(DocumentChunk::new(
                        ChunkCoordinate {
                            section: section_index,
                            index: chunks.len(),
                            start_token,
                            end_token: start_token + token_count,
                        },
                        combined,
                        token_count,
                        metadata,
                    ));

                    let mut overlap_buffer = String::new();
                    let mut overlap_tokens_acc = 0usize;
                    for sentence in buffer.iter().rev() {
                        let tokens = estimate_token_count(sentence);
                        if overlap_tokens_acc + tokens > overlap_tokens {
                            break;
                        }
                        overlap_tokens_acc += tokens;
                        if overlap_buffer.is_empty() {
                            overlap_buffer = sentence.clone();
                        } else {
                            overlap_buffer = format!("{} {}", sentence, overlap_buffer);
                        }
                    }
                    buffer = if overlap_buffer.is_empty() {
                        Vec::new()
                    } else {
                        overlap_buffer
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>()
                    };
                    token_sum = buffer
                        .iter()
                        .map(|line| estimate_token_count(line))
                        .sum::<usize>();
                    start_token += token_count.saturating_sub(overlap_tokens_acc);
                }

                buffer.push(sentence);
                token_sum += token_estimate;
            }

            if !buffer.is_empty() {
                let combined = buffer.join(" ");
                let mut token_count = estimate_token_count(&combined);
                if token_count < self.config.min_tokens
                    && !chunks.is_empty()
                    && let Some(last) = chunks.last_mut()
                {
                    last.text.push(' ');
                    last.text.push_str(&combined);
                    last.token_count += token_count;
                    last.coordinate.end_token += token_count;
                    continue;
                }

                token_count = token_count.max(self.config.min_tokens);
                let metadata = build_metadata(origin_metadata, section_index, chunks.len());
                chunks.push(DocumentChunk::new(
                    ChunkCoordinate {
                        section: section_index,
                        index: chunks.len(),
                        start_token,
                        end_token: start_token + token_count,
                    },
                    combined,
                    token_count,
                    metadata,
                ));
            }

            section_index += 1;
        }

        debug!("Chunking completed");

        chunks
    }
}

fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut buffer = String::new();

    for ch in text.chars() {
        buffer.push(ch);
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            let trimmed = buffer.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            buffer.clear();
        }
    }

    let trimmed = buffer.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_string());
    }

    sentences
}

fn estimate_token_count(text: &str) -> usize {
    let word_count = text.split_whitespace().count();
    word_count.max(1)
}

fn build_metadata(
    origin_metadata: &HashMap<String, String>,
    section: usize,
    index: usize,
) -> HashMap<String, String> {
    let mut metadata = origin_metadata.clone();
    metadata.insert("section".into(), section.to_string());
    metadata.insert("chunk_index".into(), index.to_string());
    metadata
}
