use candle::{DType, Tensor};
// use tauri::Emitter; // Removed

use super::cancel::CANCEL_GENERATION;
use super::{
    ctx::ContextSlice,
    emit::{ChunkEmitter, EmissionBackend, GenerationEvent, TauriBackend},
    minp::MinPFilter,
    sampling::build_logits_processor_from_options,
    thinking_parser::ThinkingParser,
    tool_call_parser::ToolCallParser,
};
use crate::core::attachments_text::gather_text_from_attachments;
use crate::core::config::SamplingOptions;
use crate::core::performance::InferenceTracker;
use crate::core::prompt::PromptBuilder;
use crate::core::state::SharedState;
use crate::core::token_output_stream::TokenOutputStream;
use crate::core::tokenizer::{extract_bos_token_str, extract_eos_ids};
use crate::core::types::{ChatMessage, GenerateRequest};
use crate::models::ModelBackend;
use crate::{log_infer, log_template_error};
use std::sync::atomic::Ordering;
use tracing_subscriber::prelude::*;
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

pub fn generate_stream_impl(
    app: tauri::AppHandle,
    state: SharedState<Box<dyn ModelBackend + Send>>,
    req: GenerateRequest,
) -> Result<(), String> {
    let backend = Box::new(TauriBackend::new(app));
    generate_stream_with_backend(state, req, backend)
}

