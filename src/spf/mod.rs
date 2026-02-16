pub mod types;
pub mod reader;
pub mod writer;
pub mod registry;

pub use types::*;
pub use reader::SpfReader;
pub use writer::SpfWriter;
pub use registry::SpfRegistry;

/// Progress callback type for unpack/pack operations
/// Arguments: (current_index, total_count, filename)
pub type ProgressCallback<'a> = Option<&'a dyn Fn(usize, usize, &str)>;
