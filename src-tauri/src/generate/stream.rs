use candle::Tensor;
use tauri::Emitter;

use super::cancel::CANCEL_GENERATION;
use super::{
    ctx::ContextSlice, emit::ChunkEmitter, minp::MinPFilter,
    sampling::build_logits_processor_from_options,
};
use crate::core::attachments_text::gather_text_from_attachments;
use crate::core::config::SamplingOptions;
use crate::core::performance::InferenceTracker;
use crate::core::prompt::PromptBuilder;
use crate::core::state::SharedState;
use crate::core::token_output_stream::TokenOutputStream;
use crate::core::tokenizer::{extract_bos_token_str, extract_eos_ids};
use crate::core::types::{ChatMessage, GenerateRequest};
use crate::models::common::model::ModelBackend;
use crate::{log_infer, log_template_error};
use std::sync::atomic::Ordering;
// Мультимодальные вложения отключены

pub async fn generate_stream_cmd(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    req: GenerateRequest,
) -> Result<(), String> {
    CANCEL_GENERATION.store(false, Ordering::SeqCst);
    let app_clone = app.clone();
    let state_arc: SharedState<Box<dyn ModelBackend + Send>> = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || generate_stream_impl(app_clone, state_arc, req))
        .await
        .map_err(|e| e.to_string())?
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

    // Extract BOS token before moving tokenizer into the stream helper
    let bos_opt = extract_bos_token_str(&tokenizer);
    let mut tos = TokenOutputStream::new(tokenizer);

    // Дефолты семплинга не зависят от режима размышлений.
    let (def_temp, def_top_p, def_min_p, def_top_k) =
        (0.7_f64, Some(0.9_f64), Some(0.0_f64), Some(20_usize));
    // Вычисляем эффективные значения. Если пользовательские параметры включены,
    // не используем дефолты, а берём только переданные параметры; для температуры
    // нейтральным значением считаем 1.0.
    let temperature: f64 = if req.use_custom_params {
        req.temperature.unwrap_or(1.0_f64)
    } else {
        def_temp
    };
    let top_p: Option<f64> = if req.use_custom_params {
        req.top_p
    } else {
        def_top_p
    };
    let top_k: Option<usize> = if req.use_custom_params {
        req.top_k
    } else {
        def_top_k
    };
    let min_p: Option<f64> = if req.use_custom_params {
        req.min_p
    } else {
        def_min_p
    };
    // Включаем лёгкий repeat_penalty по умолчанию только когда пользовательские параметры выключены
    let repeat_penalty: Option<f32> = if req.use_custom_params {
        req.repeat_penalty
    } else {
        Some(1.1_f32)
    };
    log_infer!(
        "request: prompt_len={}, temperature={:.3}, top_k={:?}, top_p={:?}, min_p={:?}, repeat_penalty={:?}, repeat_last_n={}, use_custom_params={}",
        req.prompt.len(),
        temperature,
        top_k,
        top_p,
        min_p,
        repeat_penalty,
        req.repeat_last_n,
        req.use_custom_params
    );
    let _ = app.emit("token", String::new());

    // Текстовые вложения (.txt/.md): читаем и подмешиваем в последний user или в prompt
    let mut msgs = req.messages.clone();
    let mut prompt_str = req.prompt.clone();
    if let Some(attachments) = req.attachments.as_ref() {
        let combined = gather_text_from_attachments(attachments).map_err(|e| e.to_string())?;
        if !combined.is_empty() {
            if let Some(ref mut m) = msgs {
                if let Some(last) = m.last_mut() {
                    if last.role.to_lowercase() == "user" {
                        last.content = format!("{}\n\n{}", last.content, combined);
                    } else {
                        m.push(ChatMessage {
                            role: "user".into(),
                            content: combined,
                        });
                    }
                } else {
                    m.push(ChatMessage {
                        role: "user".into(),
                        content: combined,
                    });
                }
            } else if !prompt_str.is_empty() {
                prompt_str = format!("{}\n\n{}", prompt_str, combined);
            } else {
                prompt_str = combined;
            }
        }
    }

    // Используем либо чат, либо чистый prompt
    let prompt = if let Some(messages) = msgs {
        build_prompt_with_template_bos(&guard.chat_template, messages, bos_opt)?
    } else {
        prompt_str
    };
    let tokens = tos
        .tokenizer()
        .encode(prompt, true)
        .map_err(|e| e.to_string())?;
    let full_context_tokens = tokens.get_ids().to_vec();
    {
        let mut sample: Vec<u32> = full_context_tokens.iter().copied().take(16).collect();
        if sample.len() < full_context_tokens.len() {
            sample.push(0xFFFF_FFFF);
        }
        log_infer!("encoded token ids (first ~16): {:?}", sample);
    }
    let ctx_slice = ContextSlice::new(full_context_tokens.clone(), guard.context_length.max(1));
    let effective_context_tokens: Vec<u32> = ctx_slice.effective_context_tokens.clone();

    // Создаём трекер inference
    let mut inference_tracker = InferenceTracker::new(
        effective_context_tokens.len(),
        guard.performance_monitor.clone(),
    );

    if ctx_slice.base_context_len != ctx_slice.encoded_len {
        log_infer!(
            "context: encoded={}, using={}, truncated_by={}",
            ctx_slice.encoded_len,
            ctx_slice.base_context_len,
            ctx_slice
                .encoded_len
                .saturating_sub(ctx_slice.base_context_len)
        );
    } else {
        log_infer!(
            "context: encoded={}, using=encoded (no truncation)",
            ctx_slice.encoded_len
        );
    }

    // Effective soft cap for new tokens: honor request.max_new_tokens if provided,
    // otherwise allow full remaining context budget.
    let context_slack = guard
        .context_length
        .saturating_sub(ctx_slice.base_context_len)
        .saturating_sub(1);
    let to_sample_soft_cap = req
        .max_new_tokens
        .unwrap_or(context_slack)
        .min(context_slack);
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
    let (mut logits_processor, sampling_desc) =
        build_logits_processor_from_options(&sampling_options);
    log_infer!("sampling strategy: {}", sampling_desc);
    let mut minp = MinPFilter::new(min_p, temperature);

    let _vocab = tos.tokenizer().get_vocab(true);

    // Начинаем prefill
    inference_tracker.start_prefill();

    let mut next_token = {
        let do_batched = effective_context_tokens.len() > 8;
        if do_batched {
            let input = Tensor::new(effective_context_tokens.as_slice(), &guard.device)
                .map_err(|e| e.to_string())?
                .unsqueeze(0)
                .map_err(|e| e.to_string())?;
            let logits = match guard.gguf_model.take() {
                Some(mut model) => {
                    let res = model.forward_layered(&input, 0);
                    match res {
                        Ok(v) => {
                            guard.gguf_model = Some(model);
                            v
                        }
                        Err(e) => {
                            guard.gguf_model = Some(model);
                            return Err(e.to_string());
                        }
                    }
                }
                _ => {
                    return Err("Model is not loaded".into());
                }
            };
            let logits = logits.squeeze(0).map_err(|e| e.to_string())?;
            let logits = minp.apply(&logits)?;
            logits_processor
                .sample(&logits)
                .map_err(|e| e.to_string())?
        } else {
            let mut last_logits_opt: Option<Tensor> = None;
            for (i, &tok) in effective_context_tokens.iter().enumerate() {
                let input = Tensor::new(&[tok], &guard.device)
                    .map_err(|e| e.to_string())?
                    .unsqueeze(0)
                    .map_err(|e| e.to_string())?;
                let logits = match guard.gguf_model.take() {
                    Some(mut model) => {
                        let res = model.forward_layered(&input, i);
                        match res {
                            Ok(v) => {
                                guard.gguf_model = Some(model);
                                v
                            }
                            Err(e) => {
                                guard.gguf_model = Some(model);
                                return Err(e.to_string());
                            }
                        }
                    }
                    _ => {
                        return Err("Model is not loaded".into());
                    }
                };
                last_logits_opt = Some(logits);
            }
            let logits = last_logits_opt.ok_or_else(|| "Empty context".to_string())?;
            let logits = logits.squeeze(0).map_err(|e| e.to_string())?;
            let logits = minp.apply(&logits)?;
            logits_processor
                .sample(&logits)
                .map_err(|e| e.to_string())?
        }
    };

    // Начинаем generation
    inference_tracker.start_generation();

    let mut emitter = ChunkEmitter::new(app.clone());

    if let Some(t) = tos.next_token(next_token).map_err(|e| e.to_string())? {
        emitter.push_maybe_emit(&t);
    }

    let stop_ids = extract_eos_ids(tos.tokenizer());
    if stop_ids.is_empty() {
        return Err("Tokenizer: unable to determine EOS/STOP ids".into());
    }
    let eos_token = stop_ids[0];

    let mut all_tokens: Vec<u32> = vec![next_token];
    let mut stop_text_buf = String::new();
    for index in 0..to_sample_soft_cap {
        if CANCEL_GENERATION.load(Ordering::SeqCst) {
            log_infer!("cancelled by user");
            break;
        }
        let input = Tensor::new(&[next_token], &guard.device)
            .map_err(|e| e.to_string())?
            .unsqueeze(0)
            .map_err(|e| e.to_string())?;
        let logits = match guard.gguf_model.take() {
            Some(mut model) => {
                let res = model.forward_layered(&input, ctx_slice.base_context_len + index);
                match res {
                    Ok(v) => {
                        guard.gguf_model = Some(model);
                        v
                    }
                    Err(e) => {
                        guard.gguf_model = Some(model);
                        return Err(e.to_string());
                    }
                }
            }
            _ => {
                return Err("Model is not loaded".into());
            }
        };
        let mut logits = logits.squeeze(0).map_err(|e| e.to_string())?;
        if let Some(rp) = repeat_penalty
            && (rp - 1.0).abs() > f32::EPSILON
        {
            if index == 0 {
                log_infer!(
                    "repeat_penalty enabled: value={:.3}, last_n={}",
                    rp,
                    req.repeat_last_n
                );
            }
            let start_at = all_tokens.len().saturating_sub(req.repeat_last_n);
            logits = candle_transformers::utils::apply_repeat_penalty(
                &logits,
                rp,
                &all_tokens[start_at..],
            )
            .map_err(|e| e.to_string())?;
        }
        let logits = minp.apply(&logits)?;
        next_token = logits_processor
            .sample(&logits)
            .map_err(|e| e.to_string())?;
        all_tokens.push(next_token);
        inference_tracker.increment_generated_tokens();

        if let Some(t) = tos.next_token(next_token).map_err(|e| e.to_string())? {
            emitter.push_maybe_emit(&t);
            // textual stop patterns for models that emit plain tags instead of special ids
            stop_text_buf.push_str(&t);
            if stop_text_buf.len() > 128 {
                // keep small window, ensure char boundary
                let mut cut = stop_text_buf.len() - 128;
                while cut < stop_text_buf.len() && !stop_text_buf.is_char_boundary(cut) {
                    cut += 1;
                }
                if cut > 0 && cut <= stop_text_buf.len() {
                    let _ = stop_text_buf.drain(..cut);
                }
            }
            if stop_text_buf.contains("<end_of_turn>")
                || stop_text_buf.contains("<|eot_id|>")
                || stop_text_buf.contains("</s>")
            {
                break;
            }
        }
        if next_token == eos_token || stop_ids.contains(&next_token) {
            break;
        }
    }

    if let Some(rest) = tos.decode_rest().map_err(|e| e.to_string())? {
        emitter.push_maybe_emit(&rest);
    }
    emitter.finalize();

    // Финализируем метрики inference
    let inference_metrics = tokio::runtime::Runtime::new()
        .map_err(|e| e.to_string())?
        .block_on(async { inference_tracker.finish().await });

    log_infer!(
        "Метрики inference: prompt_tokens={}, generated_tokens={}, total_time={}ms, tokens/sec={:.2}, memory={:.2}MB",
        inference_metrics.prompt_tokens,
        inference_metrics.generated_tokens,
        inference_metrics.total_duration_ms,
        inference_metrics.tokens_per_second,
        inference_metrics.memory_usage_mb
    );

    // Отправляем метрики на фронтенд
    let _ = app.emit("inference_metrics", &inference_metrics);

    Ok(())
}

