//! Prompt builder module for creating prompts from chat templates.
//! This module provides functionality to build prompts from chat message histories
//! using Jinja-style chat templates extracted from tokenizers.

use minijinja::{Environment, Value, context};
use serde::{Deserialize, Serialize};
use crate::{log_template, log_template_error};

/// Represents a chat message with role and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Prompt builder for creating prompts from chat templates
pub struct PromptBuilder {
    chat_template: Option<String>,
}

impl PromptBuilder {
    /// Create a new prompt builder with an optional chat template
    pub fn new(chat_template: Option<String>) -> Self {
        Self { chat_template }
    }

    /// Check if a chat template is available
    pub fn has_template(&self) -> bool {
        self.chat_template.as_ref().is_some_and(|t| !t.trim().is_empty())
    }

    /// Render a prompt from chat messages using the chat template
    pub fn render_prompt(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        let tpl = match &self.chat_template {
            Some(s) if !s.trim().is_empty() => s.clone(),
            _ => return Err("chat_template not available".into())
        };

        // Log input
        log_template!("render: msgs={}, tpl_len={}", messages.len(), tpl.len());
        
        let mut env = Environment::new();
        env.add_template("tpl", &tpl).map_err(|e| e.to_string())?;
        let tmpl = env.get_template("tpl").map_err(|e| e.to_string())?;
        
        // Create minijinja context
        let msgs_val: Vec<Value> = messages.iter().map(Value::from_serialize).collect();
        let rendered = tmpl
            .render(context! { 
                messages => msgs_val, 
                add_generation_prompt => true, 
                tools => Vec::<String>::new() 
            })
            .map_err(|e| e.to_string())?;
            
        log_template!("render ok, prefix=<<<{}>>>", rendered.chars().take(120).collect::<String>());
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
                text += &format!("{}{}\n", "\u{1f60a}user\n", payload);
            } else {
                text += &format!("{}{}\n", "\u{1f60a}assistant\n", m.content.trim());
            }
        }

        // Open assistant for current step response
        text += "\u{1f60a}assistant\n";
        text
    }

    /// Build a prompt with support for special control commands
    pub fn build_prompt_with_control(&self, messages: Vec<ChatMessage>, control: Option<&str>) -> String {
        // Try to render with template first
        if self.has_template() {
            match self.render_prompt(messages.clone()) {
                Ok(mut rendered) => {
                    // If no_think control is active and template doesn't contain think tags,
                    // add empty think block
                    if control == Some("no_think") && !rendered.contains("\u{1f60a}") {
                        rendered += "\n\u{1f60a}\n\n\u{1f60a}\n\n";
                    }
                    return rendered;
                },
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