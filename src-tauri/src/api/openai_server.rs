//! OpenAI-compatible HTTP API server.
//!
//! Provides `/v1/chat/completions` and `/v1/models` endpoints for compatibility
//! with OpenAI clients (Cursor, Continue, Open WebUI, etc.).

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post},
    Router,
};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::core::state::SharedState;
use crate::core::types::ChatMessage;
use crate::generate::tool_call_parser::{Tool, ToolCall};
use crate::models::ModelBackend;

// ============================================================================
// OpenAI API Types
// ============================================================================

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

impl From<OpenAIMessage> for ChatMessage {
    fn from(msg: OpenAIMessage) -> Self {
        ChatMessage {
            role: msg.role,
            content: msg.content,
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

// ============================================================================
// Server State
// ============================================================================

pub struct OpenAIServerState {
    pub model_state: SharedState<Box<dyn ModelBackend + Send>>,
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

    let models = if guard.gguf_model.is_some() {
        let model_id = guard
            .model_path
            .as_ref()
            .and_then(|p| {
                std::path::Path::new(p)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(String::from)
            })
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

    Ok(Json(ModelList {
        object: "list".to_string(),
        data: models,
    }))
}

async fn chat_completions_handler(
    State(state): State<Arc<OpenAIServerState>>,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if req.stream {
        // For streaming, return SSE
        let stream = create_completion_stream(state, req).await?;
        Ok(Sse::new(stream).keep_alive(KeepAlive::default()).into_response())
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
    let guard = state.model_state.lock().map_err(|_| server_error("Lock failed"))?;

    if guard.gguf_model.is_none() {
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

    drop(guard);

    // For now, return a placeholder - actual generation requires async integration
    // TODO: Integrate with actual generation pipeline
    Ok(ChatCompletion {
        id: format!("chatcmpl-{}", generate_id()),
        object: "chat.completion".to_string(),
        created: now_unix(),
        model: req.model,
        choices: vec![Choice {
            index: 0,
            message: ResponseMessage {
                role: "assistant".to_string(),
                content: "OpenAI API server is running. Full generation requires model to be loaded via Tauri.".to_string(),
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    })
}

async fn create_completion_stream(
    state: Arc<OpenAIServerState>,
    req: ChatCompletionRequest,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, (StatusCode, Json<ErrorResponse>)> {
    let guard = state.model_state.lock().map_err(|_| server_error("Lock failed"))?;

    if guard.gguf_model.is_none() {
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

    drop(guard);

    let id = format!("chatcmpl-{}", generate_id());
    let model = req.model.clone();

    // Create a simple stream with placeholder response
    // TODO: Integrate with actual generation pipeline
    let chunks = vec![
        ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: now_unix(),
            model: model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: Some("assistant".to_string()),
                    content: None,
                },
                finish_reason: None,
            }],
        },
        ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: now_unix(),
            model: model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: None,
                    content: Some("OpenAI API streaming works!".to_string()),
                },
                finish_reason: None,
            }],
        },
        ChatCompletionChunk {
            id,
            object: "chat.completion.chunk".to_string(),
            created: now_unix(),
            model,
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta::default(),
                finish_reason: Some("stop".to_string()),
            }],
        },
    ];

    let stream = stream::iter(chunks.into_iter().map(|chunk| {
        let data = serde_json::to_string(&chunk).unwrap_or_default();
        Ok(Event::default().data(data))
    }));

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
        .layer(cors)
        .with_state(state)
}

// ============================================================================
// Server lifecycle
// ============================================================================

pub async fn start_server(
    model_state: SharedState<Box<dyn ModelBackend + Send>>,
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
