use std::ops::Deref;

use crate::job::Job;

use super::*;

use helix_view::{
    buffer_view::BufferViewID,
    apply_transaction,
    editor::{Action, CloseError, ConfigEvent},
};
use ui::completers::{self, Completer};

use helix_server::buffer::BufferID;

fn quit(cx: &mut compositor::Context, args: &[Cow<str>], event: PromptEvent) -> anyhow::Result<()> {
    log::debug!("quitting...");

    if event != PromptEvent::Validate {
        return Ok(());
    }

    ensure!(args.is_empty(), ":quit takes no arguments");

    // last view and we have unsaved changes
    if cx.ui_tree.tree.views().count() == 1 {
        buffers_remaining_impl(cx.ui_tree)?
    }

    cx.block_try_flush_writes()?;
    cx.ui_tree.close(buffer_view!(cx.ui_tree).id);

    Ok(())
}

fn force_quit(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    ensure!(args.is_empty(), ":quit! takes no arguments");

    cx.block_try_flush_writes()?;
    cx.ui_tree.close(buffer_view!(cx.ui_tree).id);

    Ok(())
}

fn open(cx: &mut compositor::Context, args: &[Cow<str>], event: PromptEvent) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    ensure!(!args.is_empty(), "wrong argument count");
    for arg in args {
        let (path, pos) = args::parse_file(arg);
        let _ = cx.ui_tree.open(&path, Action::Replace)?;
        let buffer_mirror = current_mut!(cx.ui_tree);
        let view = buffer_view_mut!(cx.ui_tree);
        let pos = Selection::point(pos_at_coords(buffer_mirror.text().slice(..), pos, true));
        buffer_mirror.set_selection(pos);
        // does not affect opening a buffer without pos
        align_view(buffer_mirror, view, Align::Center);
    }
    Ok(())
}

fn buffer_close_by_ids_impl(
    cx: &mut compositor::Context,
    doc_ids: &[BufferID],
    force: bool,
) -> anyhow::Result<()> {
    cx.block_try_flush_writes()?;

    let (modified_ids, modified_names): (Vec<_>, Vec<_>) = doc_ids
        .iter()
        .filter_map(|&doc_id| {
            if let Err(CloseError::BufferModified(name)) = cx.ui_tree.close_document(doc_id, force) {
                Some((doc_id, name))
            } else {
                None
            }
        })
        .unzip();

    if let Some(first) = modified_ids.first() {
        let current = current!(cx.ui_tree);
        // If the current document is unmodified, and there are modified
        // documents, switch focus to the first modified doc.
        if !modified_ids.contains(&current.id()) {
            cx.ui_tree.switch(*first, Action::Replace);
        }
        bail!(
            "{} unsaved buffer(s) remaining: {:?}",
            modified_names.len(),
            modified_names
        );
    }

    Ok(())
}

fn buffer_gather_paths_impl(ui_tree: &mut UITree, args: &[Cow<str>]) -> Vec<BufferID> {
    // No arguments implies current document
    if args.is_empty() {
        let doc_id = buffer_view!(ui_tree).doc;
        return vec![doc_id];
    }

    let mut nonexistent_buffers = vec![];
    let mut document_ids = vec![];
    for arg in args {
        let doc_id = ui_tree.documents().find_map(|doc| {
            let arg_path = Some(Path::new(arg.as_ref()));
            if doc.path().map(|p| p.as_path()) == arg_path
                || doc.relative_path().as_deref() == arg_path
            {
                Some(doc.buffer_id())
            } else {
                None
            }
        });

        match doc_id {
            Some(doc_id) => document_ids.push(doc_id),
            None => nonexistent_buffers.push(format!("'{}'", arg)),
        }
    }

    if !nonexistent_buffers.is_empty() {
        ui_tree.set_error(format!(
            "cannot close non-existent buffers: {}",
            nonexistent_buffers.join(", ")
        ));
    }

    document_ids
}

fn buffer_close(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let document_ids = buffer_gather_paths_impl(cx.ui_tree, args);
    buffer_close_by_ids_impl(cx, &document_ids, false)
}

fn force_buffer_close(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let document_ids = buffer_gather_paths_impl(cx.ui_tree, args);
    buffer_close_by_ids_impl(cx, &document_ids, true)
}

fn buffer_gather_others_impl(editor: &mut UITree) -> Vec<BufferID> {
    let current_document = &current!(editor).id();
    editor
        .documents()
        .map(|doc| doc.buffer_id())
        .filter(|doc_id| doc_id != current_document)
        .collect()
}

fn buffer_close_others(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let document_ids = buffer_gather_others_impl(cx.ui_tree);
    buffer_close_by_ids_impl(cx, &document_ids, false)
}

fn force_buffer_close_others(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let document_ids = buffer_gather_others_impl(cx.ui_tree);
    buffer_close_by_ids_impl(cx, &document_ids, true)
}

fn buffer_gather_all_impl(editor: &mut UITree) -> Vec<BufferID> {
    editor.documents().map(|doc| doc.buffer_id()).collect()
}

fn buffer_close_all(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let document_ids = buffer_gather_all_impl(cx.ui_tree);
    buffer_close_by_ids_impl(cx, &document_ids, false)
}

fn force_buffer_close_all(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let document_ids = buffer_gather_all_impl(cx.ui_tree);
    buffer_close_by_ids_impl(cx, &document_ids, true)
}

