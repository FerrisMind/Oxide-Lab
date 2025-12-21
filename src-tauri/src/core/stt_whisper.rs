use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use byteorder::ByteOrder;
use candle::{Device, IndexOp, Tensor};
use candle_nn::ops::softmax;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokenizers::Tokenizer;

use crate::core::types::{SttModelSource, SttSettings};

use candle_transformers::models::whisper::{self as whisper, Config};
use candle_transformers::quantized_var_builder::VarBuilder;

const MODEL_FILENAME: &str = "model-tiny-q80.gguf";
const TOKENIZER_FILENAME: &str = "tokenizer-tiny.json";
const CONFIG_FILENAME: &str = "config-tiny.json";
const SETTINGS_FILENAME: &str = "stt_settings.json";

static WHISPER_STATE: OnceCell<Mutex<Option<WhisperState>>> = OnceCell::new();

struct WhisperState {
    settings: SttSettings,
    config: Config,
    tokenizer: Tokenizer,
    model: whisper::quantized_model::Whisper,
    mel_filters: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeRequest {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub repo_id: String,
    pub revision: Option<String>,
    pub model_filename: String,
    pub tokenizer_filename: String,
    pub config_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResponse {
    pub model_dir: String,
}

pub fn load_settings(app: &AppHandle) -> Result<SttSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(SttSettings::default());
    }
    let data =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read STT settings: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse STT settings: {e}"))
}

pub fn save_settings(app: &AppHandle, settings: &SttSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {e}"))?;
    }
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize STT settings: {e}"))?;
    fs::write(&path, data).map_err(|e| format!("Failed to write STT settings: {e}"))
}

pub fn transcribe(app: &AppHandle, req: TranscribeRequest) -> Result<String, String> {
    if req.sample_rate != whisper::SAMPLE_RATE as u32 {
        return Err(format!(
            "Unsupported sample rate {} (expected {})",
            req.sample_rate,
            whisper::SAMPLE_RATE
        ));
    }

    let settings = load_settings(app)?;
    let cache = WHISPER_STATE.get_or_init(|| Mutex::new(None));
    let mut guard = cache
        .lock()
        .map_err(|_| "Failed to lock STT state".to_string())?;
    let needs_reload = match guard.as_ref() {
        Some(state) => state.settings != settings,
        None => true,
    };
    if needs_reload {
        *guard = Some(load_model(app, &settings)?);
    }
    let state = guard
        .as_mut()
        .ok_or_else(|| "Failed to initialize STT state".to_string())?;

    state.model.reset_kv_cache();

    let device = Device::Cpu;
    let mel = whisper::audio::pcm_to_mel(&state.config, &req.samples, &state.mel_filters);
    let mel_len = mel.len();
    let mel = Tensor::from_vec(
        mel,
        (
            1,
            state.config.num_mel_bins,
            mel_len / state.config.num_mel_bins,
        ),
        &device,
    )
    .map_err(|e| format!("Failed to build mel tensor: {e}"))?;

    let language_token = match req.language.as_deref() {
        None | Some("auto") => Some(
            detect_language(&mut state.model, &state.tokenizer, &mel)
                .map_err(|e| format!("Failed to detect language: {e}"))?,
        ),
        Some(language) => Some(language_token(&state.tokenizer, language)?),
    };
    decode_greedy(
        &mut state.model,
        &state.tokenizer,
        &state.config,
        &mel,
        language_token,
    )
    .map_err(|e| format!("Failed to decode audio: {e}"))
}

