pub mod bridge;
pub mod client;
pub mod definition;
pub mod installer;
pub mod process;

use definition::EngineDefinition;
use futures_util::StreamExt;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct EngineManager {
    engines: Mutex<HashMap<String, EngineInstance>>,
    client: client::EngineClient,
}

pub struct EngineInstance {
    definition: EngineDefinition,
    process: Option<process::ManagedProcess>,
    status: EngineStatus,
    cancellation_token: Arc<AtomicBool>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EngineStatus {
    Stopped,
    Starting,
    Ready,
    Error(String),
}

impl EngineManager {
    pub fn new() -> Self {
        Self {
            engines: Mutex::new(HashMap::new()),
            client: client::EngineClient::new(),
        }
    }

    pub async fn load_definitions(&self) -> Result<(), String> {
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let candidates = vec![cwd.join("src-tauri/engines.json"), cwd.join("engines.json")];

        let config_path = candidates.into_iter().find(|p| p.exists());

        if let Some(path) = config_path {
            info!("Loading engines from {:?}", path);
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let defs: Vec<EngineDefinition> =
                serde_json::from_str(&content).map_err(|e| e.to_string())?;

            let mut engines_map = self.engines.lock().await;
            for def in defs {
                info!("Loaded engine definition: {}", def.id);
                engines_map.insert(
                    def.id.clone(),
                    EngineInstance {
                        definition: def,
                        process: None,
                        status: EngineStatus::Stopped,
                        cancellation_token: Arc::new(AtomicBool::new(false)),
                    },
                );
            }
        } else {
            warn!("engines.json not found in search paths");
        }
        Ok(())
    }

    fn find_free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .and_then(|l| l.local_addr())
            .map(|a| a.port())
            .unwrap_or(8080)
    }
}

// Tauri Commands

#[tauri::command]
pub async fn list_engines(
    app: AppHandle,
    state: State<'_, EngineManager>,
) -> Result<Vec<installer::EngineInstallInfo>, String> {
    let engines = state.engines.lock().await;
    let mut result = Vec::new();
    for instance in engines.values() {
        let def = &instance.definition;
        let installed = installer::is_engine_installed(&app, def);
        let binary_path = installer::resolve_engine_binary(&app, def)
            .ok()
            .map(|p| p.to_string_lossy().to_string());
        let download_url = def
            .find_variant_for_current_os()
            .and_then(|v| v.download_url.clone());
        result.push(installer::EngineInstallInfo {
            engine_id: def.id.clone(),
            name: def.name.clone(),
            description: def.description.clone(),
            installed,
            binary_path,
            download_url,
            capabilities: def
                .capabilities
                .iter()
                .map(|c| format!("{:?}", c).to_lowercase())
                .collect(),
        });
    }
    Ok(result)
}

