use std::sync::atomic::{AtomicBool, Ordering};

// Глобальный флаг отмены генерации (разделяем с модулем stream)
pub(crate) static CANCEL_GENERATION: AtomicBool = AtomicBool::new(false);

pub fn cancel_generation_cmd() -> Result<(), String> {
    CANCEL_GENERATION.store(true, Ordering::SeqCst);
    Ok(())
}


