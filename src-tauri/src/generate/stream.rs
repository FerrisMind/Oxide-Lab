use candle::Tensor;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use tauri::Emitter;

use crate::state::SharedState;
use crate::token_output_stream::TokenOutputStream;
use crate::GenerateRequest;
// use crate::model::qwen3_offload::ModelWeights as OffloadModel;
use super::cancel::CANCEL_GENERATION;
use std::sync::atomic::Ordering;

pub async fn generate_stream_cmd(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState>,
    req: GenerateRequest,
) -> Result<(), String> {
    CANCEL_GENERATION.store(false, Ordering::SeqCst);
    let app_clone = app.clone();
    let state_arc: SharedState = state.inner().clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        generate_stream_impl(app_clone, state_arc, req)
    })
    .await
    .map_err(|e| e.to_string())?;
    res
}

fn generate_stream_impl(
    app: tauri::AppHandle,
    state: SharedState,
    req: GenerateRequest,
) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let tokenizer = match (&guard.gguf_model, &guard.tokenizer) {
        (Some(_), Some(tk)) => tk.clone(),
        _ => return Err("Model/tokenizer is not loaded".into()),
    };

    let mut tos = TokenOutputStream::new(tokenizer);
    // Базовые дефолты семплинга (не зависят от режима размышлений; управление теперь через теги /think и /no_think в промпте)
    let (def_temp, def_top_p, def_min_p, def_top_k) = (0.7_f64, Some(0.9_f64), Some(0.0_f64), Some(20_usize));
    // Вычисляем эффективные значения
    let temperature: f64 = if req.use_custom_params {
        req.temperature.unwrap_or(def_temp)
    } else { def_temp };
    let top_p: Option<f64> = if req.use_custom_params { req.top_p.or(def_top_p) } else { def_top_p };
    let top_k: Option<usize> = if req.use_custom_params { req.top_k.or(def_top_k) } else { def_top_k };
    let min_p: Option<f64> = if req.use_custom_params { req.min_p.or(def_min_p) } else { def_min_p };
    // Включаем лёгкий repeat_penalty по умолчанию, чтобы уменьшить навязчивое повторение фраз
    let repeat_penalty: Option<f32> = if req.use_custom_params { req.repeat_penalty } else { Some(1.1_f32) };
    println!(
        "[infer] request: prompt_len={}, temperature={:.3}, top_k={:?}, top_p={:?}, min_p={:?}, repeat_penalty={:?}, repeat_last_n={}, use_custom_params={}",
        req.prompt.len(), temperature, top_k, top_p, min_p, repeat_penalty, req.repeat_last_n, req.use_custom_params
    );
    let _ = app.emit("token", String::new());

    let prompt = req.prompt;
    let tokens = tos
        .tokenizer()
        .encode(prompt, true)
        .map_err(|e| e.to_string())?;
    let full_context_tokens = tokens.get_ids().to_vec();
    let encoded_len = full_context_tokens.len();
    let effective_context_tokens: Vec<u32> = {
        let n = guard.context_length.max(1);
        if encoded_len > n {
            let start = encoded_len - n;
            full_context_tokens[start..].to_vec()
        } else {
            full_context_tokens.clone()
        }
    };
    let base_context_len = effective_context_tokens.len();
    if base_context_len != encoded_len {
        println!(
            "[infer] context: encoded={}, using={}, truncated_by={}",
            encoded_len, base_context_len, encoded_len.saturating_sub(base_context_len)
        );
    } else {
        println!("[infer] context: encoded={}, using=encoded (no truncation)", encoded_len);
    }

    let to_sample_soft_cap = guard.context_length.saturating_sub(base_context_len).saturating_sub(1);
    let mut logits_processor = {
        let (sampling, sampling_desc) = if temperature <= 0.0 {
            (Sampling::ArgMax, format!("ArgMax"))
        } else {
            match (top_k, top_p) {
                (None, None) => {
                    (Sampling::All { temperature },
                     format!("All(temp={:.3})", temperature))
                }
                (Some(k), None) => {
                    (Sampling::TopK { k, temperature },
                     format!("TopK(k={}, temp={:.3})", k, temperature))
                }
                (None, Some(p)) => {
                    (Sampling::TopP { p, temperature },
                     format!("TopP(p={:.3}, temp={:.3})", p, temperature))
                }
                (Some(k), Some(p)) => {
                    (Sampling::TopKThenTopP { k, p, temperature },
                     format!("TopKThenTopP(k={}, p={:.3}, temp={:.3})", k, p, temperature))
                }
            }
        };
        println!("[infer] sampling strategy: {}", sampling_desc);
        LogitsProcessor::from_sampling(42, sampling)
    };

    let mut log_min_p_once = true;
    let mut apply_min_p_mask = |logits: &Tensor| -> Result<Tensor, String> {
        let min_p = match min_p { Some(v) if v > 0.0 && v <= 1.0 => v as f32, _ => return Ok(logits.clone()) };
        if temperature <= 0.0 {
            if log_min_p_once { println!("[infer] min_p ignored because temperature <= 0"); log_min_p_once = false; }
            return Ok(logits.clone());
        }
        let t = temperature as f32;
        let vals: Vec<f32> = logits.to_vec1::<f32>().map_err(|e| e.to_string())?;
        if vals.is_empty() { return Ok(logits.clone()); }
        let mut max_val = f32::NEG_INFINITY;
        for &v in &vals { if v > max_val { max_val = v; } }
        let threshold = max_val + t * (min_p.ln());
        let mut kept = 0usize;
        let masked: Vec<f32> = vals.into_iter().map(|v| { if v >= threshold { kept += 1; v } else { f32::NEG_INFINITY } }).collect();
        if log_min_p_once { println!("[infer] min_p applied: p={:.3}, temp={:.3}, threshold={:.4}, kept={} of {}", min_p, t, threshold, kept, masked.len()); log_min_p_once = false; }
        Tensor::new(masked.as_slice(), logits.device()).map_err(|e| e.to_string())
    };

    let vocab = tos.tokenizer().get_vocab(true);

    let mut next_token = {
        let input = Tensor::new(effective_context_tokens.as_slice(), &guard.device)
            .map_err(|e| e.to_string())?
            .unsqueeze(0)
            .map_err(|e| e.to_string())?;
        // Извлекаем модель отдельно, чтобы избежать двойного заимствования guard
        let logits = if let Some(mut model) = guard.gguf_model.take() {
            let res = model.forward_layered(&input, 0);
            match res {
                Ok(v) => {
                    let dev_name = match &guard.device { candle::Device::Cpu => "CPU", candle::Device::Cuda(_) => "CUDA", candle::Device::Metal(_) => "Metal" };
                    println!("[infer] forward on: {} (n_gpu_layers={})", dev_name, guard.n_gpu_layers);
                    guard.gguf_model = Some(model);
                    v
                },
                Err(e) => {
                    // С pos‑offload режимом веса уже распределены при загрузке; если OOM происходит, просто пробрасываем ошибку
                    {
                        guard.gguf_model = Some(model);
                        return Err(e.to_string());
                    }
                }
            }
        } else { return Err("Model is not loaded".into()); };
        let logits = logits.squeeze(0).map_err(|e| e.to_string())?;
        let logits = apply_min_p_mask(&logits)?;
        logits_processor.sample(&logits).map_err(|e| e.to_string())?
    };

    use std::time::{Duration, Instant};
    let mut pending_chunk = String::new();
    let emit_interval = Duration::from_millis(16);
    let max_chunk_len = 2048usize;
    let mut last_emit_at = Instant::now();
    let maybe_emit = |app: &tauri::AppHandle, buf: &mut String, force: bool, last_emit_at: &mut Instant| {
        if buf.is_empty() { return; }
        if force || last_emit_at.elapsed() >= emit_interval || buf.len() >= max_chunk_len {
            let chunk = std::mem::take(buf);
            let _ = app.emit("token", chunk);
            *last_emit_at = Instant::now();
        }
    };

    if let Some(t) = tos.next_token(next_token).map_err(|e| e.to_string())? {
        pending_chunk.push_str(&t);
        maybe_emit(&app, &mut pending_chunk, false, &mut last_emit_at);
    }

    let eos_token = vocab
        .get("<|im_end|>")
        .copied()
        .ok_or_else(|| "Missing <|im_end|> token in tokenizer".to_string())?;
    let mut extra_stops: Vec<u32> = Vec::new();
    if let Some(id) = vocab.get("<|eot_id|>") { extra_stops.push(*id); }
    if let Some(id) = vocab.get("<|endoftext|>") { extra_stops.push(*id); }
    if let Some(id) = vocab.get("</s>") { extra_stops.push(*id); }

    let mut all_tokens: Vec<u32> = vec![next_token];
    for index in 0..to_sample_soft_cap {
        if CANCEL_GENERATION.load(Ordering::SeqCst) { println!("[infer] cancelled by user"); break; }
        let input = Tensor::new(&[next_token], &guard.device)
            .map_err(|e| e.to_string())?
            .unsqueeze(0)
            .map_err(|e| e.to_string())?;
        let logits = if let Some(mut model) = guard.gguf_model.take() {
            let res = model.forward_layered(&input, base_context_len + index);
            match res {
                Ok(v) => { guard.gguf_model = Some(model); v },
                Err(e) => {
                    {
                        guard.gguf_model = Some(model);
                        return Err(e.to_string());
                    }
                }
            }
        } else { return Err("Model is not loaded".into()); };
        let mut logits = logits.squeeze(0).map_err(|e| e.to_string())?;
        if let Some(rp) = repeat_penalty {
            if (rp - 1.0).abs() > f32::EPSILON {
                if index == 0 { println!("[infer] repeat_penalty enabled: value={:.3}, last_n={}", rp, req.repeat_last_n); }
                let start_at = all_tokens.len().saturating_sub(req.repeat_last_n);
                logits = candle_transformers::utils::apply_repeat_penalty(&logits, rp, &all_tokens[start_at..]).map_err(|e| e.to_string())?;
            }
        }
        let logits = apply_min_p_mask(&logits)?;
        next_token = logits_processor.sample(&logits).map_err(|e| e.to_string())?;
        all_tokens.push(next_token);
        if let Some(t) = tos.next_token(next_token).map_err(|e| e.to_string())? {
            pending_chunk.push_str(&t);
            maybe_emit(&app, &mut pending_chunk, false, &mut last_emit_at);
        }
        if next_token == eos_token || extra_stops.iter().any(|&s| s == next_token) { break; }
    }

    if let Some(rest) = tos.decode_rest().map_err(|e| e.to_string())? { pending_chunk.push_str(&rest); }
    maybe_emit(&app, &mut pending_chunk, true, &mut last_emit_at);
    Ok(())
}