fn buffer_next(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    goto_buffer(cx.ui_tree, Direction::Forward);
    Ok(())
}

fn buffer_previous(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    goto_buffer(cx.ui_tree, Direction::Backward);
    Ok(())
}

fn write_impl(
    cx: &mut compositor::Context,
    path: Option<&Cow<str>>,
    force: bool,
) -> anyhow::Result<()> {
    let editor_auto_fmt = cx.ui_tree.config().auto_format;
    let jobs = &mut cx.jobs;
    let buffer_mirror = current_mut!(cx.ui_tree);
    let view = buffer_view_mut!(cx.ui_tree);
    let path = path.map(AsRef::as_ref);

    let fmt = if editor_auto_fmt {
        buffer_mirror.auto_format().map(|fmt| {
            let callback = make_format_callback(
                buffer_mirror.id(),
                buffer_mirror.version(),
                view.id,
                fmt,
                Some((path.map(Into::into), force)),
            );

            jobs.add(Job::with_callback(callback).wait_before_exiting());
        })
    } else {
        None
    };

    if fmt.is_none() {
        let id = buffer_mirror.id();
        cx.ui_tree.save(id, path, force)?;
    }

    Ok(())
}

fn write(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    write_impl(cx, args.first(), false)
}

fn force_write(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    write_impl(cx, args.first(), true)
}

fn new_file(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    cx.ui_tree.new_file(Action::Replace);

    Ok(())
}

fn format(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let buffer_mirror = current_mut!(cx.ui_tree);
    let view = buffer_view_mut!(cx.ui_tree);
    if let Some(format) = buffer_mirror.format() {
        let callback = make_format_callback(buffer_mirror.id(), buffer_mirror.version(), view.id, format, None);
        cx.jobs.callback(callback);
    }

    Ok(())
}
fn set_indent_style(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    use IndentStyle::*;

    // If no argument, report current indent style.
    if args.is_empty() {
        let style = current!(cx.ui_tree).indent_style;
        cx.ui_tree.set_status(match style {
            Tabs => "tabs".to_owned(),
            Spaces(1) => "1 space".to_owned(),
            Spaces(n) if (2..=8).contains(&n) => format!("{} spaces", n),
            _ => unreachable!(), // Shouldn't happen.
        });
        return Ok(());
    }

    // Attempt to parse argument as an indent style.
    let style = match args.get(0) {
        Some(arg) if "tabs".starts_with(&arg.to_lowercase()) => Some(Tabs),
        Some(Cow::Borrowed("0")) => Some(Tabs),
        Some(arg) => arg
            .parse::<u8>()
            .ok()
            .filter(|n| (1..=8).contains(n))
            .map(Spaces),
        _ => None,
    };

    let style = style.context("invalid indent style")?;
    let buffer_mirror = current_mut!(cx.ui_tree);
    buffer_mirror.indent_style = style;

    Ok(())
}

/// Sets or reports the current document's line ending setting.
fn set_line_ending(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    use LineEnding::*;

    // If no argument, report current line ending setting.
    if args.is_empty() {
        let line_ending = current!(cx.ui_tree).line_ending;
        cx.ui_tree.set_status(match line_ending {
            Crlf => "crlf",
            LF => "line feed",
            #[cfg(feature = "unicode-lines")]
            FF => "form feed",
            #[cfg(feature = "unicode-lines")]
            CR => "carriage return",
            #[cfg(feature = "unicode-lines")]
            Nel => "next line",

            // These should never be a document's default line ending.
            #[cfg(feature = "unicode-lines")]
            VT | LS | PS => "error",
        });

        return Ok(());
    }

    let arg = args
        .get(0)
        .context("argument missing")?
        .to_ascii_lowercase();

    // Attempt to parse argument as a line ending.
    let line_ending = match arg {
        arg if arg.starts_with("crlf") => Crlf,
        arg if arg.starts_with("lf") => LF,
        #[cfg(feature = "unicode-lines")]
        arg if arg.starts_with("cr") => CR,
        #[cfg(feature = "unicode-lines")]
        arg if arg.starts_with("ff") => FF,
        #[cfg(feature = "unicode-lines")]
        arg if arg.starts_with("nel") => Nel,
        _ => bail!("invalid line ending"),
    };
    let buffer_mirror = current_mut!(cx.ui_tree);
    buffer_mirror.line_ending = line_ending;

    let mut pos = 0;
    let transaction = Transaction::change(
        buffer_mirror.text(),
        buffer_mirror.text().lines().filter_map(|line| {
            pos += line.len_chars();
            match helix_core::line_ending::get_line_ending(&line) {
                Some(ending) if ending != line_ending => {
                    let start = pos - ending.len_chars();
                    let end = pos;
                    Some((start, end, Some(line_ending.as_str().into())))
                }
                _ => None,
            }
        }),
    );
    let view = buffer_view_mut!(cx.ui_tree);
    apply_transaction(&transaction, buffer_mirror);
    buffer_mirror.append_changes_to_history(view);

    Ok(())
}

fn earlier(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let uk = args.join(" ").parse::<UndoKind>().map_err(|s| anyhow!(s))?;

    let buffer_mirror = current_mut!(cx.ui_tree);
    let view = buffer_view_mut!(cx.ui_tree);
    let success = buffer_mirror.earlier(view, uk);
    if !success {
        cx.ui_tree.set_status("Already at oldest change");
    }

    Ok(())
}

