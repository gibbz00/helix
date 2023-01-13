mod command_multiplier;
mod jump_list;

use self::command_multiplier::CommandMultiplier;

use crate::{
    event_handler::EventHandler,
    lists,
    align_view,
    clipboard::{get_clipboard_provider, ClipboardProvider},
    buffer_mirror::{DocumentSavedEventFuture, DocumentSavedEventResult},
    mode::Mode,
    graphics::{CursorKind, Rect},
    info::Info,
    input::KeyEvent,
    theme::{self, Theme},
    tree::{self, Tree},
    Align, BufferMirror, BufferView, buffer_view::BufferViewID, config::Config, jump_list::JumpList,
};
use helix_server::buffer::BufferID;
use helix_core::{
    Position,
    diagnostic::Severity,
    register::Registers,
    auto_pairs::AutoPairs,
    syntax::{self, AutoPairConfig},
    Change,
};
use helix_dap as dap;
use helix_lsp::{lsp, Call};
use helix_vcs::DiffProviderRegistry;

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    io::stdin,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use anyhow::{anyhow, bail, Error};
use arc_swap::access::{DynAccess, DynGuard};
use futures_util::{stream::select_all::SelectAll, future, StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::{
    sync::{mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},Notify, RwLock,},
    time::{sleep, Duration, Instant, Sleep},
};

pub struct Motion(pub Box<dyn Fn(&mut UITree)>);
impl Motion {
    pub fn run(&self, e: &mut UITree) {
        (self.0)(e)
    }
}
impl std::fmt::Debug for Motion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("motion")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Breakpoint {
    pub id: Option<usize>,
    pub verified: bool,
    pub message: Option<String>,

    pub line: usize,
    pub column: Option<usize>,
    pub condition: Option<String>,
    pub hit_condition: Option<String>,
    pub log_message: Option<String>,
}

use futures_util::stream::{Flatten, Once};

pub struct UITree {
    pub jump_list: JumpList,
    /// Current editing mode.
    pub mode: Mode,
    pub tree: Tree,
    pub keymap: Keymap,
    // New vec is created when traversing to a sticky keytrie
    // Each Vec in Vec is in other words a sticky level.
    pub pending_keys: Vec<Vec<KeyEvent>>,
    pub command_multiplier: CommandMultiplier,
    pub event_handler: EventHandler,
    pub next_buffer_id: BufferID,
    pub buffer_mirrors: BTreeMap<BufferID, BufferMirror>,
    pub lists: Vec<Lists>,

    // We Flatten<> to resolve the inner DocumentSavedEventFuture. For that we need a stream of streams, hence the Once<>.
    // https://stackoverflow.com/a/66875668
    pub saves: HashMap<BufferID, UnboundedSender<Once<DocumentSavedEventFuture>>>,
    pub save_queue: SelectAll<Flatten<UnboundedReceiverStream<Once<DocumentSavedEventFuture>>>>,
    pub write_count: usize,

    pub selected_register: Option<char>,
    pub registers: Registers,
    pub macro_recording: Option<(char, Vec<KeyEvent>)>,
    pub macro_replaying: Vec<char>,
    pub language_servers: helix_lsp::Registry,
    pub diagnostics: BTreeMap<lsp::Url, Vec<lsp::Diagnostic>>,
    pub diff_providers: DiffProviderRegistry,

    pub debugger: Option<dap::Client>,
    pub debugger_events: SelectAll<UnboundedReceiverStream<dap::Payload>>,
    pub breakpoints: HashMap<PathBuf, Vec<Breakpoint>>,

    pub clipboard_provider: Box<dyn ClipboardProvider>,

