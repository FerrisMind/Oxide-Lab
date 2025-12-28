pub mod cancel;
pub mod ctx;
pub mod emit;
pub mod minp;
pub mod sampling;
pub mod stream;
pub mod thinking_parser;

pub use cancel::cancel_generation_cmd;
pub use stream::generate_stream_cmd;
// Back-compat re-export for tests/examples
pub use stream::build_prompt_with_template;
