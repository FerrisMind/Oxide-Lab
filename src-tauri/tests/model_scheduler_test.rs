use candle::{DType, Device, Tensor};
use oxide_lib::core::scheduler::{ModelScheduler, SchedulerConfig};
use oxide_lib::models::ModelBackend;
use std::time::Duration;

// MockModel for testing
struct MockModel;

impl ModelBackend for MockModel {
    fn forward(&mut self, _input: &Tensor, _pos: usize) -> candle::Result<Tensor> {
        Tensor::zeros((1, 1), DType::F32, &Device::Cpu)
    }

    fn clear_kv_cache(&mut self) {}

    fn model_type(&self) -> &str {
        "MockModel"
    }

    fn vocab_size(&self) -> usize {
        1000
    }
}

#[test]
fn test_scheduler_expiration() {
    // Config with very short keep-alive
    let config = SchedulerConfig::default().with_keep_alive_secs(1);
    let mut scheduler = ModelScheduler::new(config);

    // Load model
    let model = Box::new(MockModel);
    scheduler.load_model(model, "mock-model".to_string());

    assert!(scheduler.has_model());
    assert_eq!(scheduler.get_model_id().as_deref(), Some("mock-model"));

    // Wait slightly more than keep-alive (1.1s)
    std::thread::sleep(Duration::from_millis(1100));

    // Check expiration - should unload
    scheduler.check_expiration();

    assert!(
        !scheduler.has_model(),
        "Model should be expired and unloaded"
    );
}

#[test]
fn test_scheduler_keep_alive_reset() {
    let config = SchedulerConfig::default().with_keep_alive_secs(1);
    let mut scheduler = ModelScheduler::new(config);

    scheduler.load_model(Box::new(MockModel), "mock-model".to_string());

    // Wait 0.6s
    std::thread::sleep(Duration::from_millis(600));

    // Use model
    if let Some(mut entry) = scheduler.take_model() {
        // ... use model (simulate forward) ...
        let _ = entry
            .model
            .forward(&Tensor::zeros((1, 1), DType::U32, &Device::Cpu).unwrap(), 0);
        scheduler.restore_model(entry);
    } else {
        panic!("Failed to take model");
    }

    // Wait another 0.6s (total 1.2s from start, but only 0.6s from last use)
    std::thread::sleep(Duration::from_millis(600));

    scheduler.check_expiration();

    assert!(scheduler.has_model(), "Model should NOT be expired yet");

    // Wait another 0.5s (total > 1.0s from last use)
    std::thread::sleep(Duration::from_millis(500));

    scheduler.check_expiration();
    assert!(!scheduler.has_model(), "Model should be expired now");
}