/// Build a prompt using the prompt builder with chat template support
pub fn build_prompt_with_template_bos(
    chat_template: &Option<String>,
    messages: Vec<crate::core::types::ChatMessage>,
    bos_token: Option<String>,
) -> Result<String, String> {
    let builder = PromptBuilder::new(chat_template.clone()).with_bos(bos_token);

    // Convert core::types::ChatMessage to core::prompt::ChatMessage
    let prompt_messages: Vec<crate::core::prompt::ChatMessage> = messages
        .into_iter()
        .map(|msg| crate::core::prompt::ChatMessage {
            role: msg.role,
            content: msg.content,
        })
        .collect();

    // Try to render with template first, fallback to custom formatting
    if builder.has_template() {
        match builder.render_prompt(prompt_messages.clone()) {
            Ok(rendered) => Ok(rendered),
            Err(e) => {
                log_template_error!("render failed: {}, falling back to custom formatting", e);
                Ok(builder.build_fallback_prompt(prompt_messages))
            }
        }
    } else {
        Ok(builder.build_fallback_prompt(prompt_messages))
    }
}

/// Backward-compatible helper without explicit BOS argument.
/// Delegates to `build_prompt_with_template_bos` with `bos_token=None`.
pub fn build_prompt_with_template(
    chat_template: &Option<String>,
    messages: Vec<crate::core::types::ChatMessage>,
) -> Result<String, String> {
    build_prompt_with_template_bos(chat_template, messages, None)
}