fn later(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let uk = args.join(" ").parse::<UndoKind>().map_err(|s| anyhow!(s))?;
    let buffer_mirror = current_mut!(cx.ui_tree);
    let view = buffer_view_mut!(cx.ui_tree);
    let success = buffer_view.later(view, uk);
    if !success {
        cx.ui_tree.set_status("Already at newest change");
    }

    Ok(())
}

fn write_quit(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    write_impl(cx, args.first(), false)?;
    cx.block_try_flush_writes()?;
    quit(cx, &[], event)
}

fn force_write_quit(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    write_impl(cx, args.first(), true)?;
    cx.block_try_flush_writes()?;
    force_quit(cx, &[], event)
}

/// Results in an error if there are modified buffers remaining and sets editor
/// error, otherwise returns `Ok(())`. If the current document is unmodified,
/// and there are modified documents, switches focus to one of them.
pub(super) fn buffers_remaining_impl(editor: &mut UITree) -> anyhow::Result<()> {
    let (modified_ids, modified_names): (Vec<_>, Vec<_>) = editor
        .documents()
        .filter(|doc| doc.is_modified())
        .map(|doc| (doc.buffer_id(), doc.display_name()))
        .unzip();
    if let Some(first) = modified_ids.first() {
        let current = current!(editor);
        // If the current document is unmodified, and there are modified
        // documents, switch focus to the first modified doc.
        if !modified_ids.contains(&current.id()) {
            editor.switch(*first, Action::Replace);
        }
        bail!(
            "{} unsaved buffer(s) remaining: {:?}",
            modified_names.len(),
            modified_names
        );
    }
    Ok(())
}

pub fn write_all_impl(
    cx: &mut compositor::Context,
    force: bool,
    write_scratch: bool,
) -> anyhow::Result<()> {
    let mut errors: Vec<&'static str> = Vec::new();
    let auto_format = cx.ui_tree.config().auto_format;
    let jobs = &mut cx.jobs;
    let current_view = buffer_view!(cx.ui_tree);

    // save all documents
    let saves: Vec<_> = cx
        .ui_tree
        .documents
        .values_mut()
        .filter_map(|doc| {
            if !doc.is_modified() {
                return None;
            }
            if doc.path().is_none() {
                if write_scratch {
                    errors.push("cannot write a buffer without a filename\n");
                }
                return None;
            }

            // TODO: write all done server side
            todo!()
            // Look for a view to apply the formatting change to. If the document
            // is in the current view, just use that. Otherwise, since we don't
            // have any other metric available for better selection, just pick
            // the first view arbitrarily so that we still commit the document
            // state for undos. If somehow we have a document that has not been
            // initialized with any view, initialize it with the current view.
            let target_view = if doc.selections().contains_key(&current_view.id) {
                current_view.id
            } else if let Some(view) = doc.selections().keys().next() {
                *view
            } else {
                current_view.id
            };

            let fmt = if auto_format {
                doc.auto_format().map(|fmt| {
                    let callback = make_format_callback(
                        doc.id(),
                        doc.version(),
                        target_view,
                        fmt,
                        Some((None, force)),
                    );
                    jobs.add(Job::with_callback(callback).wait_before_exiting());
                })
            } else {
                None
            };

            if fmt.is_none() {
                return Some(doc.id());
            }

            None
        })
        .collect();

    // manually call save for the rest of docs that don't have a formatter
    for id in saves {
        cx.ui_tree.save::<PathBuf>(id, None, force)?;
    }

    if !errors.is_empty() && !force {
        bail!("{:?}", errors);
    }

    Ok(())
}

fn write_all(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    write_all_impl(cx, false, true)
}

fn write_all_quit(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }
    write_all_impl(cx, false, true)?;
    quit_all_impl(cx, false)
}

fn force_write_all_quit(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }
    let _ = write_all_impl(cx, true, true);
    quit_all_impl(cx, true)
}

fn quit_all_impl(cx: &mut compositor::Context, force: bool) -> anyhow::Result<()> {
    cx.block_try_flush_writes()?;
    if !force {
        buffers_remaining_impl(cx.ui_tree)?;
    }

    // close all views
    let views: Vec<_> = cx.ui_tree.tree.views().map(|(view, _)| view.id).collect();
    for view_id in views {
        cx.ui_tree.close(view_id);
    }

    Ok(())
}

fn quit_all(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    quit_all_impl(cx, false)
}

fn force_quit_all(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    quit_all_impl(cx, true)
}

fn cquit(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let exit_code = args
        .first()
        .and_then(|code| code.parse::<i32>().ok())
        .unwrap_or(1);

    cx.ui_tree.exit_code = exit_code;
    quit_all_impl(cx, false)
}

fn force_cquit(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let exit_code = args
        .first()
        .and_then(|code| code.parse::<i32>().ok())
        .unwrap_or(1);
    cx.ui_tree.exit_code = exit_code;

    quit_all_impl(cx, true)
}

