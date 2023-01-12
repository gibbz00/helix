use helix_core::indent::IndentStyle;

/// 8kB of buffer space for encoding and decoding `Rope`s.
pub const BUF_SIZE: usize = 8192;

pub const DEFAULT_INDENT: IndentStyle = IndentStyle::Tabs;

pub const SCRATCH_BUFFER_NAME: &str = "[scratch]";
/// A snapshot of the text of a document that we want to write out to disk
#[derive(Debug, Clone)]
pub struct DocumentSavedEvent {
    pub revision: usize,
    pub buffer_id: BufferID,
    pub path: PathBuf,
    pub text: Rope,
}

pub type DocumentSavedEventResult = Result<DocumentSavedEvent, anyhow::Error>;
pub type DocumentSavedEventFuture = BoxFuture<'static, DocumentSavedEventResult>;

// uses NonZeroUsize so Option<BufferID> use one byte rather than two
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BufferID(NonZeroUsize);

impl Default for BufferID {
    fn default() -> BufferID {
        BufferID(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

impl std::fmt::Display for BufferID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}