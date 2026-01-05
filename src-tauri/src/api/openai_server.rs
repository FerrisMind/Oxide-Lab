//! OpenAI-compatible HTTP API server.
//!
//! Provides `/v1/chat/completions` and `/v1/models` endpoints for compatibility
//! with OpenAI clients (Cursor, Continue, Open WebUI, etc.).

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::core::state::SharedState;
use crate::core::types::{ChatMessage, GenerateRequest};
use crate::generate::emit::{EmissionBackend, GenerationEvent};
use crate::generate::stream::generate_stream_with_backend;
use crate::generate::tool_call_parser::{Tool, ToolCall};

// ============================================================================
// OpenAI API Types
// ============================================================================

pub const OPENAI_PORT: u16 = 11434;

#[derive(Serialize)]
pub struct ServerConfig {
    pub port: u16,
    pub running: bool,
}

#[tauri::command]
pub fn get_server_config() -> ServerConfig {
    ServerConfig {
        port: OPENAI_PORT,
        running: true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub tools: Option<Vec<Tool>>,
    /// OpenAI frequency_penalty [-2.0, 2.0] - maps to repeat_penalty
    #[serde(default)]
    pub frequency_penalty: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Array(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: Option<String>,
    // image_url field exists in spec, but we ignore it for text-only processing
}

impl From<OpenAIMessage> for ChatMessage {
    fn from(msg: OpenAIMessage) -> Self {
        let content = match msg.content {
            MessageContent::Text(t) => t,
            MessageContent::Array(parts) => parts
                .into_iter()
                .filter(|p| p.part_type == "text")
                .filter_map(|p| p.text)
                .collect::<Vec<_>>()
                .join("\n"),
        };

        ChatMessage {
            role: msg.role,
            content,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletion {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize)]
pub struct Choice {
    pub index: usize,
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: OpenAIFunction,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub arguments: String,
}

impl From<ToolCall> for OpenAIToolCall {
    fn from(tc: ToolCall) -> Self {
        OpenAIToolCall {
            id: tc.id,
            call_type: "function".to_string(),
            function: OpenAIFunction {
                name: tc.function.name,
                arguments: serde_json::to_string(&tc.function.arguments).unwrap_or_default(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChunkChoice {
    pub index: usize,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: ApiError,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String, // Simplified: supports string, not array options for now
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionChoice {
    pub text: String,
    pub index: usize,
    pub finish_reason: Option<String>,
}

// ============================================================================
// Backend Implementation
// ============================================================================

pub struct OpenAIBackend {
    tx: tokio::sync::mpsc::UnboundedSender<GenerationEvent>,
}

impl OpenAIBackend {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<GenerationEvent>) -> Self {
        Self { tx }
    }
}

impl EmissionBackend for OpenAIBackend {
    fn emit(&self, event: GenerationEvent) {
        let _ = self.tx.send(event);
    }
}

// ============================================================================
// Server State
// ============================================================================

pub struct OpenAIServerState {
    pub model_state: SharedState,
    pub shutdown_tx: broadcast::Sender<()>,
}

// ============================================================================
// Handlers
// ============================================================================

async fn models_handler(
    State(state): State<Arc<OpenAIServerState>>,
) -> Result<Json<ModelList>, (StatusCode, Json<ErrorResponse>)> {
    let guard = state.model_state.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ApiError {
                    message: "Failed to lock model state".into(),
                    error_type: "server_error".into(),
                    code: None,
                },
            }),
        )
    })?;

    let mut models = if guard.scheduler.has_model() {
        let model_id = guard
            .scheduler
            .get_model_id()
            .unwrap_or_else(|| "loaded-model".to_string());

        vec![Model {
            id: model_id,
            object: "model".to_string(),
            created: now_unix(),
            owned_by: "oxide-lab".to_string(),
        }]
    } else {
        vec![]
    };

    // Always add generic aliases to satisfy clients checking for specific models
    models.push(Model {
        id: "local-model".to_string(),
        object: "model".to_string(),
        created: now_unix(),
        owned_by: "oxide-lab".to_string(),
    });
    models.push(Model {
        id: "gpt-3.5-turbo".to_string(),
        object: "model".to_string(),
        created: now_unix(),
        owned_by: "openai".to_string(),
    });
    models.push(Model {
        id: "gpt-4".to_string(),
        object: "model".to_string(),
        created: now_unix(),
        owned_by: "openai".to_string(),
    });

    Ok(Json(ModelList {
        object: "list".to_string(),
        data: models,
    }))
}

async fn chat_completions_handler(
    State(state): State<Arc<OpenAIServerState>>,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    if req.stream {
        // For streaming, return SSE
        let stream = create_completion_stream(state, req).await?;
        Ok(Sse::new(stream)
            .keep_alive(KeepAlive::default())
            .into_response())
    } else {
        // Non-streaming response
        let completion = create_completion(state, req).await?;
        Ok(Json(completion).into_response())
    }
}

async fn create_completion(
    state: Arc<OpenAIServerState>,
    req: ChatCompletionRequest,
) -> Result<ChatCompletion, (StatusCode, Json<ErrorResponse>)> {
    // 1. Check model loaded
    {
        let guard = state
            .model_state
            .lock()
            .map_err(|_| server_error("Lock failed"))?;

        if !guard.scheduler.has_model() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ApiError {
                        message: "No model loaded".into(),
                        error_type: "invalid_request_error".into(),
                        code: None,
                    },
                }),
            ));
        }
    } // guard dropped

    // drop(guard); // removed as we used scope

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let backend = Box::new(OpenAIBackend::new(tx));
    let id = format!("chatcmpl-{}", generate_id());
    let model_name = req.model.clone();

    // OpenAI frequency_penalty [-2,2] → repeat_penalty [0.5, 2.0]
    // frequency_penalty=0 → repeat_penalty=1.0 (neutral)
    // frequency_penalty>0 → stronger penalty
    let repeat_penalty = req.frequency_penalty.map(|fp| {
        // Clamp and convert: fp ∈ [-2, 2] → rp ∈ [0.5, 1.5] (approx)
        // Simple linear: rp = 1.0 + fp * 0.25, clamped to [0.5, 2.0]
        ((1.0 + fp * 0.25).clamp(0.5, 2.0)) as f32
    });

    // Prepare GenerateRequest
    let gen_req = GenerateRequest {
        prompt: String::new(),
        messages: Some(req.messages.into_iter().map(ChatMessage::from).collect()),
        temperature: req.temperature,
        top_p: req.top_p,
        max_new_tokens: req.max_tokens,
        tools: req.tools,
        // defaults
        top_k: None,
        min_p: None,
        repeat_penalty,
        repeat_last_n: 64, // Default
        seed: None,
        use_custom_params: true,
        tracing: None,
        verbose_prompt: None,
        split_prompt: None,
        attachments: None,
        edit_index: None,
        format: None,
    };

    let state_clone = state.model_state.clone();

    // Spawn generation in blocking thread
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = generate_stream_with_backend(state_clone, gen_req, backend) {
            log::error!("Generation failed: {}", e);
        }
    });

    let mut full_content = String::new();
    let mut tool_calls = Vec::new();
    let mut usage = Usage {
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
    };
    let finish_reason = Some("stop".to_string());

    while let Some(event) = rx.recv().await {
        match event {
            GenerationEvent::Token(t) => full_content.push_str(&t),
            GenerationEvent::Message(msg) => full_content.push_str(&msg.content),
            GenerationEvent::ToolCall(tc) => tool_calls.push(tc.into()),
            GenerationEvent::Metrics(m) => {
                usage.prompt_tokens = m.prompt_tokens;
                usage.completion_tokens = m.generated_tokens;
                usage.total_tokens = m.prompt_tokens + m.generated_tokens;
            }
            GenerationEvent::Done => {}
            _ => {}
        }
    }

    Ok(ChatCompletion {
        id,
        object: "chat.completion".to_string(),
        created: now_unix(),
        model: model_name,
        choices: vec![Choice {
            index: 0,
            message: ResponseMessage {
                role: "assistant".to_string(),
                content: full_content,
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                },
            },
            finish_reason,
        }],
        usage,
    })
}