pub async fn download_model(
    app: &AppHandle,
    req: DownloadRequest,
) -> Result<DownloadResponse, String> {
    use hf_hub::api::tokio::ApiBuilder;
    use hf_hub::{Repo, RepoType};
    use tokio::task;

    let base_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?
        .join("oxide-lab")
        .join("stt_models");
    let repo_dir = base_dir.join(sanitize_repo_id(&req.repo_id));
    task::spawn_blocking({
        let repo_dir = repo_dir.clone();
        move || {
            fs::create_dir_all(&repo_dir)
                .map_err(|e| format!("Failed to create STT model dir: {e}"))
        }
    })
    .await
    .map_err(|e| format!("Failed to join STT dir task: {e}"))??;

    let api = ApiBuilder::new()
        .with_progress(false)
        .build()
        .map_err(|e| format!("Failed to init hf-hub: {e}"))?;
    let revision = req.revision.clone().unwrap_or_else(|| "main".to_string());
    let repo = api.repo(Repo::with_revision(
        req.repo_id.clone(),
        RepoType::Model,
        revision,
    ));

    let model_path = repo
        .get(&req.model_filename)
        .await
        .map_err(|e| format!("Failed to download model file: {e}"))?;
    let tokenizer_path = repo
        .get(&req.tokenizer_filename)
        .await
        .map_err(|e| format!("Failed to download tokenizer file: {e}"))?;
    let config_path = repo
        .get(&req.config_filename)
        .await
        .map_err(|e| format!("Failed to download config file: {e}"))?;

    let copies = vec![
        (model_path, req.model_filename.clone()),
        (tokenizer_path, req.tokenizer_filename.clone()),
        (config_path, req.config_filename.clone()),
    ];

    let repo_dir_clone = repo_dir.clone();
    task::spawn_blocking(move || -> Result<(), String> {
        for (src, name) in copies {
            let dest = repo_dir_clone.join(name);
            fs::copy(&src, &dest).map_err(|e| format!("Failed to copy downloaded file: {e}"))?;
        }
        Ok(())
    })
    .await
    .map_err(|e| format!("Failed to join download copy task: {e}"))??;

    Ok(DownloadResponse {
        model_dir: repo_dir.to_string_lossy().to_string(),
    })
}

fn load_model(app: &AppHandle, settings: &SttSettings) -> Result<WhisperState, String> {
    let model_dir = resolve_model_dir(app, settings)?;
    let config_path = model_dir.join(CONFIG_FILENAME);
    let tokenizer_path = model_dir.join(TOKENIZER_FILENAME);
    let model_path = model_dir.join(MODEL_FILENAME);

    if !config_path.exists() || !tokenizer_path.exists() || !model_path.exists() {
        return Err("STT model files are missing".to_string());
    }

    let config_data =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read STT config: {e}"))?;
    let config: Config = serde_json::from_str(&config_data)
        .map_err(|e| format!("Failed to parse STT config: {e}"))?;

    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| format!("Failed to load STT tokenizer: {e}"))?;

    let mel_filters = load_mel_filters(&config)?;

    let device = Device::Cpu;
    let vb = VarBuilder::from_gguf(&model_path, &device)
        .map_err(|e| format!("Failed to load STT GGUF: {e}"))?;
    let model = whisper::quantized_model::Whisper::load(&vb, config.clone())
        .map_err(|e| format!("Failed to build STT model: {e}"))?;

    Ok(WhisperState {
        settings: settings.clone(),
        config,
        tokenizer,
        model,
        mel_filters,
    })
}

fn load_mel_filters(config: &Config) -> Result<Vec<f32>, String> {
    let bytes = include_bytes!("whisper_melfilters.bytes");
    let mut filters = vec![0f32; bytes.len() / 4];
    byteorder::LittleEndian::read_f32_into(bytes, &mut filters);
    if config.num_mel_bins == 80 {
        Ok(filters)
    } else {
        Err(format!("Unsupported mel bins: {}", config.num_mel_bins))
    }
}

fn resolve_model_dir(app: &AppHandle, settings: &SttSettings) -> Result<PathBuf, String> {
    match settings.source {
        SttModelSource::Bundled => resolve_bundled_model_dir(app),
        SttModelSource::Custom => {
            let dir = settings
                .custom_dir
                .as_ref()
                .ok_or_else(|| "Custom STT directory is not set".to_string())?;
            let path = PathBuf::from(dir);
            if path.exists() {
                Ok(path)
            } else {
                Err("Custom STT directory does not exist".to_string())
            }
        }
    }
}

fn resolve_bundled_model_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to resolve resource dir: {e}"))?;
    let candidate = resource_dir.join("whisper");
    if candidate.exists() {
        return Ok(candidate);
    }
    let dev_candidate = std::env::current_dir()
        .map_err(|e| format!("Failed to get current dir: {e}"))?
        .join("src-tauri")
        .join("whisper");
    if dev_candidate.exists() {
        return Ok(dev_candidate);
    }
    Err("Bundled STT model directory not found".to_string())
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    Ok(base.join("oxide-lab").join(SETTINGS_FILENAME))
}

