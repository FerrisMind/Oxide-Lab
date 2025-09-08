use candle::Device;
use candle::utils::{cuda_is_available, metal_is_available};
use crate::core::types::DevicePreference;

pub fn select_device(pref: Option<DevicePreference>) -> Device {
    match pref.unwrap_or(DevicePreference::Auto) {
        DevicePreference::Auto => {
            // Авто-выбор устройства CUDA → Metal → CPU, как в примерах Candle
            // Проверяем CUDA только если фича включена при компиляции
            if cuda_is_available() {
                match Device::new_cuda(0) {
                    Ok(device) => {
                        println!("[device] auto-selected CUDA");
                        return device;
                    }
                    Err(e) => {
                        eprintln!("[device] CUDA init failed: {}, falling back to next option", e);
                    }
                }
            }
            
            // Проверяем Metal только если фича включена при компиляции
            if metal_is_available() {
                match Device::new_metal(0) {
                    Ok(device) => {
                        println!("[device] auto-selected Metal");
                        return device;
                    }
                    Err(e) => {
                        eprintln!("[device] Metal init failed: {}, falling back to CPU", e);
                    }
                }
            }
            
            println!("[device] auto-selected CPU");
            Device::Cpu
        },
        DevicePreference::Cpu => Device::Cpu,
        DevicePreference::Cuda { index } => {
            match Device::new_cuda(index) {
                Ok(device) => device,
                Err(e) => {
                    eprintln!("[device] CUDA init failed: {}, falling back to CPU", e);
                    Device::Cpu
                }
            }
        },
        DevicePreference::Metal => {
            match Device::new_metal(0) {
                Ok(device) => device,
                Err(e) => {
                    eprintln!("[device] Metal init failed: {}, falling back to CPU", e);
                    Device::Cpu
                }
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