fn theme(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    let true_color = cx.ui_tree.config.load().true_color || crate::true_color();
    match event {
        PromptEvent::Abort => {
            cx.ui_tree.unset_theme_preview();
        }
        PromptEvent::Update => {
            if args.is_empty() {
                // Ensures that a preview theme gets cleaned up if the user backspaces until the prompt is empty.
                cx.ui_tree.unset_theme_preview();
            } else if let Some(theme_name) = args.first() {
                if let Ok(theme) = cx.ui_tree.theme_loader.load(theme_name) {
                    if !(true_color || theme.is_16_color()) {
                        bail!("Unsupported theme: theme requires true color support");
                    }
                    cx.ui_tree.set_theme_preview(theme);
                };
            };
        }
        PromptEvent::Validate => {
            if let Some(theme_name) = args.first() {
                let theme = cx
                    .ui_tree
                    .theme_loader
                    .load(theme_name)
                    .map_err(|err| anyhow::anyhow!("Could not load theme: {}", err))?;
                if !(true_color || theme.is_16_color()) {
                    bail!("Unsupported theme: theme requires true color support");
                }
                cx.ui_tree.set_theme(theme);
            } else {
                let name = cx.ui_tree.theme.name().to_string();

                cx.ui_tree.set_status(name);
            }
        }
    };

    Ok(())
}

fn yank_main_selection_to_clipboard(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    yank_main_selection_to_clipboard_impl(cx.ui_tree, ClipboardType::Clipboard)
}

fn yank_joined_to_clipboard(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let doc = current!(cx.ui_tree);
    let default_sep = Cow::Borrowed(doc.line_ending.as_str());
    let separator = args.first().unwrap_or(&default_sep);
    yank_joined_to_clipboard_impl(cx.ui_tree, separator, ClipboardType::Clipboard)
}

fn yank_main_selection_to_primary_clipboard(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    yank_main_selection_to_clipboard_impl(cx.ui_tree, ClipboardType::Selection)
}

fn yank_joined_to_primary_clipboard(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let doc = current!(cx.ui_tree);
    let default_sep = Cow::Borrowed(doc.line_ending.as_str());
    let separator = args.first().unwrap_or(&default_sep);
    yank_joined_to_clipboard_impl(cx.ui_tree, separator, ClipboardType::Selection)
}

fn paste_clipboard_after(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    paste_clipboard_impl(cx.ui_tree, Paste::After, ClipboardType::Clipboard, 1)
}

fn paste_clipboard_before(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    paste_clipboard_impl(cx.ui_tree, Paste::Before, ClipboardType::Clipboard, 1)
}

fn paste_primary_clipboard_after(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    paste_clipboard_impl(cx.ui_tree, Paste::After, ClipboardType::Selection, 1)
}

fn paste_primary_clipboard_before(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    paste_clipboard_impl(cx.ui_tree, Paste::Before, ClipboardType::Selection, 1)
}

fn replace_selections_with_clipboard_impl(
    cx: &mut compositor::Context,
    clipboard_type: ClipboardType,
) -> anyhow::Result<()> {
    let buffer_mirror = current_mut!(cx.ui_tree);
    let view = buffer_view_mut!(cx.ui_tree);
    match cx.ui_tree.clipboard_provider.get_contents(clipboard_type) {
        Ok(contents) => {
            let selection = buffer_mirror.selection(view.id);
            let transaction = Transaction::change_by_selection(buffer_mirror.text(), selection, |range| {
                (range.from(), range.to(), Some(contents.as_str().into()))
            });

            apply_transaction(&transaction, buffer_mirror);
            buffer_mirror.append_changes_to_history(view);
            Ok(())
        }
        Err(e) => Err(e.context("Couldn't get system clipboard contents")),
    }
}

fn replace_selections_with_clipboard(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    replace_selections_with_clipboard_impl(cx, ClipboardType::Clipboard)
}

fn replace_selections_with_primary_clipboard(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    replace_selections_with_clipboard_impl(cx, ClipboardType::Selection)
}

fn show_clipboard_provider(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    cx.ui_tree
        .set_status(cx.ui_tree.clipboard_provider.name().to_string());
    Ok(())
}

fn change_current_directory(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let dir = helix_core::path::expand_tilde(
        args.first()
            .context("target directory not provided")?
            .as_ref()
            .as_ref(),
    );

    if let Err(e) = std::env::set_current_dir(dir) {
        bail!("Couldn't change the current working directory: {}", e);
    }

    let cwd = std::env::current_dir().context("Couldn't get the new working directory")?;
    cx.ui_tree.set_status(format!(
        "Current working directory is now {}",
        cwd.display()
    ));
    Ok(())
}

fn show_current_directory(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let cwd = std::env::current_dir().context("Couldn't get the new working directory")?;
    cx.ui_tree
        .set_status(format!("Current working directory is {}", cwd.display()));
    Ok(())
}

/// Sets the [`Document`]'s encoding..
fn set_encoding(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let buffer_mirror = current_mut!(cx.ui_tree);
    if let Some(label) = args.first() {
        buffer_mirror.set_encoding(label)
    } else {
        let encoding = buffer_mirror.encoding().name().to_owned();
        cx.ui_tree.set_status(encoding);
        Ok(())
    }
}

/// Reload the [`Document`] from its source file.
fn reload(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let scrolloff = cx.ui_tree.config().scrolloff;
    let redraw_handle = cx.ui_tree.redraw_handle.clone();
    let buffer_mirror = current_mut!(cx.ui_tree);
    let view = buffer_view_mut!(cx.ui_tree);
    buffer_mirror.reload(view, &cx.ui_tree.diff_providers, redraw_handle)
        .map(|_| {
            view.ensure_cursor_in_view(buffer_mirror, scrolloff);
        })
}