fn sanitize_repo_id(repo_id: &str) -> String {
    repo_id.replace('/', "__")
}

fn token_id(tokenizer: &Tokenizer, token: &str) -> Result<u32, String> {
    let token = tokenizer
        .token_to_id(token)
        .ok_or_else(|| format!("Token not found: {token}"))?;
    Ok(token)
}

fn language_token(tokenizer: &Tokenizer, language: &str) -> Result<u32, String> {
    let supported = LANGUAGES.iter().any(|(code, _)| *code == language);
    if !supported {
        return Err(format!("Unsupported STT language: {language}"));
    }
    token_id(tokenizer, &format!("<|{language}|>"))
}

fn decode_greedy(
    model: &mut whisper::quantized_model::Whisper,
    tokenizer: &Tokenizer,
    config: &Config,
    mel: &Tensor,
    language_token: Option<u32>,
) -> Result<String, String> {
    let device = mel.device();
    let audio_features = model
        .encoder
        .forward(mel, true)
        .map_err(|e| format!("Failed to run encoder: {e}"))?;

    let mut tokens = Vec::new();
    tokens.push(token_id(tokenizer, whisper::SOT_TOKEN)?);
    if let Some(lang) = language_token {
        tokens.push(lang);
    }
    tokens.push(token_id(tokenizer, whisper::TRANSCRIBE_TOKEN)?);
    tokens.push(token_id(tokenizer, whisper::NO_TIMESTAMPS_TOKEN)?);

    let eot_token = token_id(tokenizer, whisper::EOT_TOKEN)?;
    while tokens.len() < config.max_target_positions {
        let input = Tensor::new(tokens.as_slice(), device)
            .map_err(|e| format!("Failed to build token tensor: {e}"))?
            .unsqueeze(0)
            .map_err(|e| format!("Failed to build token batch: {e}"))?;
        let ys = model
            .decoder
            .forward(&input, &audio_features, true)
            .map_err(|e| format!("Failed to run decoder: {e}"))?;
        let logits = model
            .decoder
            .final_linear(&ys)
            .map_err(|e| format!("Failed to compute logits: {e}"))?;
        let last = logits
            .i((0, tokens.len() - 1))
            .map_err(|e| format!("Failed to slice logits: {e}"))?;
        let mut scores = last
            .to_vec1::<f32>()
            .map_err(|e| format!("Failed to read logits: {e}"))?;
        for token in &config.suppress_tokens {
            if let Some(score) = scores.get_mut(*token as usize) {
                *score = f32::NEG_INFINITY;
            }
        }
        let (next_token, _) = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .ok_or_else(|| "Failed to choose next token".to_string())?;
        let next_token = next_token as u32;
        tokens.push(next_token);
        if next_token == eot_token {
            break;
        }
    }

    tokenizer
        .decode(&tokens, true)
        .map_err(|e| format!("Failed to decode tokens: {e}"))
}