    pub syn_loader: Arc<syntax::Loader>,
    pub theme_loader: Arc<theme::Loader>,
    /// last_theme is used for theme previews. We store the current theme here,
    /// and if previewing is cancelled, we can return to it.
    pub last_theme: Option<Theme>,
    /// The currently applied editor theme. While previewing a theme, the previewed theme
    /// is set here.
    pub theme: Theme,
    pub last_line_number: Option<usize>,
    pub status_msg: Option<(Cow<'static, str>, Severity)>,
    pub autoinfo: Option<Info>,

    pub config: Box<dyn DynAccess<Config>>,
    pub auto_pairs: Option<AutoPairs>,

    pub idle_timer: Pin<Box<Sleep>>,
    pub last_motion: Option<Motion>,

    pub last_completion: Option<CompleteAction>,

    pub exit_code: i32,

    pub config_events: (UnboundedSender<ConfigEvent>, UnboundedReceiver<ConfigEvent>),
    /// Allows asynchronous tasks to control the rendering
    /// The `Notify` allows asynchronous tasks to request the editor to perform a redraw
    /// The `RwLock` blocks the editor from performing the render until an exclusive lock can be aquired
    pub redraw_handle: RedrawHandle,
    pub needs_redraw: bool,
}

pub type RedrawHandle = (Arc<Notify>, Arc<RwLock<()>>);

#[derive(Debug)]
pub enum EditorEvent {
    DocumentSaved(DocumentSavedEventResult),
    ConfigEvent(ConfigEvent),
    LanguageServerMessage((usize, Call)),
    DebuggerEvent(dap::Payload),
    IdleTimer,
}

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    Refresh,
    Update(Box<Config>),
}

enum ThemeAction {
    Set,
    Preview,
}

#[derive(Debug, Clone)]
pub struct CompleteAction {
    pub trigger_offset: usize,
    pub changes: Vec<Change>,
}

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Load,
    Replace,
    HorizontalSplit,
    VerticalSplit,
}

/// Error thrown on failed document closed
pub enum CloseError {
    /// Document doesn't exist
    DoesNotExist,
    /// Buffer is modified
    BufferModified(String),
    /// Document failed to save
    SaveError(anyhow::Error),
}

impl UITree {
    pub fn new(
        mut area: Rect,
        theme_loader: Arc<theme::Loader>,
        syn_loader: Arc<syntax::Loader>,
        config: Box<dyn DynAccess<Config>>,
    ) -> Self {
        let conf = config.load();
        let auto_pairs = (&conf.auto_pairs).into();

        // HAXX: offset the render area height by 1 to account for prompt/commandline
        area.height -= 1;

        Self {
            mode: Mode::Normal,
            tree: Tree::new(area),
            jump_list: JumpList::new((buffer_id, Selection::point(0))), // TODO: use actual sel
            event_handler: EventHandler::start(&self),
            keymap: config.keys,
            pending_keys: vec![Vec::new()],
            command_multiplier: CommandMultiplier::new(),
            next_buffer_id: BufferID::default(),
            buffer_mirrors: BTreeMap::new(),
            saves: HashMap::new(),
            save_queue: SelectAll::new(),
            write_count: 0,
            selected_register: None,
            macro_recording: None,
            macro_replaying: Vec::new(),
            theme: theme_loader.default(),
            language_servers: helix_lsp::Registry::new(),
            diagnostics: BTreeMap::new(),
            diff_providers: DiffProviderRegistry::default(),
            debugger: None,
            debugger_events: SelectAll::new(),
            breakpoints: HashMap::new(),
            syn_loader,
            theme_loader,
            last_theme: None,
            last_line_number: None,
            registers: Registers::default(),
            clipboard_provider: get_clipboard_provider(),
            status_msg: None,
            autoinfo: None,
            idle_timer: Box::pin(sleep(conf.idle_timeout)),
            last_motion: None,
            last_completion: None,
            config,
            auto_pairs,
            exit_code: 0,
            config_events: unbounded_channel(),
            redraw_handle: Default::default(),
            needs_redraw: false,
        }
    }

    /// Store a jump on a buffer_mirror to the jumplist.
    fn push_jump(buffer_mirror: &BufferMirror) {
        self.jumps.push((buffer_mirror.buffer_id(), buffer_mirror.selection().clone()));
    }

    pub fn remove_buffer_from_histories(&mut self, buffer_id: &BufferID, buffer_view_id: &BufferViewID) {
        self.jump_list.remove(buffer_id);
        self.tree.get(buffer_view_id).buffer_access_history.retain(|buff_id| buff_id != buffer_id);
    }

    /// Current editing mode for the [`Editor`].
    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn config(&self) -> DynGuard<Config> {
        self.config.load()
    }

    /// Call if the config has changed to let the editor update all
    /// relevant members.
    pub fn refresh_config(&mut self) {
        let config = self.config();
        self.auto_pairs = (&config.auto_pairs).into();
        self.reset_idle_timer();
    }