fn reload_all(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let scrolloff = cx.ui_tree.config().scrolloff;
    let view_id = buffer_view!(cx.ui_tree).id;

    // TODO: ui_tree will all the view_ids.
    let docs_view_ids: Vec<(BufferID, Vec<BufferViewID>)> = cx
        .ui_tree
        .documents_mut()
        .map(|doc| {
            let mut view_ids: Vec<_> = doc.selections().keys().cloned().collect();

            if view_ids.is_empty() {
                doc.ensure_view_init(view_id);
                view_ids.push(view_id);
            };

            (doc.id(), view_ids)
        })
        .collect();

    for (doc_id, view_ids) in docs_view_ids {
        let buffer_mirror = current_mut!(cx.ui_tree, &doc_id);
        // Every doc is guaranteed to have at least 1 view at this point.
        let view = buffer_view_mut!(cx.ui_tree, view_ids[0]);
        // Ensure that the view is synced with the document's history.
        view.sync_changes(buffer_mirror);

        let redraw_handle = cx.ui_tree.redraw_handle.clone();
        buffer_mirror.reload(view, &cx.ui_tree.diff_providers, redraw_handle)?;

        for view_id in view_ids {
            let view = buffer_view_mut!(cx.ui_tree, view_id);
            if view.doc.eq(&doc_id) {
                view.ensure_cursor_in_view(buffer_mirror, scrolloff);
            }
        }
    }

    Ok(())
}

/// Update the [`Document`] if it has been modified.
fn update(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let buffer_mirror = current_mut!(cx.ui_tree);
    if buffer_mirror.is_modified() {
        write(cx, args, event)
    } else {
        Ok(())
    }
}

fn lsp_workspace_command(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let buffer_mirror = current_mut!(cx.ui_tree);

    let language_server = match buffer_mirror.language_server() {
        Some(language_server) => language_server,
        None => {
            cx.ui_tree
                .set_status("Language server not active for current buffer");
            return Ok(());
        }
    };

    let options = match &language_server.capabilities().execute_command_provider {
        Some(options) => options,
        None => {
            cx.ui_tree
                .set_status("Workspace commands are not supported for this language server");
            return Ok(());
        }
    };
    if args.is_empty() {
        let commands = options
            .commands
            .iter()
            .map(|command| helix_lsp::lsp::Command {
                title: command.clone(),
                command: command.clone(),
                arguments: None,
            })
            .collect::<Vec<_>>();
        let callback = async move {
            let call: job::Callback = Callback::EditorCompositor(Box::new(
                move |_editor: &mut UITree, compositor: &mut Compositor| {
                    let picker = ui::Picker::new(commands, (), |cx, command, _action| {
                        execute_lsp_command(cx.ui_tree, command.clone());
                    });
                    compositor.push(Box::new(overlayed(picker)))
                },
            ));
            Ok(call)
        };
        cx.jobs.callback(callback);
    } else {
        let command = args.join(" ");
        if options.commands.iter().any(|c| c == &command) {
            execute_lsp_command(
                cx.ui_tree,
                helix_lsp::lsp::Command {
                    title: command.clone(),
                    arguments: None,
                    command,
                },
            );
        } else {
            cx.ui_tree.set_status(format!(
                "`{command}` is not supported for this language server"
            ));
            return Ok(());
        }
    }
    Ok(())
}

fn lsp_restart(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let buffer_mirror = current_mut!(cx.ui_tree);
    let config = buffer_mirror
        .language_config()
        .context("LSP not defined for the current document")?;

    let scope = config.scope.clone();
    cx.ui_tree.language_servers.restart(config, buffer_mirror.path())?;

    // This collect is needed because refresh_language_server would need to re-borrow editor.
    let document_ids_to_refresh: Vec<BufferID> = cx
        .ui_tree
        .documents()
        .filter_map(|doc| match doc.language_config() {
            Some(config) if config.scope.eq(&scope) => Some(doc.id()),
            _ => None,
        })
        .collect();

    for document_id in document_ids_to_refresh {
        cx.ui_tree.refresh_language_server(document_id);
    }

    Ok(())
}

fn tree_sitter_scopes(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let buffer_mirror = current_mut!(cx.ui_tree);
    let text = buffer_mirror.text().slice(..);
    let pos = buffer_mirror.selection().primary().cursor(text);
    let scopes = indent::get_scopes(buffer_mirror.syntax(), text, pos);

    let contents = format!("```json\n{:?}\n````", scopes);

    let callback = async move {
        let call: job::Callback = Callback::EditorCompositor(Box::new(
            move |editor: &mut UITree, compositor: &mut Compositor| {
                let contents = ui::Markdown::new(contents, editor.syn_loader.clone());
                let popup = Popup::new("hover", contents).auto_close(true);
                compositor.replace_or_push("hover", popup);
            },
        ));
        Ok(call)
    };

    cx.jobs.callback(callback);

    Ok(())
}

fn vsplit(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let id = buffer_view!(cx.ui_tree).doc;

    if args.is_empty() {
        cx.ui_tree.switch(id, Action::VerticalSplit);
    } else {
        for arg in args {
            cx.ui_tree
                .open(&PathBuf::from(arg.as_ref()), Action::VerticalSplit)?;
        }
    }

    Ok(())
}

