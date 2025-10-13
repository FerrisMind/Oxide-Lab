use crate::core::types::DevicePreference;
use crate::{log_device, log_device_error};
use candle::Device;
use candle::utils::{cuda_is_available, metal_is_available};

pub fn select_device(pref: Option<DevicePreference>) -> Device {
    match pref.unwrap_or(DevicePreference::Auto) {
        DevicePreference::Auto => {
            // Авто-выбор устройства CUDA → Metal → CPU, как в примерах Candle
            // Проверяем CUDA только если фича включена при компиляции
            if cuda_is_available() {
                match Device::new_cuda(0) {
                    Ok(device) => {
                        log_device!("auto-selected CUDA");
                        return device;
                    }
                    Err(e) => {
                        log_device_error!("CUDA init failed: {}, falling back to next option", e);
                    }
                }
            }

            // Проверяем Metal только если фича включена при компиляции
            if metal_is_available() {
                match Device::new_metal(0) {
                    Ok(device) => {
                        log_device!("auto-selected Metal");
                        return device;
                    }
                    Err(e) => {
                        log_device_error!("Metal init failed: {}, falling back to CPU", e);
                    }
                }
            }

            log_device!("auto-selected CPU");
            Device::Cpu
        }
        DevicePreference::Cpu => Device::Cpu,
        DevicePreference::Cuda { index } => match Device::new_cuda(index) {
            Ok(device) => device,
            Err(e) => {
                log_device_error!("CUDA init failed: {}, falling back to CPU", e);
                Device::Cpu
            }
        },
        DevicePreference::Metal => match Device::new_metal(0) {
            Ok(device) => device,
            Err(e) => {
                log_device_error!("Metal init failed: {}, falling back to CPU", e);
                Device::Cpu
            }
        },
    }
}

pub fn device_label(d: &Device) -> &'static str {
    match d {
        Device::Cpu => "CPU",
        Device::Cuda(_) => "CUDA",
        Device::Metal(_) => "Metal",
    }
}
