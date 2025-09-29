// Типы для метрик производительности

export interface PerformanceMetric {
  operation_name: string;
  duration_ms: number;
  timestamp: string;
  memory_usage_mb: number;
  additional_data?: any;
}

export interface ModelLoadMetrics {
  total_duration_ms: number;
  stages: LoadStage[];
  model_size_mb: number;
  memory_before_mb: number;
  memory_after_mb: number;
  memory_delta_mb: number;
}

export interface LoadStage {
  name: string;
  duration_ms: number;
}

export interface InferenceMetrics {
  prompt_tokens: number;
  generated_tokens: number;
  total_duration_ms: number;
  prefill_duration_ms: number;
  generation_duration_ms: number;
  tokens_per_second: number;
  prefill_tokens_per_second: number;
  memory_usage_mb: number;
  timestamp: string;
}

export interface StartupMetrics {
  total_duration_ms: number;
  stages: StartupStage[];
  memory_at_start_mb: number;
  memory_at_ready_mb: number;
  timestamp: string;
}

export interface StartupStage {
  name: string;
  duration_ms: number;
}

export interface PerformanceSummary {
  current_memory_mb: number;
  last_model_load?: ModelLoadMetrics;
  last_inference?: InferenceMetrics;
  startup?: StartupMetrics;
  average_tokens_per_second: number;
  total_generated_tokens: number;
}
