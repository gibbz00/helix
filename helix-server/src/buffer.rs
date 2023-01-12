use helix_core::indent::IndentStyle;
use helix_core::{
    encoding,
    history::{History, State, UndoKind},
    indent::{auto_detect_indent_style, IndentStyle},
    line_ending::auto_detect_line_ending,
    syntax::{self, LanguageConfiguration},
    ChangeSet, Diagnostic, LineEnding, Rope, RopeBuilder, Selection, Syntax, Transaction,
    DEFAULT_LINE_ENDING,
};

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

pub struct Buffer {
    pub buffer_id: BufferID,
    text: Rope,
    path: Option<PathBuf>,
    encoding: &'static encoding::Encoding,
    pub indent_style: IndentStyle,
    pub line_ending: LineEnding,
    syntax: Option<Syntax>,
    /// Corresponding language scope name. Usually `source.<lang>`.
    pub(crate) language: Option<Arc<LanguageConfiguration>>,
    /// Pending changes since last history commit.
    changes: ChangeSet,
    /// State at last commit. Used for calculating reverts.
    old_state: Option<State>,
    /// Undo tree.
    // It can be used as a cell where we will take it out to get some parts of the history and put
    // it back as it separated from the edits. We could split out the parts manually but that will
    // be more troublesome.
    pub history: Cell<History>,
    pub savepoint: Option<Transaction>,
    last_saved_revision: usize,
    version: usize,
    pub(crate) modified_since_accessed: bool,
    diagnostics: Vec<Diagnostic>,
    language_server: Option<Arc<helix_lsp::Client>>,
    diff_handle: Option<DiffHandle>,
}

impl Buffer {
    pub fn from(text: Rope, encoding: Option<&'static encoding::Encoding>) -> Self {
        let encoding = encoding.unwrap_or(encoding::UTF_8);
        let changes = ChangeSet::new(&text);
        let old_state = None;

        Self {
            buffer_id: BufferID::default(),
            path: None,
            encoding,
            text,
            indent_style: DEFAULT_INDENT,
            line_ending: DEFAULT_LINE_ENDING,
            language: None,
            changes,
            old_state,
            diagnostics: Vec::new(),
            version: 0,
            history: Cell::new(History::default()),
            savepoint: None,
            last_saved_revision: 0,
            modified_since_accessed: false,
            language_server: None,
            diff_handle: None,
        }
    }
}
    