fn hsplit(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let id = buffer_view!(cx.ui_tree).doc;

    if args.is_empty() {
        cx.ui_tree.switch(id, Action::HorizontalSplit);
    } else {
        for arg in args {
            cx.ui_tree
                .open(&PathBuf::from(arg.as_ref()), Action::HorizontalSplit)?;
        }
    }

    Ok(())
}

fn vsplit_new(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    cx.ui_tree.new_file(Action::VerticalSplit);

    Ok(())
}

fn hsplit_new(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    cx.ui_tree.new_file(Action::HorizontalSplit);

    Ok(())
}

fn debug_eval(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    if let Some(debugger) = cx.ui_tree.debugger.as_mut() {
        let (frame, thread_id) = match (debugger.active_frame, debugger.thread_id) {
            (Some(frame), Some(thread_id)) => (frame, thread_id),
            _ => {
                bail!("Cannot find current stack frame to access variables")
            }
        };

        // TODO: support no frame_id

        let frame_id = debugger.stack_frames[&thread_id][frame].id;
        let response = helix_lsp::block_on(debugger.eval(args.join(" "), Some(frame_id)))?;
        cx.ui_tree.set_status(response.result);
    }
    Ok(())
}

fn debug_start(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let mut args = args.to_owned();
    let name = match args.len() {
        0 => None,
        _ => Some(args.remove(0)),
    };
    dap_start_impl(cx, name.as_deref(), None, Some(args))
}

fn debug_remote(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let mut args = args.to_owned();
    let address = match args.len() {
        0 => None,
        _ => Some(args.remove(0).parse()?),
    };
    let name = match args.len() {
        0 => None,
        _ => Some(args.remove(0)),
    };
    dap_start_impl(cx, name.as_deref(), address, Some(args))
}

fn tutor(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let path = helix_loader::runtime_dir().join("tutor");
    cx.ui_tree.open(&path, Action::Replace)?;
    // Unset path to prevent accidentally saving to the original tutor file.
    current_mut!(cx.ui_tree).set_path(None)?;
    Ok(())
}

pub(super) fn goto_line_number(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    match event {
        PromptEvent::Abort => {
            if let Some(line_number) = cx.ui_tree.last_line_number {
                goto_line_impl(cx.ui_tree, NonZeroUsize::new(line_number));
                let buffer_mirror = current_mut!(cx.ui_tree);
                let view = buffer_view_mut!(cx.ui_tree);
                view.ensure_cursor_in_view(buffer_mirror, line_number);
                cx.ui_tree.last_line_number = None;
            }
            return Ok(());
        }
        PromptEvent::Validate => {
            ensure!(!args.is_empty(), "Line number required");
            cx.ui_tree.last_line_number = None;
        }
        PromptEvent::Update => {
            if args.is_empty() {
                if let Some(line_number) = cx.ui_tree.last_line_number {
                    // When a user hits backspace and there are no numbers left,
                    // we can bring them back to their original line
                    goto_line_impl(cx.ui_tree, NonZeroUsize::new(line_number));
                    let buffer_mirror = current_mut!(cx.ui_tree);
                    let view = buffer_view_mut!(cx.ui_tree);
                    view.ensure_cursor_in_view(buffer_mirror, line_number);
                    cx.ui_tree.last_line_number = None;
                }
                return Ok(());
            }
            let buffer_mirror = current_mut!(cx.ui_tree);
            let text = buffer_mirror.text().slice(..);
            let line = buffer_mirror.selection().primary().cursor_line(text);
            cx.ui_tree.last_line_number.get_or_insert(line + 1);
        }
    }
    let line = args[0].parse::<usize>()?;
    goto_line_impl(cx.ui_tree, NonZeroUsize::new(line));
    let buffer_mirror = current_mut!(cx.ui_tree);
    let view = buffer_view_mut!(cx.ui_tree);
    view.ensure_cursor_in_view(buffer_mirror, line);
    Ok(())
}

// Fetch the current value of a config option and output as status.
fn get_option(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    if args.len() != 1 {
        anyhow::bail!("Bad arguments. Usage: `:get key`");
    }

    let key = &args[0].to_lowercase();
    let key_error = || anyhow::anyhow!("Unknown key `{}`", key);

    let config = serde_json::json!(cx.ui_tree.config().deref());
    let pointer = format!("/{}", key.replace('.', "/"));
    let value = config.pointer(&pointer).ok_or_else(key_error)?;

    cx.ui_tree.set_status(value.to_string());
    Ok(())
}

/// Change config at runtime. Access nested values by dot syntax, for
/// example to disable smart case search, use `:set search.smart-case false`.
fn set_option(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    if args.len() != 2 {
        anyhow::bail!("Bad arguments. Usage: `:set key field`");
    }
    let (key, arg) = (&args[0].to_lowercase(), &args[1]);

    let key_error = || anyhow::anyhow!("Unknown key `{}`", key);
    let field_error = |_| anyhow::anyhow!("Could not parse field `{}`", arg);

    let mut config = serde_json::json!(&cx.ui_tree.config().deref());
    let pointer = format!("/{}", key.replace('.', "/"));
    let value = config.pointer_mut(&pointer).ok_or_else(key_error)?;

    *value = if value.is_string() {
        // JSON strings require quotes, so we can't .parse() directly
        serde_json::Value::String(arg.to_string())
    } else {
        arg.parse().map_err(field_error)?
    };
    let config = serde_json::from_value(config).map_err(field_error)?;

    cx.ui_tree
        .config_events
        .0
        .send(ConfigEvent::Update(config))?;
    Ok(())
}

