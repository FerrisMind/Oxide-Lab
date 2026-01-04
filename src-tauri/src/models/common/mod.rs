//! Common utilities for model backends

pub mod flash_helpers;

pub use flash_helpers::{is_flash_attention_available, scaled_dot_product_attention};