#[tauri::command]
pub async fn install_engine_cmd(
    app: AppHandle,
    state: State<'_, EngineManager>,
    engine_id: String,
) -> Result<String, String> {
    let def = {
        let engines = state.engines.lock().await;
        let instance = engines.get(&engine_id).ok_or("Engine ID not found")?;
        instance.definition.clone()
    };

    let path = installer::install_engine(app, &def).await?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn uninstall_engine_cmd(
    app: AppHandle,
    state: State<'_, EngineManager>,
    engine_id: String,
) -> Result<(), String> {
    let exists = {
        let engines = state.engines.lock().await;
        engines.contains_key(&engine_id)
    };
    if !exists {
        return Err("Engine ID not found".to_string());
    }
    installer::uninstall_engine(&app, &engine_id)
}

#[tauri::command]
pub async fn start_engine(
    app: AppHandle,
    state: State<'_, EngineManager>,
    engine_id: String,
    model_path: String,
) -> Result<(), String> {
    let mut engines = state.engines.lock().await;
    let instance = engines.get_mut(&engine_id).ok_or("Engine ID not found")?;

    if let EngineStatus::Ready = instance.status {
        // If already ready, verify process is actually alive?
        // For now, assume it is.
        info!("Engine {} is already running.", engine_id);

        // TODO: check if model_path is different? Multimodel switching requires restart.
        // Simplified: always restart if user asks to start explicit model?
        // Or user calls "stop" then "start".
        // Let's assume frontend logic ensures stop/start for model change.
        return Ok(());
    }

    instance.status = EngineStatus::Starting;
    app.emit(
        "engine_status",
        serde_json::json!({ "engine_id": engine_id, "status": "starting" }),
    )
    .unwrap_or(());

    info!("Starting engine {} with model {}", engine_id, model_path);

    instance
        .definition
        .find_variant_for_current_os()
        .ok_or("No binary variants defined for current platform")?;
    let binary_path = installer::resolve_engine_binary(&app, &instance.definition)?;

    if !binary_path.exists() {
        let err = format!("Binary not found at {:?}", binary_path);
        instance.status = EngineStatus::Error(err.clone());
        return Err(err);
    }

    let port = EngineManager::find_free_port();
    let mut args: Vec<String> = Vec::new();
    for arg_tpl in &instance.definition.args_template {
        let replaced = arg_tpl
            .replace("{model_path}", &model_path)
            .replace("{port}", &port.to_string());
        args.push(replaced);
    }

    let process_res = process::ManagedProcess::new(
        binary_path
            .to_str()
            .ok_or_else(|| format!("Invalid engine binary path: {}", binary_path.display()))?,
        &args,
        port,
        &instance.definition.health_endpoint,
    );

    match process_res {
        Ok(proc) => {
            if let Err(e) = proc.wait_ready().await {
                instance.status = EngineStatus::Error(e.clone());
                app.emit(
                    "engine_status",
                    serde_json::json!({ "engine_id": engine_id, "status": "error", "error": e }),
                )
                .unwrap_or(());
                return Err(e);
            }

            instance.process = Some(proc);
            instance.status = EngineStatus::Ready;
            instance.cancellation_token.store(false, Ordering::SeqCst);

            info!("Engine {} started successfully on port {}", engine_id, port);
            app.emit(
                "engine_status",
                serde_json::json!({
                    "engine_id": engine_id,
                    "status": "ready",
                    "port": port
                }),
            )
            .unwrap_or(());
            Ok(())
        }
        Err(e) => {
            let err_str = e.to_string();
            instance.status = EngineStatus::Error(err_str.clone());
            app.emit(
                "engine_status",
                serde_json::json!({ "engine_id": engine_id, "status": "error", "error": err_str }),
            )
            .unwrap_or(());
            Err(err_str)
        }
    }
}

#[tauri::command]
pub async fn stop_engine(
    app: AppHandle,
    state: State<'_, EngineManager>,
    engine_id: String,
) -> Result<(), String> {
    let mut engines = state.engines.lock().await;
    let instance = engines.get_mut(&engine_id).ok_or("Engine ID not found")?;

    if let Some(mut proc) = instance.process.take() {
        let _ = proc.kill();
    }
    instance.status = EngineStatus::Stopped;
    app.emit(
        "engine_status",
        serde_json::json!({
            "engine_id": engine_id,
            "status": "stopped"
        }),
    )
    .unwrap_or(());
    Ok(())
}

#[tauri::command]
pub async fn generate(
    app: AppHandle,
    state: State<'_, EngineManager>,
    engine_id: String,
    req: serde_json::Value,
) -> Result<(), String> {
    let (port, token, client) = {
        let engines = state.engines.lock().await;
        let instance = engines.get(&engine_id).ok_or("Engine not found")?;

        if let EngineStatus::Ready = instance.status {
            match &instance.process {
                Some(proc) => (
                    proc.port,
                    instance.cancellation_token.clone(),
                    state.client.clone(),
                ),
                None => return Err("Engine process missing".to_string()),
            }
        } else {
            return Err(format!("Engine is not ready: {:?}", instance.status));
        }
    };

    token.store(false, Ordering::SeqCst);
    let base_url = format!("http://127.0.0.1:{}", port);

    // Notify frontend start
    app.emit("message_start", ()).map_err(|e| e.to_string())?;

    let mut stream = client.stream_chat_completion(&base_url, req).await?;

    while let Some(chunk_res) = stream.next().await {
        if token.load(Ordering::Relaxed) {
            info!("Generation cancelled by user");
            break;
        }

        match chunk_res {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                let events = bridge::parse_sse_chunk(&text);

                for event in events {
                    if event.done {
                        app.emit("message_done", ()).map_err(|e| e.to_string())?;
                        return Ok(());
                    }
                    app.emit("message", &event).map_err(|e| e.to_string())?;
                }
            }
            Err(e) => {
                error!("Stream error: {}", e);
                return Err(e.to_string());
            }
        }
    }

    app.emit("message_done", ()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn cancel_generation(
    state: State<'_, EngineManager>,
    engine_id: String,
) -> Result<(), String> {
    let engines = state.engines.lock().await;
    if let Some(instance) = engines.get(&engine_id) {
        instance.cancellation_token.store(true, Ordering::SeqCst);
    }
    Ok(())
}