/// Change the language of the current buffer at runtime.
fn language(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    if args.len() != 1 {
        anyhow::bail!("Bad arguments. Usage: `:set-language language`");
    }

    let buffer_mirror = current_mut!(cx.ui_tree);

    if args[0] == "text" {
        buffer_mirror.set_language(None, None)
    } else {
        buffer_mirror.set_language_by_language_id(&args[0], cx.ui_tree.syn_loader.clone())?;
    }
    buffer_mirror.detect_indent_and_line_ending();

    let id = buffer_mirror.id();
    cx.ui_tree.refresh_language_server(id);
    Ok(())
}

fn sort(cx: &mut compositor::Context, args: &[Cow<str>], event: PromptEvent) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    sort_impl(cx, args, false)
}

fn sort_reverse(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    sort_impl(cx, args, true)
}

fn sort_impl(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    reverse: bool,
) -> anyhow::Result<()> {
    let buffer_mirror = current_mut!(cx.ui_tree);
    let text = buffer_mirror.text().slice(..);

    let selection = buffer_mirror.selection();

    let mut fragments: Vec<_> = selection
        .slices(text)
        .map(|fragment| fragment.chunks().collect())
        .collect();

    fragments.sort_by(match reverse {
        true => |a: &Tendril, b: &Tendril| b.cmp(a),
        false => |a: &Tendril, b: &Tendril| a.cmp(b),
    });

    let transaction = Transaction::change(
        buffer_mirror.text(),
        selection
            .into_iter()
            .zip(fragments)
            .map(|(s, fragment)| (s.from(), s.to(), Some(fragment))),
    );

    let view = buffer_view_mut!(cx.ui_tree);
    apply_transaction(&transaction, buffer_mirror);
    buffer_mirror.append_changes_to_history(view);

    Ok(())
}

fn reflow(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let scrolloff = cx.ui_tree.config().scrolloff;
    let buffer_mirror = current_mut!(cx.ui_tree);

    const DEFAULT_MAX_LEN: usize = 79;

    // Find the max line length by checking the following sources in order:
    //   - The passed argument in `args`
    //   - The configured max_line_len for this language in languages.toml
    //   - The const default we set above
    let max_line_len: usize = args
        .get(0)
        .map(|num| num.parse::<usize>())
        .transpose()?
        .or_else(|| {
            buffer_mirror.language_config()
                .and_then(|config| config.max_line_length)
        })
        .unwrap_or(DEFAULT_MAX_LEN);

    let rope = buffer_mirror.text();

    let selection = buffer_mirror.selection();
    let transaction = Transaction::change_by_selection(rope, selection, |range| {
        let fragment = range.fragment(rope.slice(..));
        let reflowed_text = helix_core::wrap::reflow_hard_wrap(&fragment, max_line_len);

        (range.from(), range.to(), Some(reflowed_text))
    });

    let view = buffer_view_mut!(cx.ui_tree);
    apply_transaction(&transaction, buffer_mirror);
    buffer_mirror.append_changes_to_history(view);
    view.ensure_cursor_in_view(buffer_mirror, scrolloff);

    Ok(())
}

fn tree_sitter_subtree(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let buffer_mirror = current_mut!(cx.ui_tree);

    if let Some(syntax) = buffer_mirror.syntax() {
        let primary_selection = buffer_mirror.selection().primary();
        let text = buffer_mirror.text();
        let from = text.char_to_byte(primary_selection.from());
        let to = text.char_to_byte(primary_selection.to());
        if let Some(selected_node) = syntax
            .tree()
            .root_node()
            .descendant_for_byte_range(from, to)
        {
            let mut contents = String::from("```tsq\n");
            helix_core::syntax::pretty_print_tree(&mut contents, selected_node)?;
            contents.push_str("\n```");

            let callback = async move {
                let call: job::Callback = Callback::EditorCompositor(Box::new(
                    move |editor: &mut UITree, compositor: &mut Compositor| {
                        let contents = ui::Markdown::new(contents, editor.syn_loader.clone());
                        let popup = Popup::new("hover", contents).auto_close(true);
                        compositor.replace_or_push("hover", popup);
                    },
                ));
                Ok(call)
            };

            cx.jobs.callback(callback);
        }
    }

    Ok(())
}

fn open_config(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    cx.ui_tree
        .open(&helix_loader::config_file(), Action::Replace)?;
    Ok(())
}

fn open_log(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    cx.ui_tree.open(&helix_loader::log_file(), Action::Replace)?;
    Ok(())
}

fn refresh_config(
    cx: &mut compositor::Context,
    _args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    cx.ui_tree.config_events.0.send(ConfigEvent::Refresh)?;
    Ok(())
}

fn append_output(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    ensure!(!args.is_empty(), "Shell command required");
    shell(cx, &args.join(" "), &ShellBehavior::Append);
    Ok(())
}

