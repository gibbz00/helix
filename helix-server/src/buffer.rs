use helix_core::indent::IndentStyle;

/// 8kB of buffer space for encoding and decoding `Rope`s.
pub const BUF_SIZE: usize = 8192;

pub const DEFAULT_INDENT: IndentStyle = IndentStyle::Tabs;

pub const SCRATCH_BUFFER_NAME: &str = "[scratch]";