    pub fn clear_idle_timer(&mut self) {
        // equivalent to internal Instant::far_future() (30 years)
        self.idle_timer
            .as_mut()
            .reset(Instant::now() + Duration::from_secs(86400 * 365 * 30));
    }

    pub fn reset_idle_timer(&mut self) {
        let config = self.config();
        self.idle_timer
            .as_mut()
            .reset(Instant::now() + Duration::from_millis(config.idle_timout));
    }

    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    #[inline]
    pub fn set_status<T: Into<Cow<'static, str>>>(&mut self, status: T) {
        let status = status.into();
        log::debug!("editor status: {}", status);
        self.status_msg = Some((status, Severity::Info));
    }

    #[inline]
    pub fn set_error<T: Into<Cow<'static, str>>>(&mut self, error: T) {
        let error = error.into();
        log::error!("editor error: {}", error);
        self.status_msg = Some((error, Severity::Error));
    }

    #[inline]
    pub fn get_status(&self) -> Option<(&Cow<'static, str>, &Severity)> {
        self.status_msg.as_ref().map(|(status, sev)| (status, sev))
    }

    /// Returns true if the current status is an error
    #[inline]
    pub fn is_err(&self) -> bool {
        self.status_msg
            .as_ref()
            .map(|(_, sev)| *sev == Severity::Error)
            .unwrap_or(false)
    }

    pub fn unset_theme_preview(&mut self) {
        if let Some(last_theme) = self.last_theme.take() {
            self.set_theme(last_theme);
        }
        // None likely occurs when the user types ":theme" and then exits before previewing
    }

    pub fn set_theme_preview(&mut self, theme: Theme) {
        self.set_theme_impl(theme, ThemeAction::Preview);
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.set_theme_impl(theme, ThemeAction::Set);
    }

    fn set_theme_impl(&mut self, theme: Theme, preview: ThemeAction) {
        // `ui.selection` is the only scope required to be able to render a theme.
        if theme.find_scope_index("ui.selection").is_none() {
            self.set_error("Invalid theme: `ui.selection` required");
            return;
        }

        let scopes = theme.scopes();
        self.syn_loader.set_scopes(scopes.to_vec());

        match preview {
            ThemeAction::Preview => {
                let last_theme = std::mem::replace(&mut self.theme, theme);
                // only insert on first preview: this will be the last theme the user has saved
                self.last_theme.get_or_insert(last_theme);
            }
            ThemeAction::Set => {
                self.last_theme = None;
                self.theme = theme;
            }
        }

        self._refresh();
    }

    /// Refreshes the language server for a given buffer
    pub fn refresh_language_server(&mut self, buffer_id: BufferID) -> Option<()> {
        let buffer = self.buffer_mirrors.get_mut(&buffer_id)?;
        Self::launch_language_server(&mut self.language_servers, buffer)
    }

    /// Launch a language server for a given buffer
    fn launch_language_server(ls: &mut helix_lsp::Registry, buffer: &mut BufferMirror) -> Option<()> {
        // if doc doesn't have a URL it's a scratch buffer, ignore it
        let buffer_url = buffer.url()?;

        // try to find a language server based on the language name
        let language_server = buffer.language.as_ref().and_then(|language| {
            ls.get(language, buffer.path())
                .map_err(|e| {
                    log::error!(
                        "Failed to initialize the LSP for `{}` {{ {} }}",
                        language.scope(),
                        e
                    )
                })
                .ok()
                .flatten()
        });
        if let Some(language_server) = language_server {
            // only spawn a new lang server if the servers aren't the same
            if Some(language_server.id()) != buffer.language_server().map(|server| server.id()) {
                if let Some(language_server) = buffer.language_server() {
                    tokio::spawn(language_server.text_document_did_close(buffer.identifier()));
                }

                let language_id = buffer.language_id().map(ToOwned::to_owned).unwrap_or_default();

                // TODO: this now races with on_init code if the init happens too quickly
                tokio::spawn(language_server.text_document_did_open(
                    buffer_url,
                    buffer.version(),
                    buffer.text(),
                    language_id,
                ));

                buffer.set_language_server(Some(language_server));
            }
        }
        Some(())
    }

    fn _refresh(&mut self) {
        let config = self.config();
        for (view, _) in self.tree.views_mut() {
            let buffer_mirror = self.buffer_mirrors.get_mut(&view.buffer_mirror_id).unwrap()
            view.sync_changes(buffer_mirror);
            view.ensure_cursor_in_view(buffer_mirror, config.scrolloff)
        }
    }

    fn replace_document_in_view(&mut self, current_view: BufferViewID, doc_id: BufferID) {
        let view = self.tree.get_mut(current_view);
        view.buffer_mirror_id = doc_id;
        view.offset = Position::default();


        let buffer_mirror = self.buffer_mirrors.get_mut(&doc_id).unwrap();
        view.sync_changes(buffer_mirror);

        align_view(buffer_mirror, view, Align::Center);
    }

    pub fn switch(&mut self, id: BufferID, action: Action) {
        use crate::tree::Layout;

        if !self.buffer_mirrors.contains_key(&id) {
            log::error!("cannot switch to document that does not exist (anymore)");
            return;
        }

        self.enter_normal_mode();

        match action {
            Action::Replace => {
                let buffer_mirror = current!(self);
                // If the current view is an empty scratch buffer and is not displayed in any other views, delete it.
                // Boolean value is determined before the call to `view_mut` because the operation requires a borrow
                // of `self.tree`, which is mutably borrowed when `view_mut` is called.
                let remove_empty_scratch = !buffer_mirror.is_modified()
                    // If the buffer has no path and is not modified, it is an empty scratch buffer.
                    && buffer_mirror.path().is_none()
                    // If the buffer we are changing to is not this buffer
                    && id != buffer_mirror.buffer_id
                    // Ensure the buffer is not displayed in any other splits.
                    && !self
                        .tree
                        .traverse()
                        .any(|(_, v)| v.buffer_mirror_id == buffer_mirror.buffer_id
                            && v.view_id != self.tree.get(self.tree.focus.view_id));

                let buffer_mirror = current_mut!(self);
                let view = buffer_view_mut!(self);
                let view_id = view.view_id;

                // Append any outstanding changes to history in the old document.
                buffer_mirror.append_changes_to_history(view);

                if remove_empty_scratch {
                    // Copy `doc.id` into a variable before calling `self.documents.remove`, which requires a mutable
                    // borrow, invalidating direct access to `doc.id`.
                    let id = buffer_mirror.buffer_id;
                    self.buffer_mirrors.remove(&id);

                    for (view, _) in self.tree.views_mut() {
                        self.remove_buffer_from_histories(&id, &view.view_id);
                    }
                } else {
                    let jump = (buffer_mirror.id(), buffer_mirror.selection().clone());
                    buffer_mirror.jumps.push(jump);
                    if buffer_mirror.buffer_id != id {
                        view.add_to_access_history(view.buffer_mirror_id);
                        // Set last modified doc if modified and last modified doc is different
                        if std::mem::take(&mut buffer_mirror.modified_since_accessed)
                            && view.last_modified_buffers[0] != Some(view.buffer_mirror_id)
                        {
                            view.last_modified_buffers = [Some(view.buffer_mirror_id), view.last_modified_buffers[0]];
                        }
                    }
                }

                self.replace_document_in_view(view_id, id);

                return;
            }
            Action::Load => {
                // TODO: what purpose does load solve now given that
                // ensure_current_cursor has been deprecated? 
            }
            Action::HorizontalSplit | Action::VerticalSplit => {
                // copy the current view, unless there is no view yet
                let view = self
                    .tree
                    .try_get(self.tree.focus)
                    .filter(|v| id == v.buffer_mirror_id) // Different Document
                    .cloned()
                    .unwrap_or_else(|| BufferView::new(id, self.config().gutters.clone()));
                let view_id = self.tree.split(
                    view,
                    match action {
                        Action::HorizontalSplit => Layout::Horizontal,
                        Action::VerticalSplit => Layout::Vertical,
                        _ => unreachable!(),
                    },
                );
            }
        }

        self._refresh();
    }

    /// Generate an id for a new document and register it.
    fn new_document(&mut self, mut doc: BufferMirror) -> BufferID {
        let id = self.next_buffer_id;
        // Safety: adding 1 from 1 is fine, probably impossible to reach usize max
        self.next_buffer_id =
            BufferID(unsafe { NonZeroUsize::new_unchecked(self.next_buffer_id.0.get() + 1) });
        doc.buffer_id = id;
        self.buffer_mirrors.insert(id, doc);

        let (save_sender, save_receiver) = tokio::sync::mpsc::unbounded_channel();
        self.saves.insert(id, save_sender);

        let stream = UnboundedReceiverStream::new(save_receiver).flatten();
        self.save_queue.push(stream);

        id
    }

    fn new_file_from_document(&mut self, action: Action, doc: BufferMirror) -> BufferID {
        let id = self.new_document(doc);
        self.switch(id, action);
        id
    }

    pub fn new_file(&mut self, action: Action) -> BufferID {
        self.new_file_from_document(action, BufferMirror::default())
    }

    pub fn new_file_from_stdin(&mut self, action: Action) -> Result<BufferID, Error> {
        let (rope, encoding) = crate::buffer_mirror::from_reader(&mut stdin(), None)?;
        Ok(self.new_file_from_document(action, BufferMirror::from(rope, Some(encoding))))
    }

    // ??? possible use for integration tests
    pub fn open(&mut self, path: &Path, action: Action) -> Result<BufferID, Error> {
        let path = helix_core::path::get_canonicalized_path(path)?;
        let id = self.document_by_path(&path).map(|doc| doc.buffer_id);

        let id = if let Some(id) = id {
            id
        } else {
            let mut doc = BufferMirror::open(&path, None, Some(self.syn_loader.clone()))?;

            let _ = Self::launch_language_server(&mut self.language_servers, &mut doc);
            if let Some(diff_base) = self.diff_providers.get_diff_base(&path) {
                doc.set_diff_base(diff_base, self.redraw_handle.clone());
            }
            self.new_document(doc)
        };

        self.switch(id, action);
        Ok(id)
    }

    pub fn close(&mut self, id: BufferViewID) {
        self.tree.remove(id);
        self._refresh();
    }

    pub fn close_document(&mut self, doc_id: BufferID, force: bool) -> Result<(), CloseError> {
        let doc = match self.buffer_mirrors.get_mut(&doc_id) {
            Some(doc) => doc,
            None => return Err(CloseError::DoesNotExist),
        };
        if !force && doc.is_modified() {
            return Err(CloseError::BufferModified(doc.display_name().into_owned()));
        }

        // This will also disallow any follow-up writes
        self.saves.remove(&doc_id);

        if let Some(language_server) = doc.language_server() {
            // TODO: track error
            tokio::spawn(language_server.text_document_did_close(doc.identifier()));
        }

        enum Action {
            Close(BufferViewID),
            ReplaceDoc(BufferViewID, BufferID),
        }

        let actions: Vec<Action> = self
            .tree
            .views_mut()
            .filter_map(|(view, _focus)| {
                // function moved here to ui_tree as remove_buffer_from_history
                self.remove_buffer_from_histories(&doc_id, &view.view_id);

                if view.buffer_mirror_id == doc_id {
                    // something was previously open in the view, switch to previous doc
                    if let Some(prev_doc) = view.buffer_access_history.pop() {
                        Some(Action::ReplaceDoc(view.view_id, prev_doc))
                    } else {
                        // only the document that is being closed was in the view, close it
                        Some(Action::Close(view.view_id))
                    }
                } else {
                    None
                }
            })
            .collect();

        for action in actions {
            match action {
                Action::Close(view_id) => {
                    self.close(view_id);
                }
                Action::ReplaceDoc(view_id, doc_id) => {
                    self.replace_document_in_view(view_id, doc_id);
                }
            }
        }

        self.buffer_mirrors.remove(&doc_id);

        // If the document we removed was visible in all views, we will have no more views. We don't
        // want to close the editor just for a simple buffer close, so we need to create a new view
        // containing either an existing document, or a brand new document.
        if self.tree.views().next().is_none() {
            let doc_id = self
                .buffer_mirrors
                .iter()
                .map(|(&doc_id, _)| doc_id)
                .next()
                .unwrap_or_else(|| self.new_document(BufferMirror::default()));
            let view = BufferView::new(doc_id, self.config().gutters.clone());
            let view_id = self.tree.insert(view);
        }

        self._refresh();

        Ok(())
    }

    pub fn save<P: Into<PathBuf>>(
        &mut self,
        doc_id: BufferID,
        path: Option<P>,
        force: bool,
    ) -> anyhow::Result<()> {
        // convert a channel of futures to pipe into main queue one by one
        // via stream.then() ? then push into main future

        let path = path.map(|path| path.into());
        let buffer_mirror = self.buffer_mirrors.get_mut(&doc_id).unwrap()
        let future = buffer_mirror.save(path, force)?;

        use futures_util::stream;

        self.saves
            .get(&doc_id)
            .ok_or_else(|| anyhow::format_err!("saves are closed for this document!"))?
            .send(stream::once(Box::pin(future)))
            .map_err(|err| anyhow!("failed to send save event: {}", err))?;

        self.write_count += 1;

        Ok(())
    }

    pub fn resize(&mut self, area: Rect) {
        if self.tree.resize(area) {
            self._refresh();
        };
    }

    pub fn focus(&mut self, view_id: BufferViewID) {
        let prev_id = std::mem::replace(&mut self.tree.focus, view_id);

        // if leaving the view: mode should reset and the cursor should be
        // within view
        if prev_id != view_id {
            self.enter_normal_mode();
            self.ensure_cursor_in_view(view_id);

            // Update jumplist selections with new document changes.
            for (view, _focused) in self.tree.views_mut() {
                let buffer_mirror = self.buffer_mirrors.get_mut(&view.buffer_mirror_id).unwrap()
                view.sync_changes(buffer_mirror);
            }
        }
    }

    pub fn focus_next(&mut self) {
        self.focus(self.tree.next());
    }

    pub fn focus_direction(&mut self, direction: tree::Direction) {
        let current_view = self.tree.focus;
        if let Some(id) = self.tree.find_split_in_direction(current_view, direction) {
            self.focus(id)
        }
    }

    pub fn swap_split_in_direction(&mut self, direction: tree::Direction) {
        self.tree.swap_split_in_direction(direction);
    }

    pub fn transpose_view(&mut self) {
        self.tree.transpose();
    }

    pub fn should_close(&self) -> bool {
        self.tree.is_empty()
    }

    pub fn ensure_cursor_in_view(&mut self, id: BufferViewID) {
        let config = self.config();
        let view = self.tree.get_mut(id);
        let doc = &self.buffer_mirrors[&view.buffer_mirror_id];
        view.ensure_cursor_in_view(doc, config.scrolloff)
    }

    #[inline]
    pub fn document(&self, id: BufferID) -> Option<&BufferMirror> {
        self.buffer_mirrors.get(&id)
    }

    #[inline]
    pub fn document_mut(&mut self, id: BufferID) -> Option<&mut BufferMirror> {
        self.buffer_mirrors.get_mut(&id)
    }

    #[inline]
    pub fn documents(&self) -> impl Iterator<Item = &BufferMirror> {
        self.buffer_mirrors.values()
    }

    #[inline]
    pub fn documents_mut(&mut self) -> impl Iterator<Item = &mut BufferMirror> {
        self.buffer_mirrors.values_mut()
    }

    pub fn document_by_path<P: AsRef<Path>>(&self, path: P) -> Option<&BufferMirror> {
        self.documents()
            .find(|doc| doc.path().map(|p| p == path.as_ref()).unwrap_or(false))
    }

    pub fn document_by_path_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut BufferMirror> {
        self.documents_mut()
            .find(|doc| doc.path().map(|p| p == path.as_ref()).unwrap_or(false))
    }

    pub fn cursor(&self) -> (Option<Position>, CursorKind) {
        let config = self.config();
        let buffer_mirror = current!(self);
        let view = buffer_view!(self);
        let cursor = buffer_mirror
            .selection()
            .primary()
            .cursor(buffer_mirror.text().slice(..));
        if let Some(mut pos) = view.screen_coords_at_pos(buffer_mirror, buffer_mirror.text().slice(..), cursor) {
            let inner = view.inner_area(buffer_mirror);
            pos.col += inner.x as usize;
            pos.row += inner.y as usize;
            let cursorkind = config.cursor_shape.from_mode(self.mode);
            (Some(pos), cursorkind)
        } else {
            (None, CursorKind::default())
        }
    }

    /// Closes language servers with timeout. The default timeout is 10000 ms, use
    /// `timeout` parameter to override this.
    pub async fn close_language_servers(
        &self,
        timeout: Option<u64>,
    ) -> Result<(), tokio::time::error::Elapsed> {
        tokio::time::timeout(
            Duration::from_millis(timeout.unwrap_or(3000)),
            future::join_all(
                self.language_servers
                    .iter_clients()
                    .map(|client| client.force_shutdown()),
            ),
        )
        .await
        .map(|_| ())
    }

    pub async fn wait_event(&mut self) -> EditorEvent {
        // the loop only runs once or twice and would be better implemented with a recursion + const generic
        // however due to limitations with async functions that can not be implemented right now
        loop {
            tokio::select! {
                biased;

                Some(event) = self.save_queue.next() => {
                    self.write_count -= 1;
                    return EditorEvent::DocumentSaved(event)
                }
                Some(config_event) = self.config_events.1.recv() => {
                    return EditorEvent::ConfigEvent(config_event)
                }
                Some(message) = self.language_servers.incoming.next() => {
                    return EditorEvent::LanguageServerMessage(message)
                }
                Some(event) = self.debugger_events.next() => {
                    return EditorEvent::DebuggerEvent(event)
                }

                _ = self.redraw_handle.0.notified() => {
                    if  !self.needs_redraw{
                        self.needs_redraw = true;
                        let timeout = Instant::now() + Duration::from_millis(96);
                        if timeout < self.idle_timer.deadline(){
                            self.idle_timer.as_mut().reset(timeout)
                        }
                    }
                }

                _ = &mut self.idle_timer  => {
                    return EditorEvent::IdleTimer
                }
            }
        }
    }

    pub async fn flush_writes(&mut self) -> anyhow::Result<()> {
        while self.write_count > 0 {
            if let Some(save_event) = self.save_queue.next().await {
                self.write_count -= 1;

                let save_event = match save_event {
                    Ok(event) => event,
                    Err(err) => {
                        self.set_error(err.to_string());
                        bail!(err);
                    }
                };
                let buffer_mirror = self.buffer_mirrors.get_mut(&save_event.buffer_id).unwrap()
                buffer_mirror.set_last_saved_revision(save_event.revision);
            }
        }

        Ok(())
    }

    /// Switches the editor into normal mode.
    pub fn enter_normal_mode(&mut self) {
        use helix_core::{graphemes, Range};

        if self.mode == Mode::Normal {
            return;
        }

        self.mode = Mode::Normal;
        let buffer_mirror = current_mut!(self);

        try_restore_indent(buffer_mirror);

        // if leaving append mode, move cursor back by 1
        if buffer_mirror.restore_cursor {
            let text = buffer_mirror.text().slice(..);
            let selection = buffer_mirror.selection().clone().transform(|range| {
                Range::new(
                    range.from(),
                    graphemes::prev_grapheme_boundary(text, range.to()),
                )
            });

            buffer_mirror.set_selection(selection);
            buffer_mirror.restore_cursor = false;
        }
    }
}