fn insert_output(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    ensure!(!args.is_empty(), "Shell command required");
    shell(cx, &args.join(" "), &ShellBehavior::Insert);
    Ok(())
}

fn pipe_to(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    pipe_impl(cx, args, event, &ShellBehavior::Ignore)
}

fn pipe(cx: &mut compositor::Context, args: &[Cow<str>], event: PromptEvent) -> anyhow::Result<()> {
    pipe_impl(cx, args, event, &ShellBehavior::Replace)
}

fn pipe_impl(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
    behavior: &ShellBehavior,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    ensure!(!args.is_empty(), "Shell command required");
    shell(cx, &args.join(" "), behavior);
    Ok(())
}

fn run_shell_command(
    cx: &mut compositor::Context,
    args: &[Cow<str>],
    event: PromptEvent,
) -> anyhow::Result<()> {
    if event != PromptEvent::Validate {
        return Ok(());
    }

    let shell = &cx.ui_tree.config().shell;
    let (output, success) = shell_impl(shell, &args.join(" "), None)?;
    if success {
        cx.ui_tree.set_status("Command succeed");
    } else {
        cx.ui_tree.set_error("Command failed");
    }

    if !output.is_empty() {
        let callback = async move {
            let call: job::Callback = Callback::EditorCompositor(Box::new(
                move |editor: &mut UITree, compositor: &mut Compositor| {
                    let contents = ui::Markdown::new(
                        format!("```sh\n{}\n```", output),
                        editor.syn_loader.clone(),
                    );
                    let popup = Popup::new("shell", contents).position(Some(
                        helix_core::Position::new(editor.cursor().0.unwrap_or_default().row, 2),
                    ));
                    compositor.replace_or_push("shell", popup);
                },
            ));
            Ok(call)
        };

        cx.jobs.callback(callback);
    }

    Ok(())
}

#[allow(clippy::unnecessary_unwrap)]
pub(super) fn command_mode(cx: &mut Context) {
    use shellwords::Shellwords;
    use helix_view::command::{Command, COMMAND_MAP, CommandArguments};

    let mut prompt = Prompt::new(":".into(), Some(':'), |editor: &UITree, input: &str| {
            static FUZZY_MATCHER: Lazy<fuzzy_matcher::skim::SkimMatcherV2> =
                Lazy::new(fuzzy_matcher::skim::SkimMatcherV2::default);

            let shellwords = Shellwords::from(input);
            let words = shellwords.words();

            if words.is_empty() || (words.len() == 1 && !shellwords.ends_with_whitespace()) {
                // If the command has not been finished yet, complete commands.
                let mut matches: Vec<_> = COMMAND_MAP
                    .iter()
                    .filter_map(|command| {
                        FUZZY_MATCHER
                            .fuzzy_match(command.name, input)
                            .map(|score| (command.name, score))
                    })
                    .collect();

                matches.sort_unstable_by_key(|(_file, score)| std::cmp::Reverse(*score));
                matches
                    .into_iter()
                    .map(|(name, _)| (0.., name.into()))
                    .collect()
            } else {
                // Otherwise, use the command's completer and the last shellword
                // as completion input.
                let (part, part_len) = if words.len() == 1 || shellwords.ends_with_whitespace() {
                    (&Cow::Borrowed(""), 0)
                } else {
                    (
                        words.last().unwrap(),
                        shellwords.parts().last().unwrap().len(),
                    )
                };

                // TODO: show the completer for the respective argument
                todo!();
                if let Some( Command {
                    args: Some(CommandArgument),
                    ..
                }) = .get(&words[0] as &str)
                {
                    completer(editor, part)
                        .into_iter()
                        .map(|(range, file)| {
                            let file = shellwords::escape(file);

                            // offset ranges to input
                            let offset = input.len() - part_len;
                            let range = (range.start + offset)..;
                            (range, file)
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }, // completion
        move |cx: &mut compositor::Context, input: &str, event: PromptEvent| {
            let parts = input.split_whitespace().collect::<Vec<&str>>();
            if parts.is_empty() {
                return;
            }

            // If command is numeric, interpret as line number and go there.
            if parts.len() == 1 && parts[0].parse::<usize>().ok().is_some() {
                if let Err(e) = typed::goto_line_number(cx, &[Cow::from(parts[0])], event) {
                    cx.ui_tree.set_error(format!("{}", e));
                }
                return;
            }

            // Handle typable commands
            if let Some(cmd) = COMMAND_MAP.get(parts[0]) {
                let shellwords = Shellwords::from(input);
                let args = shellwords.words();

                if let Err(e) = (cmd.fun)(cx, &args[1..], event) {
                    cx.ui_tree.set_error(format!("{}", e));
                }
            } else if event == PromptEvent::Validate {
                cx.ui_tree
                    .set_error(format!("no such command: '{}'", parts[0]));
            }
        },
    );
    prompt.doc_fn = Box::new(|input: &str| {
        let part = input.split(' ').next().unwrap_or_default();
        if let Some(Command {description, aliases, ..}) = COMMAND_MAP.get(part) {
            if aliases.is_empty() {
                return Some((*description).into());
            }
            return Some(format!("{descrpition}\nAliases: {}", command.aliases.join(", ")).into());
        }
        None
    });

    // Calculate initial completion
    prompt.recalculate_completion(cx.ui_tree);
    cx.push_layer(Box::new(prompt));
}
