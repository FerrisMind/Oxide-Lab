// API команды для мониторинга производительности
use crate::core::performance::{PerformanceMetric, StartupMetrics};
use crate::core::state::SharedState;
use crate::models::common::model::ModelBackend;

/// Получить все метрики производительности
#[tauri::command]
pub async fn get_performance_metrics(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<Vec<PerformanceMetric>, String> {
    let monitor = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        guard.performance_monitor.clone()
    };
    let metrics = monitor.get_metrics().await;
    Ok(metrics)
}

/// Получить среднюю длительность операции
#[tauri::command]
pub async fn get_average_duration(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    operation_name: String,
) -> Result<Option<f64>, String> {
    let monitor = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        guard.performance_monitor.clone()
    };
    let duration = monitor.get_average_duration(&operation_name).await;
    Ok(duration)
}

/// Получить текущее использование памяти
#[tauri::command]
pub async fn get_memory_usage(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<f64, String> {
    let monitor = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        guard.performance_monitor.clone()
    };
    let memory_mb = monitor.get_memory_usage_mb().await;
    Ok(memory_mb)
}

/// Очистить все метрики производительности
#[tauri::command]
pub async fn clear_performance_metrics(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<(), String> {
    let monitor = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        guard.performance_monitor.clone()
    };
    monitor.clear_metrics().await;
    Ok(())
}

/// Получить метрики запуска приложения
#[tauri::command]
pub async fn get_startup_metrics(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<Option<StartupMetrics>, String> {
    let monitor = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        guard.performance_monitor.clone()
    };
    let metrics = monitor.get_startup_metrics().await;
    Ok(metrics)
}
