use oxide_lib::core::config::SamplingOptions;

#[test]
fn test_sampling_options_default() {
    let options = SamplingOptions::new();
    assert_eq!(options.temperature, 0.7);
    assert_eq!(options.top_p, Some(0.9));
    assert_eq!(options.top_k, Some(20));
    assert_eq!(options.min_p, Some(0.0));
    assert_eq!(options.seed, None);
    assert_eq!(options.repeat_penalty, Some(1.1));
    assert_eq!(options.repeat_last_n, 64);
}

#[test]
fn test_sampling_options_conservative() {
    let options = SamplingOptions::conservative();
    assert_eq!(options.temperature, 0.2);
    assert_eq!(options.top_p, Some(0.8));
    assert_eq!(options.top_k, Some(10));
    assert_eq!(options.min_p, Some(0.0));
    assert_eq!(options.seed, None);
    assert_eq!(options.repeat_penalty, Some(1.2));
    assert_eq!(options.repeat_last_n, 64);
}

#[test]
fn test_sampling_options_creative() {
    let options = SamplingOptions::creative();
    assert_eq!(options.temperature, 0.9);
    assert_eq!(options.top_p, Some(0.95));
    assert_eq!(options.top_k, Some(50));
    assert_eq!(options.min_p, Some(0.0));
    assert_eq!(options.seed, None);
    assert_eq!(options.repeat_penalty, Some(1.05));
    assert_eq!(options.repeat_last_n, 64);
}

#[test]
fn test_sampling_options_argmax() {
    let options = SamplingOptions::argmax();
    assert_eq!(options.temperature, 0.0);
    assert_eq!(options.top_p, None);
    assert_eq!(options.top_k, None);
    assert_eq!(options.min_p, None);
    assert_eq!(options.seed, None);
    assert_eq!(options.repeat_penalty, Some(1.1));
    assert_eq!(options.repeat_last_n, 64);
}

#[test]
fn test_sampling_options_effective_seed() {
    let mut options = SamplingOptions::new();
    assert_eq!(options.effective_seed(), 42);

    options.seed = Some(12345);
    assert_eq!(options.effective_seed(), 12345);
}

#[test]
fn test_sampling_options_should_apply_repeat_penalty() {
    let mut options = SamplingOptions::new();

    // Default should apply repeat penalty
    assert!(options.should_apply_repeat_penalty());

    // No repeat penalty
    options.repeat_penalty = None;
    assert!(!options.should_apply_repeat_penalty());

    // Repeat penalty of 1.0 (no penalty)
    options.repeat_penalty = Some(1.0);
    assert!(!options.should_apply_repeat_penalty());

    // Repeat penalty > 1.0
    options.repeat_penalty = Some(1.2);
    assert!(options.should_apply_repeat_penalty());
}