async fn create_completion_stream(
    state: Arc<OpenAIServerState>,
    req: ChatCompletionRequest,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, (StatusCode, Json<ErrorResponse>)> {
    // Check if model is loaded - scope the guard to ensure drop
    {
        let guard = state
            .model_state
            .lock()
            .map_err(|_| server_error("Lock failed"))?;

        if !guard.scheduler.has_model() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ApiError {
                        message: "No model loaded".into(),
                        error_type: "invalid_request_error".into(),
                        code: None,
                    },
                }),
            ));
        }
    } // guard dropped here

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let backend = Box::new(OpenAIBackend::new(tx));
    let id = format!("chatcmpl-{}", generate_id());
    let model_id = req.model.clone();

    // OpenAI frequency_penalty → repeat_penalty conversion
    let repeat_penalty = req
        .frequency_penalty
        .map(|fp| ((1.0 + fp * 0.25).clamp(0.5, 2.0)) as f32);

    // Prepare GenerateRequest
    let gen_req = GenerateRequest {
        prompt: String::new(),
        messages: Some(req.messages.into_iter().map(ChatMessage::from).collect()),
        temperature: req.temperature,
        top_p: req.top_p,
        max_new_tokens: req.max_tokens,
        tools: req.tools,
        // defaults
        top_k: None,
        min_p: None,
        repeat_penalty,
        repeat_last_n: 64, // Default
        seed: None,
        use_custom_params: true,
        tracing: None,
        verbose_prompt: None,
        split_prompt: None,
        attachments: None,
        edit_index: None,
        format: None,
    };

    let state_clone = state.model_state.clone();

    // Spawn generation in blocking thread
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = generate_stream_with_backend(state_clone, gen_req, backend) {
            log::error!("Generation failed: {}", e);
        }
    });

    let stream = stream::unfold(
        (rx, id, model_id, false, false), // Added done_sent state
        move |(mut rx, id, model_id, mut finished, done_sent)| async move {
            if done_sent {
                return None;
            }

            if finished {
                // Send [DONE] and stop
                return Some((
                    Ok(Event::default().data("[DONE]")),
                    (rx, id, model_id, true, true),
                ));
            }

            match rx.recv().await {
                Some(event) => {
                    let chunk = match event {
                        GenerationEvent::Start => ChatCompletionChunk {
                            id: id.clone(),
                            object: "chat.completion.chunk".to_string(),
                            created: now_unix(),
                            model: model_id.clone(),
                            choices: vec![ChunkChoice {
                                index: 0,
                                delta: Delta {
                                    role: Some("assistant".to_string()),
                                    content: None,
                                    tool_calls: None,
                                },
                                finish_reason: None,
                            }],
                        },
                        GenerationEvent::Token(t) => ChatCompletionChunk {
                            id: id.clone(),
                            object: "chat.completion.chunk".to_string(),
                            created: now_unix(),
                            model: model_id.clone(),
                            choices: vec![ChunkChoice {
                                index: 0,
                                delta: Delta {
                                    role: None,
                                    content: Some(t),
                                    tool_calls: None,
                                },
                                finish_reason: None,
                            }],
                        },
                        GenerationEvent::Message(msg) => {
                            let content = if msg.content.is_empty() {
                                None
                            } else {
                                Some(msg.content)
                            };

                            ChatCompletionChunk {
                                id: id.clone(),
                                object: "chat.completion.chunk".to_string(),
                                created: now_unix(),
                                model: model_id.clone(),
                                choices: vec![ChunkChoice {
                                    index: 0,
                                    delta: Delta {
                                        role: None,
                                        content,
                                        tool_calls: None,
                                    },
                                    finish_reason: None,
                                }],
                            }
                        }
                        GenerationEvent::ToolCall(tc) => {
                            let tc_openai: OpenAIToolCall = tc.into();
                            ChatCompletionChunk {
                                id: id.clone(),
                                object: "chat.completion.chunk".to_string(),
                                created: now_unix(),
                                model: model_id.clone(),
                                choices: vec![ChunkChoice {
                                    index: 0,
                                    delta: Delta {
                                        role: None,
                                        content: None,
                                        tool_calls: Some(vec![tc_openai]),
                                    },
                                    finish_reason: None,
                                }],
                            }
                        }
                        GenerationEvent::Metrics(_) | GenerationEvent::PromptDump(_) => {
                            ChatCompletionChunk {
                                id: id.clone(),
                                object: "chat.completion.chunk".to_string(),
                                created: now_unix(),
                                model: model_id.clone(),
                                choices: vec![ChunkChoice {
                                    index: 0,
                                    delta: Delta::default(),
                                    finish_reason: None,
                                }],
                            }
                        }
                        GenerationEvent::Done => {
                            finished = true;
                            ChatCompletionChunk {
                                id: id.clone(),
                                object: "chat.completion.chunk".to_string(),
                                created: now_unix(),
                                model: model_id.clone(),
                                choices: vec![ChunkChoice {
                                    index: 0,
                                    delta: Delta::default(),
                                    finish_reason: Some("stop".to_string()),
                                }],
                            }
                        }
                    };

                    let data = serde_json::to_string(&chunk).unwrap_or_default();
                    Some((
                        Ok(Event::default().data(data)),
                        (rx, id, model_id, finished, done_sent),
                    ))
                }
                None => None,
            }
        },
    );

    Ok(stream)
}

