pub mod stream;
pub mod cancel;
pub mod sampling;
pub mod minp;
pub mod emit;
pub mod ctx;

pub use stream::generate_stream_cmd;
pub use cancel::cancel_generation_cmd;
// Back-compat re-export for tests/examples
pub use stream::build_prompt_with_template;


