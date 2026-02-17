pub mod reader;
pub mod registry;
pub mod types;
pub mod writer;

pub use reader::SpfReader;
pub use registry::SpfRegistry;
pub use types::*;
pub use writer::SpfWriter;

/// Progress callback type for unpack/pack operations
/// Arguments: (current_index, total_count, filename)
pub type ProgressCallback<'a> = Option<&'a dyn Fn(usize, usize, &str)>;
