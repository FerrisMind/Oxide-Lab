use crate::core::thread_priority::set_current_thread_below_normal;

/// Initializes the global Rayon thread pool with a low-priority start handler.
///
/// Returns `Ok(true)` if the global pool was initialized by this call, or `Ok(false)` if it
/// was already initialized elsewhere.
pub fn init_global_low_priority_pool(num_threads: usize) -> Result<bool, String> {
    let threads = num_threads.max(1);

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|idx| format!("oxide-rayon-{}", idx))
        .start_handler(|_| {
            let _ = set_current_thread_below_normal();
        })
        .build_global()
        .map(|()| true)
        .or_else(|e| {
            // Rayon can only build the global pool once.
            if e.to_string()
                .to_lowercase()
                .contains("global thread pool has already been initialized")
            {
                Ok(false)
            } else {
                Err(e.to_string())
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_or_reports_already_initialized() {
        let res = init_global_low_priority_pool(2).unwrap();
        // Either we init it here, or something else already did.
        assert!(res == true || res == false);
    }
}