async fn completions_handler(
    State(state): State<Arc<OpenAIServerState>>,
    Json(req): Json<CompletionRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    if req.stream {
        let stream = create_legacy_completion_stream(state, req).await?;
        Ok(Sse::new(stream)
            .keep_alive(KeepAlive::default())
            .into_response())
    } else {
        let completion = create_legacy_completion(state, req).await?;
        Ok(Json(completion).into_response())
    }
}

async fn create_legacy_completion(
    state: Arc<OpenAIServerState>,
    req: CompletionRequest,
) -> Result<CompletionResponse, (StatusCode, Json<ErrorResponse>)> {
    // 1. Check model loaded
    {
        let guard = state
            .model_state
            .lock()
            .map_err(|_| server_error("Lock failed"))?;
        if !guard.scheduler.has_model() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ApiError {
                        message: "No model loaded".into(),
                        error_type: "invalid_request_error".into(),
                        code: None,
                    },
                }),
            ));
        }
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let backend = Box::new(OpenAIBackend::new(tx));
    let id = format!("cmpl-{}", generate_id());
    let model_name = req.model.clone();

    let gen_req = GenerateRequest {
        prompt: req.prompt.clone(),
        messages: None,
        temperature: req.temperature,
        top_p: req.top_p,
        max_new_tokens: req.max_tokens,
        tools: None,
        top_k: None,
        min_p: None,
        repeat_penalty: None,
        repeat_last_n: 64,
        seed: None,
        use_custom_params: true,
        tracing: None,
        verbose_prompt: None,
        split_prompt: None,
        attachments: None,
        edit_index: None,
        format: None,
    };

    let state_clone = state.model_state.clone();

    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = generate_stream_with_backend(state_clone, gen_req, backend) {
            log::error!("Generation failed: {}", e);
        }
    });

    let mut full_text = String::new();
    let mut usage = Usage {
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
    };

    while let Some(event) = rx.recv().await {
        match event {
            GenerationEvent::Token(t) => full_text.push_str(&t),
            GenerationEvent::Message(msg) => full_text.push_str(&msg.content),
            GenerationEvent::Metrics(m) => {
                usage.prompt_tokens = m.prompt_tokens;
                usage.completion_tokens = m.generated_tokens;
                usage.total_tokens = m.prompt_tokens + m.generated_tokens;
            }
            _ => {}
        }
    }

    Ok(CompletionResponse {
        id,
        object: "text_completion".to_string(),
        created: now_unix(),
        model: model_name,
        choices: vec![CompletionChoice {
            text: full_text,
            index: 0,
            finish_reason: Some("stop".to_string()),
        }],
        usage,
    })
}