pub fn generate_stream_with_backend(
    state: SharedState<Box<dyn ModelBackend + Send>>,
    req: GenerateRequest,
    backend: Box<dyn EmissionBackend>,
) -> Result<(), String> {
    let _trace_guard = if req.tracing.unwrap_or(false) {
        let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new().build();
        let subscriber = tracing_subscriber::registry().with(chrome_layer);
        let _ = tracing::subscriber::set_global_default(subscriber);
        Some(guard)
    } else {
        None
    };
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
    backend.emit(GenerationEvent::Token(String::new())); // Keep this direct emit for now as it's separate from generation loop

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

    // Determine limit for prompt: context_length - reservation
    // This ensures we always have space for generation.
    let reserve_default = 512;
    let limit_reserve = (guard.context_length as f64 * 0.4) as usize;
    let generation_reserve = req
        .max_new_tokens
        .unwrap_or(reserve_default)
        .min(limit_reserve)
        .max(64);
    let prompt_limit = guard
        .context_length
        .saturating_sub(generation_reserve)
        .max(1);

    // Ollama-style "smart" truncation:
    // 1. Always keep System Prompt (if present).
    // 2. Always keep Last Message (if possible).
    // 3. Fill remaining space with history from newest to oldest.
    let prompt = if let Some(messages) = msgs {
        if messages.is_empty() {
            String::new()
        } else {
            // Extract system messages
            let system_msgs: Vec<ChatMessage> = messages
                .iter()
                .filter(|m| m.role == "system")
                .cloned()
                .collect();

            // All non-system messages
            let other_msgs: Vec<ChatMessage> = messages
                .iter()
                .filter(|m| m.role != "system")
                .cloned()
                .collect();

            if other_msgs.is_empty() {
                // Only system messages?
                build_prompt_with_template_bos(&guard.chat_template, system_msgs, bos_opt.clone())?
            } else {
                // Try to fit as many recent messages as possible
                let mut best_prompt = String::new();
                let n = other_msgs.len();

                // Try from including ALL to including ONLY LAST
                // Note: Ollama goes from n-1 down to 0 (including i..end).
                // We do the same.
                for i in 0..n {
                    let subset_others = &other_msgs[i..]; // suffix starting at i
                    let mut candidate_msgs = system_msgs.clone();
                    candidate_msgs.extend_from_slice(subset_others);

                    let p = build_prompt_with_template_bos(
                        &guard.chat_template,
                        candidate_msgs,
                        bos_opt.clone(),
                    )?;

                    // Check length (expensive but necessary for correct truncation)
                    let encoded = tos
                        .tokenizer()
                        .encode(p.clone(), true)
                        .map_err(|e| e.to_string())?;
                    if encoded.get_ids().len() <= prompt_limit {
                        best_prompt = p;
                        break; // We found the largest suffix that fits (since we started with longest? NO)
                    // Wait, if we want the largest suffix, we should start with i=0 (longest) and increment i (shorten) until it fits.
                    // Correct. i=0 means ALL messages. If it fits, we are good. break.
                    // If not, i=1 (drop oldest).
                    } else if i == n - 1 {
                        // Even the last message + system doesn't fit!
                        // We must use it and let it be truncated by raw token limit later if needed,
                        // OR we force it (user wants last message).
                        // We set best_prompt to this minimal set.
                        best_prompt = p;
                    }
                }

                // If best_prompt is still empty (shouldn't happen logic above covers it), default to last msg
                if best_prompt.is_empty() {
                    let mut minimal = system_msgs.clone();
                    minimal.push(other_msgs.last().unwrap().clone());
                    best_prompt = build_prompt_with_template_bos(
                        &guard.chat_template,
                        minimal,
                        bos_opt.clone(),
                    )?;
                }
                best_prompt
            }
        }
    } else {
        prompt_str
    };

    // Detect implicit thinking: if prompt ends with <think>, start parser in thinking mode
    let starts_in_thinking = prompt.trim_end().ends_with("<think>");

    let tokens = tos
        .tokenizer()
        .encode(prompt, true)
        .map_err(|e| e.to_string())?;
    let full_context_tokens = tokens.get_ids().to_vec();

    if req.verbose_prompt.unwrap_or(false) {
        let toks = tokens.get_tokens();
        let ids = tokens.get_ids();
        let mut dump = String::new();
        for (tok, id) in toks.iter().zip(ids.iter()) {
            let t = tok.replace('▁', " ").replace("<0x0A>", "\n");
            dump.push_str(&format!("{id:7} -> '{t}'\n"));
        }
        backend.emit(GenerationEvent::PromptDump(dump));
    }
    {
        let mut sample: Vec<u32> = full_context_tokens.iter().copied().take(16).collect();
        if sample.len() < full_context_tokens.len() {
            sample.push(0xFFFF_FFFF);
        }
        log_infer!("encoded token ids (first ~16): {:?}", sample);
    }

    let ctx_slice = ContextSlice::new(full_context_tokens.clone(), prompt_limit);
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

    // Сбрасываем KV-кэш перед новым запросом, чтобы не тянуть состояние предыдущего диалога.
    if let Some(model) = guard.gguf_model.as_mut() {
        model.clear_kv_cache();
    }

    // Начинаем prefill
    inference_tracker.start_prefill();

    let mut next_token = {
        let split_prompt = req.split_prompt.unwrap_or(false);
        let do_batched = !split_prompt && effective_context_tokens.len() > 8;
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
            // Convert to F32 for sampling (like candle examples)
            let logits = logits.to_dtype(DType::F32).map_err(|e| e.to_string())?;
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
            // Convert to F32 for sampling (like candle examples)
            let logits = logits.to_dtype(DType::F32).map_err(|e| e.to_string())?;
            let logits = minp.apply(&logits)?;
            logits_processor
                .sample(&logits)
                .map_err(|e| e.to_string())?
        }
    };

    // Начинаем generation
    inference_tracker.start_generation();

    let mut emitter = ChunkEmitter::new(backend);

    emitter.emit_start(); // Signal frontend to create assistant message

    log::debug!("[stream] starts_in_thinking: {}", starts_in_thinking);

    let mut thinking_parser = if starts_in_thinking {
        ThinkingParser::new_in_thinking_mode()
    } else {
        ThinkingParser::new()
    };

    // Create tool call parser if tools are provided
    let mut tool_call_parser = req.tools.as_ref().map(|tools| {
        log_infer!("tool calling enabled with {} tools", tools.len());
        ToolCallParser::with_json_tag(tools.clone())
    });

    if let Some(t) = tos.next_token(next_token).map_err(|e| e.to_string())? {
        let chunk = thinking_parser.process_token(&t);
        // Process tool calls if parser is active
        if let Some(ref mut tcp) = tool_call_parser {
            let result = tcp.add(&chunk.content);
            for call in result.calls {
                emitter.emit_tool_call(&call);
            }
        }
        emitter.emit_message(chunk);
    }

    let stop_ids = extract_eos_ids(tos.tokenizer());
    if stop_ids.is_empty() {
        return Err("Tokenizer: unable to determine EOS/STOP ids".into());
    }
    let eos_token = stop_ids[0];

    let mut all_tokens: Vec<u32> = vec![next_token];
    let mut stop_text_buf = String::new();
    for index in 0..to_sample_soft_cap {
        let _span = tracing::info_span!("decode", index).entered();
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
        let logits = logits.squeeze(0).map_err(|e| e.to_string())?;
        // Convert to F32 for sampling (like candle examples)
        let mut logits = logits.to_dtype(DType::F32).map_err(|e| e.to_string())?;
        if let Some(rp) = repeat_penalty
            && (rp - 1.0).abs() > f32::EPSILON
        {
            let start_at = all_tokens.len().saturating_sub(req.repeat_last_n);
            let penalty_tokens = &all_tokens[start_at..];
            // Only apply penalty if we have tokens to penalize (avoids shape mismatch with empty slice)
            if !penalty_tokens.is_empty() {
                if index == 0 {
                    log_infer!(
                        "repeat_penalty enabled: value={:.3}, last_n={}, applying to {} tokens",
                        rp,
                        req.repeat_last_n,
                        penalty_tokens.len()
                    );
                }
                logits =
                    candle_transformers::utils::apply_repeat_penalty(&logits, rp, penalty_tokens)
                        .map_err(|e| e.to_string())?;
            }
        }
        let logits = minp.apply(&logits)?;
        next_token = logits_processor
            .sample(&logits)
            .map_err(|e| e.to_string())?;
        all_tokens.push(next_token);
        inference_tracker.increment_generated_tokens();

        if next_token == eos_token || stop_ids.contains(&next_token) {
            break;
        }

        if let Some(t) = tos.next_token(next_token).map_err(|e| e.to_string())? {
            let chunk = thinking_parser.process_token(&t);
            // Process tool calls if parser is active
            if let Some(ref mut tcp) = tool_call_parser {
                let result = tcp.add(&chunk.content);
                for call in result.calls {
                    emitter.emit_tool_call(&call);
                }
            }
            emitter.emit_message(chunk);
            stop_text_buf.push_str(&t);
            if stop_text_buf.len() > 128 {
                let mut cut = stop_text_buf.len() - 128;
                while cut < stop_text_buf.len() && !stop_text_buf.is_char_boundary(cut) {
                    cut += 1;
                }
                if cut > 0 && cut <= stop_text_buf.len() {
                    let _ = stop_text_buf.drain(..cut);
                }
            }
            if stop_text_buf.contains("<end_of_turn>")
                || stop_text_buf.contains("<|end_of_turn|>")
                || stop_text_buf.contains("<|eot_id|>")
                || stop_text_buf.contains("</s>")
            {
                break;
            }
        }
    }

    if let Some(rest) = tos.decode_rest().map_err(|e| e.to_string())? {
        let chunk = thinking_parser.process_token(&rest);
        emitter.emit_message(chunk);
    }
    // Flush any remaining buffered partial tags
    let final_chunk = thinking_parser.flush();
    emitter.emit_message(final_chunk);
    emitter.finalize();

    // Очищаем KV-кэш после запроса, чтобы следующее поколение стартовало с чистого состояния.
    if let Some(model) = guard.gguf_model.as_mut() {
        model.clear_kv_cache();
    }

    // Финализируем метрики inference - используем существующий runtime если доступен
    let inference_metrics = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(async { inference_tracker.finish().await }),
        Err(_) => {
            // Fallback: создаём новый runtime только если текущий недоступен
            tokio::runtime::Runtime::new()
                .map_err(|e| e.to_string())?
                .block_on(async { inference_tracker.finish().await })
        }
    };

    log_infer!(
        "Метрики inference: prompt_tokens={}, generated_tokens={}, total_time={}ms, tokens/sec={:.2}, memory={:.2}MB",
        inference_metrics.prompt_tokens,
        inference_metrics.generated_tokens,
        inference_metrics.total_duration_ms,
        inference_metrics.tokens_per_second,
        inference_metrics.memory_usage_mb
    );

    // Отправляем метрики на фронтенд
    emitter.emit_metrics(inference_metrics);

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
