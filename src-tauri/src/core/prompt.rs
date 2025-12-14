//! Prompt builder module for creating prompts from chat templates.
//! This module provides functionality to build prompts from chat message histories
//! using Jinja-style chat templates extracted from tokenizers.

use crate::{log_template, log_template_error};
use minijinja::{context, Environment, Value};
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Represents a chat message with role and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Prompt builder for creating prompts from chat templates
pub struct PromptBuilder {
    chat_template: Option<String>,
    bos_token: Option<String>,
}

/// Нормализует чат-шаблоны, написанные в стилистике Jinja2/Python,
/// в совместимый с MiniJinja вид. Основной кейс — методы строк вида
/// `foo.startswith("bar")`/`foo.endswith("bar")`, которые MiniJinja не
/// поддерживает напрямую. Мы переписываем их в фильтры
/// `foo|starts_with("bar")`/`foo|ends_with("bar")`.
pub fn normalize_chat_template(raw: &str) -> String {
    static STARTS_RE: OnceCell<Regex> = OnceCell::new();
    static ENDS_RE: OnceCell<Regex> = OnceCell::new();
    static SPLIT_RE: OnceCell<Regex> = OnceCell::new();

    let starts_re = STARTS_RE
        .get_or_init(|| Regex::new(r"\.startswith\s*\(").expect("valid startswith regex"));
    let ends_re = ENDS_RE
        .get_or_init(|| Regex::new(r"\.endswith\s*\(").expect("valid endswith regex"));
    let split_re = SPLIT_RE
        .get_or_init(|| Regex::new(r"\.split\s*\(").expect("valid split regex"));

    let tmp = starts_re.replace_all(raw, "|starts_with(");
    let tmp = ends_re.replace_all(&tmp, "|ends_with(");
    split_re.replace_all(&tmp, "|split(").into_owned()
}

/// Регистрирует общие фильтры для работы с шаблонами чата.
pub fn configure_chat_template_environment<'a>(env: &mut Environment<'a>) {
    env.add_filter("starts_with", |s: &str, needle: &str| s.starts_with(needle));
    env.add_filter("ends_with", |s: &str, needle: &str| s.ends_with(needle));
}

impl PromptBuilder {
    /// Create a new prompt builder with an optional chat template
    pub fn new(chat_template: Option<String>) -> Self {
        Self {
            chat_template,
            bos_token: None,
        }
    }

    /// Create with explicit bos token variable (for templates that reference it)
    pub fn with_bos(mut self, bos_token: Option<String>) -> Self {
        self.bos_token = bos_token;
        self
    }

    /// Check if a chat template is available
    pub fn has_template(&self) -> bool {
        self.chat_template
            .as_ref()
            .is_some_and(|t| !t.trim().is_empty())
    }

    /// Render a prompt from chat messages using the chat template
    pub fn render_prompt(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        let tpl = match &self.chat_template {
            Some(s) if !s.trim().is_empty() => s.clone(),
            _ => return Err("chat_template not available".into()),
        };
        let tpl = normalize_chat_template(&tpl);

        // Log input
        log_template!("render: msgs={}, tpl_len={}", messages.len(), tpl.len());

        let mut env = Environment::new();
        configure_chat_template_environment(&mut env);
        env.add_template("tpl", &tpl).map_err(|e| e.to_string())?;
        let tmpl = env.get_template("tpl").map_err(|e| e.to_string())?;

        // Create minijinja context
        let msgs_val: Vec<Value> = messages.iter().map(Value::from_serialize).collect();
        // Inject optional bos_token if provided (needed by many LLaMA/Gemma templates)
        let rendered = if let Some(bos) = &self.bos_token {
            tmpl.render(context! {
                messages => msgs_val,
                add_generation_prompt => true,
                tools => Vec::<String>::new(),
                bos_token => bos,
            })
        } else {
            tmpl.render(context! {
                messages => msgs_val,
                add_generation_prompt => true,
                tools => Vec::<String>::new(),
            })
        }
        .map_err(|e| e.to_string())?;

        log_template!(
            "render ok, prefix=<<<{}>>>",
            rendered.chars().take(120).collect::<String>()
        );
        Ok(rendered)
    }

    /// Build a prompt using fallback formatting when no template is available
    pub fn build_fallback_prompt(&self, messages: Vec<ChatMessage>) -> String {
        let mut text = String::new();

        // Process each message in the history
        for m in messages {
            if m.role == "user" {
                // For user messages, strip any special command prefixes but keep content
                let payload = m.content.trim();
                text += &format!("{}{}\n", "user\n", payload);
            } else {
                text += &format!("{}{}\n", "assistant\n", m.content.trim());
            }
        }

        // Open assistant for current step response
        text += "assistant\n";
        text
    }

    /// Build a prompt with support for special control commands
    pub fn build_prompt_with_control(
        &self,
        messages: Vec<ChatMessage>,
        _control: Option<&str>,
    ) -> String {
        // Try to render with template first
        if self.has_template() {
            match self.render_prompt(messages.clone()) {
                Ok(rendered) => {
                    // Backend no longer injects empty think blocks for no_think control;
                    // render logic for think blocks moved to frontend parser/renderer.
                    return rendered;
                }
                Err(e) => {
                    log_template_error!("render failed: {}", e);
                    // Fall through to fallback
                }
            }
        }

        // Fallback to custom formatting
        self.build_fallback_prompt(messages)
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_chat_template;

    #[test]
    fn rewrites_py_string_methods_to_filters() {
        let tpl = r#"{{ foo.startswith("<") }} {{ bar.endswith(">") }} {{ baz.split(", ") }}"#;
        let normalized = normalize_chat_template(tpl);

        assert!(normalized.contains(r#"foo|starts_with("<")"#));
        assert!(normalized.contains(r#"bar|ends_with(">")"#));
        assert!(normalized.contains(r#"baz|split(", ")"#));
    }

    #[test]
    fn keeps_unrelated_content_intact() {
        let tpl =
            r#"{% for message in messages %}{{ message.role }}: {{ message.content }}{% endfor %}"#;
        assert_eq!(normalize_chat_template(tpl), tpl);
    }
}
