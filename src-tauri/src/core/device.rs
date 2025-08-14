use candle::Device;
use crate::core::types::DevicePreference;

pub fn select_device(pref: Option<DevicePreference>) -> Device {
    match pref.unwrap_or(DevicePreference::Auto) {
        // По умолчанию всегда CPU, даже если CUDA доступна.
        // Переключение на CUDA выполняется явно через UI/команду set_device.
        DevicePreference::Auto => Device::Cpu,
        DevicePreference::Cpu => Device::Cpu,
        DevicePreference::Cuda { index } => Device::cuda_if_available(index).unwrap_or(Device::Cpu),
        DevicePreference::Metal => Device::Cpu,
    }
}

pub fn device_label(d: &Device) -> &'static str {
    match d {
        Device::Cpu => "CPU",
        Device::Cuda(_) => "CUDA",
        Device::Metal(_) => "Metal",
    }
}


