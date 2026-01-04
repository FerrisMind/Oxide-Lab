use crate::core::device::device_label;
use crate::core::state::SharedState;
use crate::core::types::DevicePreference;
use crate::models::ModelBackend;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub fn set_device(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    pref: DevicePreference,
) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    crate::api::device::set_device(&mut guard, pref)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfoDto {
    pub cuda_build: bool,
    pub cuda_available: bool,
    pub current: String,
    pub avx: bool,
    pub neon: bool,
    pub simd128: bool,
    pub f16c: bool,
}

#[tauri::command]
pub fn get_device_info(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<DeviceInfoDto, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let current = device_label(&guard.device).to_string();
    let cuda_build = cfg!(feature = "cuda");
    #[cfg(feature = "cuda")]
    let cuda_available = candle::Device::cuda_if_available(0).is_ok();
    #[cfg(not(feature = "cuda"))]
    let cuda_available = false;
    let avx = candle::utils::with_avx();
    let neon = candle::utils::with_neon();
    let simd128 = candle::utils::with_simd128();
    let f16c = candle::utils::with_f16c();
    Ok(DeviceInfoDto {
        cuda_build,
        cuda_available,
        current,
        avx,
        neon,
        simd128,
        f16c,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeCudaDto {
    pub cuda_build: bool,
    pub ok: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub fn probe_cuda() -> Result<ProbeCudaDto, String> {
    let cuda_build = cfg!(feature = "cuda");
    #[cfg(feature = "cuda")]
    {
        match candle::Device::cuda_if_available(0) {
            Ok(_) => Ok(ProbeCudaDto {
                cuda_build,
                ok: true,
                error: None,
            }),
            Err(e) => Ok(ProbeCudaDto {
                cuda_build,
                ok: false,
                error: Some(e.to_string()),
            }),
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        Ok(ProbeCudaDto {
            cuda_build,
            ok: false,
            error: Some("built without cuda feature".to_string()),
        })
    }
}
