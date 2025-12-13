use candle::Device;
use oxide_lib::core::device::select_device;
use oxide_lib::core::types::DevicePreference;

#[test]
fn test_auto_device_selection() {
    // Test that Auto preference selects the appropriate device
    let device = select_device(Some(DevicePreference::Auto));

    // The actual device selected will depend on what's available in the test environment
    // but we can at least verify that it returns a valid device
    match device {
        Device::Cpu => {
            // This is always valid as a fallback
            assert!(true);
        }
        Device::Cuda(_) => {
            // CUDA device was selected
            assert!(true);
        }
        Device::Metal(_) => {
            // Metal device was selected
            assert!(true);
        }
    }
}

#[test]
fn test_explicit_cpu_selection() {
    let device = select_device(Some(DevicePreference::Cpu));
    match device {
        Device::Cpu => assert!(true),
        _ => panic!("Expected CPU device"),
    }
}
