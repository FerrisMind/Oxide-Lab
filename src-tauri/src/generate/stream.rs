use candle::Tensor;
use tauri::Emitter;

use crate::core::state::SharedState;
use crate::models::common::model::ModelBackend;
use crate::core::token_output_stream::TokenOutputStream;
use crate::core::tokenizer::extract_eos_ids;
use crate::core::prompt::PromptBuilder;
use crate::core::config::SamplingOptions;
use crate::core::types::GenerateRequest;
use super::{sampling::build_logits_processor_from_options, minp::MinPFilter, emit::ChunkEmitter, ctx::ContextSlice};
use super::cancel::CANCEL_GENERATION;
use std::sync::atomic::Ordering;

pub async fn generate_stream_cmd(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    req: GenerateRequest,
) -> Result<(), String> {
    CANCEL_GENERATION.store(false, Ordering::SeqCst);
    let app_clone = app.clone();
    let state_arc: SharedState<Box<dyn ModelBackend + Send>> = state.inner().clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        generate_stream_impl(app_clone, state_arc, req)
    })
    .await
    .map_err(|e| e.to_string())?;
    res
}

pub fn detect_no_think(req: &GenerateRequest) -> bool {
    let prompt_lower = req.prompt.to_lowercase();
    prompt_lower.contains("<think>") && prompt_lower.contains("</think>")
}