async fn create_legacy_completion_stream(
    state: Arc<OpenAIServerState>,
    req: CompletionRequest,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, (StatusCode, Json<ErrorResponse>)> {
    // Check if model is loaded
    {
        let guard = state
            .model_state
            .lock()
            .map_err(|_| server_error("Lock failed"))?;
        if !guard.scheduler.has_model() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ApiError {
                        message: "No model loaded".into(),
                        error_type: "invalid_request_error".into(),
                        code: None,
                    },
                }),
            ));
        }
    }

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let backend = Box::new(OpenAIBackend::new(tx));
    let id = format!("cmpl-{}", generate_id());
    let model_id = req.model.clone();

    let gen_req = GenerateRequest {
        prompt: req.prompt.clone(),
        messages: None,
        temperature: req.temperature,
        top_p: req.top_p,
        max_new_tokens: req.max_tokens,
        tools: None,
        top_k: None,
        min_p: None,
        repeat_penalty: None,
        repeat_last_n: 64,
        seed: None,
        use_custom_params: true,
        tracing: None,
        verbose_prompt: None,
        split_prompt: None,
        attachments: None,
        edit_index: None,
        format: None,
    };

    let state_clone = state.model_state.clone();

    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = generate_stream_with_backend(state_clone, gen_req, backend) {
            log::error!("Generation failed: {}", e);
        }
    });

    let stream = stream::unfold(
        (rx, id, model_id, false, false),
        move |(mut rx, id, model_id, mut finished, done_sent)| async move {
            if done_sent {
                return None;
            }

            if finished {
                return Some((
                    Ok(Event::default().data("[DONE]")),
                    (rx, id, model_id, true, true),
                ));
            }

            match rx.recv().await {
                Some(event) => {
                    let chunk = match event {
                        GenerationEvent::Token(t) => CompletionResponse {
                            id: id.clone(),
                            object: "text_completion".to_string(), // Keep consistent
                            created: now_unix(),
                            model: model_id.clone(),
                            choices: vec![CompletionChoice {
                                text: t,
                                index: 0,
                                finish_reason: None,
                            }],
                            usage: Usage {
                                prompt_tokens: 0,
                                completion_tokens: 0,
                                total_tokens: 0,
                            },
                        },
                        GenerationEvent::Message(msg) => CompletionResponse {
                            id: id.clone(),
                            object: "text_completion".to_string(),
                            created: now_unix(),
                            model: model_id.clone(),
                            choices: vec![CompletionChoice {
                                text: msg.content,
                                index: 0,
                                finish_reason: None,
                            }],
                            usage: Usage {
                                prompt_tokens: 0,
                                completion_tokens: 0,
                                total_tokens: 0,
                            },
                        },
                        // Handle other events as needed, or map them to empty chunks/log
                        GenerationEvent::Done => {
                            finished = true;
                            CompletionResponse {
                                id: id.clone(),
                                object: "text_completion".to_string(),
                                created: now_unix(),
                                model: model_id.clone(),
                                choices: vec![CompletionChoice {
                                    text: "".to_string(),
                                    index: 0,
                                    finish_reason: Some("stop".to_string()),
                                }],
                                usage: Usage {
                                    prompt_tokens: 0,
                                    completion_tokens: 0,
                                    total_tokens: 0,
                                },
                            }
                        }
                        _ => CompletionResponse {
                            id: id.clone(),
                            object: "text_completion".to_string(),
                            created: now_unix(),
                            model: model_id.clone(),
                            choices: vec![CompletionChoice {
                                text: String::new(),
                                index: 0,
                                finish_reason: None,
                            }],
                            usage: Usage {
                                prompt_tokens: 0,
                                completion_tokens: 0,
                                total_tokens: 0,
                            },
                        },
                    };

                    let data = serde_json::to_string(&chunk).unwrap_or_default();
                    Some((
                        Ok(Event::default().data(data)),
                        (rx, id, model_id, finished, done_sent),
                    ))
                }
                None => None,
            }
        },
    );

    Ok(stream)
}

fn server_error(msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: ApiError {
                message: msg.into(),
                error_type: "server_error".into(),
                code: None,
            },
        }),
    )
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generate_id() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..12)
        .map(|_| {
            let idx = rng.random_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect()
}

// ============================================================================
// Router
// ============================================================================

pub fn create_router(state: Arc<OpenAIServerState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/v1/models", get(models_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/completions", post(completions_handler))
        .layer(cors)
        .with_state(state)
}

// ============================================================================
// Server lifecycle
// ============================================================================

pub async fn start_server(
    model_state: SharedState,
    port: u16,
) -> Result<broadcast::Sender<()>, std::io::Error> {
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let state = Arc::new(OpenAIServerState {
        model_state,
        shutdown_tx: shutdown_tx.clone(),
    });

    let app = create_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    log::info!("OpenAI API server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let shutdown_rx = shutdown_tx.subscribe();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let mut rx = shutdown_rx;
                let _ = rx.recv().await;
                log::info!("OpenAI API server shutting down");
            })
            .await
            .ok();
    });

    Ok(shutdown_tx)
}