const LANGUAGES: [(&str, &str); 99] = [
    ("en", "english"),
    ("zh", "chinese"),
    ("de", "german"),
    ("es", "spanish"),
    ("ru", "russian"),
    ("ko", "korean"),
    ("fr", "french"),
    ("ja", "japanese"),
    ("pt", "portuguese"),
    ("tr", "turkish"),
    ("pl", "polish"),
    ("ca", "catalan"),
    ("nl", "dutch"),
    ("ar", "arabic"),
    ("sv", "swedish"),
    ("it", "italian"),
    ("id", "indonesian"),
    ("hi", "hindi"),
    ("fi", "finnish"),
    ("vi", "vietnamese"),
    ("he", "hebrew"),
    ("uk", "ukrainian"),
    ("el", "greek"),
    ("ms", "malay"),
    ("cs", "czech"),
    ("ro", "romanian"),
    ("da", "danish"),
    ("hu", "hungarian"),
    ("ta", "tamil"),
    ("no", "norwegian"),
    ("th", "thai"),
    ("ur", "urdu"),
    ("hr", "croatian"),
    ("bg", "bulgarian"),
    ("lt", "lithuanian"),
    ("la", "latin"),
    ("mi", "maori"),
    ("ml", "malayalam"),
    ("cy", "welsh"),
    ("sk", "slovak"),
    ("te", "telugu"),
    ("fa", "persian"),
    ("lv", "latvian"),
    ("bn", "bengali"),
    ("sr", "serbian"),
    ("az", "azerbaijani"),
    ("sl", "slovenian"),
    ("kn", "kannada"),
    ("et", "estonian"),
    ("mk", "macedonian"),
    ("br", "breton"),
    ("eu", "basque"),
    ("is", "icelandic"),
    ("hy", "armenian"),
    ("ne", "nepali"),
    ("mn", "mongolian"),
    ("bs", "bosnian"),
    ("kk", "kazakh"),
    ("sq", "albanian"),
    ("sw", "swahili"),
    ("gl", "galician"),
    ("mr", "marathi"),
    ("pa", "punjabi"),
    ("si", "sinhala"),
    ("km", "khmer"),
    ("sn", "shona"),
    ("yo", "yoruba"),
    ("so", "somali"),
    ("af", "afrikaans"),
    ("oc", "occitan"),
    ("ka", "georgian"),
    ("be", "belarusian"),
    ("tg", "tajik"),
    ("sd", "sindhi"),
    ("gu", "gujarati"),
    ("am", "amharic"),
    ("yi", "yiddish"),
    ("lo", "lao"),
    ("uz", "uzbek"),
    ("fo", "faroese"),
    ("ht", "haitian creole"),
    ("ps", "pashto"),
    ("tk", "turkmen"),
    ("nn", "nynorsk"),
    ("mt", "maltese"),
    ("sa", "sanskrit"),
    ("lb", "luxembourgish"),
    ("my", "myanmar"),
    ("bo", "tibetan"),
    ("tl", "tagalog"),
    ("mg", "malagasy"),
    ("as", "assamese"),
    ("tt", "tatar"),
    ("haw", "hawaiian"),
    ("ln", "lingala"),
    ("ha", "hausa"),
    ("ba", "bashkir"),
    ("jw", "javanese"),
    ("su", "sundanese"),
];

fn detect_language(
    model: &mut whisper::quantized_model::Whisper,
    tokenizer: &Tokenizer,
    mel: &Tensor,
) -> Result<u32, String> {
    let (_bsize, _, seq_len) = mel.dims3().map_err(|e| format!("{e}"))?;
    let mel = mel
        .narrow(2, 0, usize::min(seq_len, model.config.max_source_positions))
        .map_err(|e| format!("Failed to narrow mel: {e}"))?;
    let device = mel.device();
    let language_token_ids = LANGUAGES
        .iter()
        .map(|(token, _)| token_id(tokenizer, &format!("<|{token}|>")))
        .collect::<Result<Vec<_>, _>>()?;
    let sot_token = token_id(tokenizer, whisper::SOT_TOKEN)?;
    let audio_features = model
        .encoder
        .forward(&mel, true)
        .map_err(|e| format!("Failed to run encoder: {e}"))?;
    let tokens = Tensor::new(&[[sot_token]], device)
        .map_err(|e| format!("Failed to build token tensor: {e}"))?;
    let language_token_ids = Tensor::new(language_token_ids.as_slice(), device)
        .map_err(|e| format!("Failed to build language tensor: {e}"))?;
    let ys = model
        .decoder
        .forward(&tokens, &audio_features, true)
        .map_err(|e| format!("Failed to run decoder: {e}"))?;
    let logits = model
        .decoder
        .final_linear(&ys.i(..1).map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("Failed to compute logits: {e}"))?
        .i(0)
        .map_err(|e| format!("{e}"))?
        .i(0)
        .map_err(|e| format!("{e}"))?;
    let logits = logits
        .index_select(&language_token_ids, 0)
        .map_err(|e| format!("{e}"))?;
    let probs = softmax(&logits, candle::D::Minus1)
        .map_err(|e| format!("{e}"))?
        .to_vec1::<f32>()
        .map_err(|e| format!("{e}"))?;
    let mut probs = LANGUAGES.iter().zip(probs.iter()).collect::<Vec<_>>();
    probs.sort_by(|(_, p1), (_, p2)| p2.total_cmp(p1));
    let language = token_id(tokenizer, &format!("<|{}|>", probs[0].0.0))?;
    Ok(language)
}