fn generate_stream_impl(
    app: tauri::AppHandle,
    state: SharedState<Box<dyn ModelBackend + Send>>,
    req: GenerateRequest,
) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let tokenizer = match (&guard.gguf_model, &guard.tokenizer) {
        (Some(_), Some(tk)) => tk.clone(),
        _ => return Err("Model/tokenizer is not loaded".into()),
    };

    let mut tos = TokenOutputStream::new(tokenizer);
    // Детекция наличия тега <think>...</think> в промпте. Учитываем возможные вариации регистра.
    let is_no_think = detect_no_think(&req);

    // Если обнаружили режим no_think, логируем это отдельно
    if is_no_think {
        println!("[infer] no_think detected in prompt");
    }
    // Дефолты семплинга не зависят от режима размышлений.
    let (def_temp, def_top_p, def_min_p, def_top_k) = (0.7_f64, Some(0.9_f64), Some(0.0_f64), Some(20_usize));
    // Вычисляем эффективные значения. Если пользовательские параметры включены,
    // не используем дефолты, а берём только переданные параметры; для температуры
    // нейтральным значением считаем 1.0.
    let temperature: f64 = if req.use_custom_params {
        req.temperature.unwrap_or(1.0_f64)
    } else { def_temp };
    let top_p: Option<f64> = if req.use_custom_params { req.top_p } else { def_top_p };
    let top_k: Option<usize> = if req.use_custom_params { req.top_k } else { def_top_k };
    let min_p: Option<f64> = if req.use_custom_params { req.min_p } else { def_min_p };
    // Включаем лёгкий repeat_penalty по умолчанию только когда пользовательские параметры выключены
    let repeat_penalty: Option<f32> = if req.use_custom_params { req.repeat_penalty } else { Some(1.1_f32) };
    println!(
        "[infer] request: prompt_len={}, temperature={:.3}, top_k={:?}, top_p={:?}, min_p={:?}, repeat_penalty={:?}, repeat_last_n={}, use_custom_params={}, no_think_detected={}",
        req.prompt.len(), temperature, top_k, top_p, min_p, repeat_penalty, req.repeat_last_n, req.use_custom_params, is_no_think
    );
    let _ = app.emit("token", String::new());

    // Use chat messages if provided, otherwise use the prompt directly
    let prompt = if let Some(messages) = req.messages {
        // Build prompt using chat template
        build_prompt_with_template(&guard.chat_template, messages)?
    } else {
        req.prompt
    };
    let tokens = tos
        .tokenizer()
        .encode(prompt, true)
        .map_err(|e| e.to_string())?;
    let full_context_tokens = tokens.get_ids().to_vec();
    let ctx_slice = ContextSlice::new(full_context_tokens.clone(), guard.context_length.max(1));
    let effective_context_tokens: Vec<u32> = ctx_slice.effective_context_tokens.clone();
    if ctx_slice.base_context_len != ctx_slice.encoded_len {
        println!(
            "[infer] context: encoded={}, using={}, truncated_by={}",
            ctx_slice.encoded_len, ctx_slice.base_context_len, ctx_slice.encoded_len.saturating_sub(ctx_slice.base_context_len)
        );
    } else {
        println!("[infer] context: encoded={}, using=encoded (no truncation)", ctx_slice.encoded_len);
    }

    let to_sample_soft_cap = guard.context_length.saturating_sub(ctx_slice.base_context_len).saturating_sub(1);
    let seed: u64 = req.seed.unwrap_or(42);
    let sampling_options = SamplingOptions {
        temperature,
        top_k,
        top_p,
        min_p,
        seed: Some(seed),
        repeat_penalty,
        repeat_last_n: req.repeat_last_n,
    };
    let (mut logits_processor, sampling_desc) = build_logits_processor_from_options(&sampling_options);
    println!("[infer] sampling strategy: {}", sampling_desc);
    let mut minp = MinPFilter::new(min_p, temperature);

    let _vocab = tos.tokenizer().get_vocab(true);

    // Prefill: пошагово прогоняем весь контекст через kv_cache, без кастомной causal mask
    let mut next_token = {
        let mut last_logits_opt: Option<Tensor> = None;
        for (i, &tok) in effective_context_tokens.iter().enumerate() {
            let input = Tensor::new(&[tok], &guard.device)
                .map_err(|e| e.to_string())?
                .unsqueeze(0)
                .map_err(|e| e.to_string())?;
            let logits = if let Some(mut model) = guard.gguf_model.take() {
                let res = model.forward_layered(&input, i);
                match res {
                    Ok(v) => { guard.gguf_model = Some(model); v },
                    Err(e) => { guard.gguf_model = Some(model); return Err(e.to_string()); }
                }
            } else { return Err("Model is not loaded".into()); };
            last_logits_opt = Some(logits);
        }
        let logits = last_logits_opt.ok_or_else(|| "Empty context".to_string())?;
        let logits = logits.squeeze(0).map_err(|e| e.to_string())?;
        let logits = minp.apply(&logits)?;
        logits_processor.sample(&logits).map_err(|e| e.to_string())?
    };

    let mut emitter = ChunkEmitter::new(app.clone(), is_no_think);

    if let Some(t) = tos.next_token(next_token).map_err(|e| e.to_string())? { emitter.push_maybe_emit(&t); }

    let stop_ids = extract_eos_ids(tos.tokenizer());
    if stop_ids.is_empty() {
        return Err("Tokenizer: unable to determine EOS/STOP ids".into());
    }
    let eos_token = stop_ids[0];

    let mut all_tokens: Vec<u32> = vec![next_token];
    for index in 0..to_sample_soft_cap {
        if CANCEL_GENERATION.load(Ordering::SeqCst) { println!("[infer] cancelled by user"); break; }
        let input = Tensor::new(&[next_token], &guard.device)
            .map_err(|e| e.to_string())?
            .unsqueeze(0)
            .map_err(|e| e.to_string())?;
        let logits = if let Some(mut model) = guard.gguf_model.take() {
            let res = model.forward_layered(&input, ctx_slice.base_context_len + index);
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
        let logits = minp.apply(&logits)?;
        next_token = logits_processor.sample(&logits).map_err(|e| e.to_string())?;
        all_tokens.push(next_token);
        if let Some(t) = tos.next_token(next_token).map_err(|e| e.to_string())? { emitter.push_maybe_emit(&t); }
        if next_token == eos_token || stop_ids.contains(&next_token) { break; }
    }

    if let Some(rest) = tos.decode_rest().map_err(|e| e.to_string())? { emitter.push_maybe_emit(&rest); }
    emitter.flush();
    Ok(())
}

/// Build a prompt using the prompt builder with chat template support
pub fn build_prompt_with_template(
    chat_template: &Option<String>,
    messages: Vec<crate::core::types::ChatMessage>,
) -> Result<String, String> {
    let builder = PromptBuilder::new(chat_template.clone());
    
    // Convert core::types::ChatMessage to core::prompt::ChatMessage
    let prompt_messages: Vec<crate::core::prompt::ChatMessage> = messages.into_iter().map(|msg| {
        crate::core::prompt::ChatMessage {
            role: msg.role,
            content: msg.content,
        }
    }).collect();
    
    // Try to render with template first, fallback to custom formatting
    if builder.has_template() {
        match builder.render_prompt(prompt_messages.clone()) {
            Ok(rendered) => Ok(rendered),
            Err(e) => {
                println!("[template] render failed: {}, falling back to custom formatting", e);
                Ok(builder.build_fallback_prompt(prompt_messages))
            }
        }
    } else {
        Ok(builder.build_fallback_prompt(prompt_messages))
    }
}