fn try_restore_indent(doc: &mut BufferMirror) {
    use helix_core::{
        chars::char_is_whitespace, line_ending::line_end_char_index, Operation, Transaction,
    };

    fn inserted_a_new_blank_line(changes: &[Operation], pos: usize, line_end_pos: usize) -> bool {
        if let [Operation::Retain(move_pos), Operation::Insert(ref inserted_str), Operation::Retain(_)] =
            changes
        {
            move_pos + inserted_str.len() == pos
                && inserted_str.starts_with('\n')
                && inserted_str.chars().skip(1).all(char_is_whitespace)
                && pos == line_end_pos // ensure no characters exists after current position
        } else {
            false
        }
    }

    let doc_changes = doc.changes().changes();
    let text = doc.text().slice(..);
    let range = doc.selection().primary();
    let pos = range.cursor(text);
    let line_end_pos = line_end_char_index(&text, range.cursor_line(text));

    if inserted_a_new_blank_line(doc_changes, pos, line_end_pos) {
        // Removes tailing whitespaces.
        let transaction =
            Transaction::change_by_selection(doc.text(), doc.selection(), |range| {
                let line_start_pos = text.line_to_char(range.cursor_line(text));
                (line_start_pos, pos, None)
            });
        crate::apply_transaction(&transaction, doc);
    }
}
