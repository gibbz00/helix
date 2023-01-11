pub(crate) mod dap;
pub(crate) mod lsp;
pub(crate) mod typed;

pub use dap::*;
pub use lsp::*;
pub use typed::*;

use crate::{
    commands::insert::*,
    args,
    keymap::CommandList,
    compositor::{self, Component, Compositor},
    job::{Callback, self, Jobs},
    ui::{self, overlay::overlayed, FilePicker, Picker, Popup, Prompt, PromptEvent, menu::{Cell, Row}},
};

use helix_vcs::Hunk;
use helix_core::{
    comment, coords_at_pos, encoding, find_first_non_whitespace_char, find_root, graphemes,
    history::UndoKind,
    increment::date_time::DateTimeIncrementor,
    increment::{number::NumberIncrementor, Increment},
    indent,
    indent::IndentStyle,
    line_ending::{get_line_ending_of_str, line_end_char_index, str_is_line_ending},
    match_brackets,
    movement::{self, Direction, Movement},
    object, pos_at_coords, pos_at_visual_coords,
    regex::{self, Regex, RegexBuilder},
    search::{self, CharMatcher},
    selection, shellwords, surround, textobject,
    tree_sitter::Node,
    unicode::width::UnicodeWidthChar,
    visual_coords_at_pos, LineEnding, Position, Range, Rope, RopeGraphemes, RopeSlice, Selection,
    SmallVec, Tendril, Transaction,
};
use helix_view::{
    align_view, Align,
    apply_transaction,
    clipboard::ClipboardType,
    buffer::{FormatterError, Mode, SCRATCH_BUFFER_NAME},
    ui_tree::{Action, Motion},
    info::Info,
    input::KeyEvent,
    keyboard::KeyCode,
    tree,
    buffer_view::BufferView,
    Buffer, BufferID, ui_tree, BufferViewID,
};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    future::Future,
    borrow::Cow,
    path::{Path, PathBuf},
    fmt,
};
use anyhow::{anyhow, bail, ensure, Context as _};
use fuzzy_matcher::FuzzyMatcher;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use serde::de::{self, Deserialize, Deserializer};
use grep_regex::RegexMatcherBuilder;
use grep_searcher::{sinks, BinaryDetection, SearcherBuilder};
use ignore::{DirEntry, WalkBuilder, WalkState};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct Context<'a> {
    pub ui_tree: &ui_tree,
    pub callback: Option<crate::compositor::Callback>,
    pub on_next_key_callback: Option<Box<dyn FnOnce(&mut Context, KeyEvent)>>,
    pub jobs: &'a mut Jobs,
}

impl<'a> Context<'a> {
    /// Push a new component onto the compositor.
    pub fn push_layer(&mut self, component: Box<dyn Component>) {
        self.callback = Some(Box::new(|compositor: &mut Compositor, _| {
            compositor.push(component)
        }));
    }

    #[inline]
    pub fn on_next_key(
        &mut self,
        on_next_key_callback: impl FnOnce(&mut Context, KeyEvent) + 'static,
    ) {
        self.on_next_key_callback = Some(Box::new(on_next_key_callback));
    }

    #[inline]
    pub fn callback<T, F>(
        &mut self,
        call: impl Future<Output = helix_lsp::Result<serde_json::Value>> + 'static + Send,
        callback: F,
    ) where
        T: for<'de> serde::Deserialize<'de> + Send + 'static,
        F: FnOnce(&mut UITree, &mut Compositor, T) + Send + 'static,
    {
        let callback = Box::pin(async move {
            let json = call.await?;
            let response = serde_json::from_value(json)?;
            let call: job::Callback = Callback::ui_treeCompositor(Box::new(
                move |ui_tree: &mut UITree, compositor: &mut Compositor| {
                    callback(ui_tree, compositor, response)
                },
            ));
            Ok(call)
        });
        self.jobs.callback(callback);
    }
}


fn move_impl<F>(cx: &mut Context, move_fn: F, dir: Direction, behaviour: Movement)
where
    F: Fn(RopeSlice, Range, Direction, usize, Movement, usize) -> Range,
{
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer
        .selection(buffer_view.view_id)
        .clone()
        .transform(|range| move_fn(text, range, dir, command_multiplier, behaviour, buffer.tab_width()));
    buffer.set_selection(buffer_view.view_id, selection);
}

use helix_core::movement::{move_horizontally, move_vertically};

fn move_char_left(cx: &mut Context) {
    move_impl(cx, move_horizontally, Direction::Backward, Movement::Move)
}

fn move_char_right(cx: &mut Context) {
    move_impl(cx, move_horizontally, Direction::Forward, Movement::Move)
}

fn move_line_up(cx: &mut Context) {
    move_impl(cx, move_vertically, Direction::Backward, Movement::Move)
}

fn move_line_down(cx: &mut Context) {
    move_impl(cx, move_vertically, Direction::Forward, Movement::Move)
}

fn extend_char_left(cx: &mut Context) {
    move_impl(cx, move_horizontally, Direction::Backward, Movement::Extend)
}

fn extend_char_right(cx: &mut Context) {
    move_impl(cx, move_horizontally, Direction::Forward, Movement::Extend)
}

fn extend_line_up(cx: &mut Context) {
    move_impl(cx, move_vertically, Direction::Backward, Movement::Extend)
}

fn extend_line_down(cx: &mut Context) {
    move_impl(cx, move_vertically, Direction::Forward, Movement::Extend)
}

fn goto_line_end_impl(buffer_view: &mut BufferView, buffer: &mut Buffer, movement: Movement) {
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let line = range.cursor_line(text);
        let line_start = text.line_to_char(line);

        let pos = graphemes::prev_grapheme_boundary(text, line_end_char_index(&text, line))
            .max(line_start);

        range.put_cursor(text, pos, movement == Movement::Extend)
    });
    buffer.set_selection(buffer_view.view_id, selection);
}

fn goto_line_end(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    goto_line_end_impl(
        buffer_view,
        buffer,
        if cx.ui_tree.mode == Mode::Select {
            Movement::Extend
        } else {
            Movement::Move
        },
    )
}

fn extend_to_line_end(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    goto_line_end_impl(buffer_view, buffer, Movement::Extend)
}

fn goto_line_end_newline_impl(buffer_view: &mut BufferView, buffer: &mut Buffer, movement: Movement) {
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let line = range.cursor_line(text);
        let pos = line_end_char_index(&text, line);

        range.put_cursor(text, pos, movement == Movement::Extend)
    });
    buffer.set_selection(buffer_view.view_id, selection);
}

fn goto_line_end_newline(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    goto_line_end_newline_impl(
        buffer_view,
        buffer,
        if cx.ui_tree.mode == Mode::Select {
            Movement::Extend
        } else {
            Movement::Move
        },
    )
}

fn extend_to_line_end_newline(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    goto_line_end_newline_impl(buffer_view, buffer, Movement::Extend)
}

fn goto_next_buffer(cx: &mut Context) {
    goto_buffer(cx.ui_tree, Direction::Forward);
}

fn goto_previous_buffer(cx: &mut Context) {
    goto_buffer(cx.ui_tree, Direction::Backward);
}

fn goto_buffer(ui_tree: &mut UITree, direction: Direction) {
    let current = buffer_view!(ui_tree).buffer;

    let id = match direction {
        Direction::Forward => {
            let iter = ui_tree.buffers.keys();
            let mut iter = iter.skip_while(|id| *id != &current);
            iter.next(); // skip current item
            iter.next().or_else(|| ui_tree.buffers.keys().next())
        }
        Direction::Backward => {
            let iter = ui_tree.buffers.keys();
            let mut iter = iter.rev().skip_while(|id| *id != &current);
            iter.next(); // skip current item
            iter.next().or_else(|| ui_tree.buffers.keys().rev().next())
        }
    }
    .unwrap();

    let id = *id;

    ui_tree.switch(id, Action::Replace);
}


fn kill_to_line_start(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let line = range.cursor_line(text);
        let first_char = text.line_to_char(line);
        let anchor = range.cursor(text);
        let head = if anchor == first_char && line != 0 {
            // select until previous line
            line_end_char_index(&text, line - 1)
        } else if let Some(pos) = find_first_non_whitespace_char(text.line(line)) {
            if first_char + pos < anchor {
                // select until first non-blank in line if cursor is after it
                first_char + pos
            } else {
                // select until start of line
                first_char
            }
        } else {
            // select until start of line
            first_char
        };
        Range::new(head, anchor)
    });
    delete_selection_insert_mode(buffer, buffer_view, &selection);

    lsp::signature_help_impl(cx, SignatureHelpInvoked::Automatic);
}

fn kill_to_line_end(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let line = range.cursor_line(text);
        let line_end_pos = line_end_char_index(&text, line);
        let pos = range.cursor(text);

        let mut new_range = range.put_cursor(text, line_end_pos, true);
        // don't want to remove the line separator itself if the cursor doesn't reach the end of line.
        if pos != line_end_pos {
            new_range.head = line_end_pos;
        }
        new_range
    });
    delete_selection_insert_mode(buffer, buffer_view, &selection);

    lsp::signature_help_impl(cx, SignatureHelpInvoked::Automatic);
}

fn goto_first_nonwhitespace(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let line = range.cursor_line(text);

        if let Some(pos) = find_first_non_whitespace_char(text.line(line)) {
            let pos = pos + text.line_to_char(line);
            range.put_cursor(text, pos, cx.ui_tree.mode == Mode::Select)
        } else {
            range
        }
    });
    buffer.set_selection(buffer_view.view_id, selection);
}

fn trim_selections(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let ranges: SmallVec<[Range; 1]> = buffer
        .selection(buffer_view.view_id)
        .iter()
        .filter_map(|range| {
            if range.is_empty() || range.slice(text).chars().all(|ch| ch.is_whitespace()) {
                return None;
            }
            let mut start = range.from();
            let mut end = range.to();
            start = movement::skip_while(text, start, |x| x.is_whitespace()).unwrap_or(start);
            end = movement::backwards_skip_while(text, end, |x| x.is_whitespace()).unwrap_or(end);
            Some(Range::new(start, end).with_direction(range.direction()))
        })
        .collect();

    if !ranges.is_empty() {
        let primary = buffer.selection(buffer_view.view_id).primary();
        let idx = ranges
            .iter()
            .position(|range| range.overlaps(&primary))
            .unwrap_or(ranges.len() - 1);
        buffer.set_selection(buffer_view.view_id, Selection::new(ranges, idx));
    } else {
        collapse_selection(cx);
        keep_primary_selection(cx);
    };
}

// align text in selection
fn align_selections(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);
    let selection = buffer.selection(buffer_view.view_id);

    let tab_width = buffer.tab_width();
    let mut column_widths: Vec<Vec<_>> = Vec::new();
    let mut last_line = text.len_lines() + 1;
    let mut col = 0;

    for range in selection {
        let coords = visual_coords_at_pos(text, range.head, tab_width);
        let anchor_coords = visual_coords_at_pos(text, range.anchor, tab_width);

        if coords.row != anchor_coords.row {
            cx.ui_tree
                .set_error("align cannot work with multi line selections");
            return;
        }

        col = if coords.row == last_line { col + 1 } else { 0 };

        if col >= column_widths.len() {
            column_widths.push(Vec::new());
        }
        column_widths[col].push((range.from(), coords.col));

        last_line = coords.row;
    }

    let mut changes = Vec::with_capacity(selection.len());

    // Account for changes on each row
    let len = column_widths.first().map(|cols| cols.len()).unwrap_or(0);
    let mut offs = vec![0; len];

    for col in column_widths {
        let max_col = col
            .iter()
            .enumerate()
            .map(|(row, (_, cursor))| *cursor + offs[row])
            .max()
            .unwrap_or(0);

        for (row, (insert_pos, last_col)) in col.into_iter().enumerate() {
            let ins_count = max_col - (last_col + offs[row]);

            if ins_count == 0 {
                continue;
            }

            offs[row] += ins_count;

            changes.push((insert_pos, insert_pos, Some(" ".repeat(ins_count).into())));
        }
    }

    // The changeset has to be sorted
    changes.sort_unstable_by_key(|(from, _, _)| *from);

    let transaction = Transaction::change(buffer.text(), changes.into_iter());
    apply_transaction(&transaction, buffer, buffer_view);
}

fn goto_window(cx: &mut Context, align: Align) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get() - 1;
    let config = cx.ui_tree.config();
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let height = buffer_view.inner_height();

    // respect user given count if any
    // - 1 so we have at least one gap in the middle.
    // a height of 6 with padding of 3 on each side will keep shifting the buffer_view back and forth
    // as we type
    let scrolloff = config.scrolloff.min(height.saturating_sub(1) / 2);

    let last_line = buffer_view.last_line(buffer);

    let line = match align {
        Align::Top => buffer_view.offset.row + scrolloff + count,
        Align::Center => buffer_view.offset.row + ((last_line - buffer_view.offset.row) / 2),
        Align::Bottom => last_line.saturating_sub(scrolloff + count),
    }
    .max(buffer_view.offset.row + scrolloff)
    .min(last_line.saturating_sub(scrolloff));

    let pos = buffer.text().line_to_char(line);
    let text = buffer.text().slice(..);
    let selection = buffer
        .selection(buffer_view.view_id)
        .clone()
        .transform(|range| range.put_cursor(text, pos, cx.ui_tree.mode == Mode::Select));
    buffer.set_selection(buffer_view.view_id, selection);
}

fn goto_window_top(cx: &mut Context) {
    goto_window(cx, Align::Top)
}

fn goto_window_center(cx: &mut Context) {
    goto_window(cx, Align::Center)
}

fn goto_window_bottom(cx: &mut Context) {
    goto_window(cx, Align::Bottom)
}

fn move_word_impl<F>(cx: &mut Context, move_fn: F)
where
    F: Fn(RopeSlice, Range, usize) -> Range,
{
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer
        .selection(buffer_view.view_id)
        .clone()
        .transform(|range| move_fn(text, range, count));
    buffer.set_selection(buffer_view.view_id, selection);
}

fn move_next_word_start(cx: &mut Context) {
    move_word_impl(cx, movement::move_next_word_start)
}

fn move_prev_word_start(cx: &mut Context) {
    move_word_impl(cx, movement::move_prev_word_start)
}

fn move_prev_word_end(cx: &mut Context) {
    move_word_impl(cx, movement::move_prev_word_end)
}

fn move_next_word_end(cx: &mut Context) {
    move_word_impl(cx, movement::move_next_word_end)
}

fn move_next_long_word_start(cx: &mut Context) {
    move_word_impl(cx, movement::move_next_long_word_start)
}

fn move_prev_long_word_start(cx: &mut Context) {
    move_word_impl(cx, movement::move_prev_long_word_start)
}

fn move_next_long_word_end(cx: &mut Context) {
    move_word_impl(cx, movement::move_next_long_word_end)
}

fn goto_para_impl<F>(cx: &mut Context, move_fn: F)
where
    F: Fn(RopeSlice, Range, usize, Movement) -> Range + 'static,
{
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let motion = move |ui_tree: &mut UITree| {
        let (buffer_view, buffer) = current!(ui_tree);
        let text = buffer.text().slice(..);
        let behavior = if ui_tree.mode == Mode::Select {
            Movement::Extend
        } else {
            Movement::Move
        };

        let selection = buffer
            .selection(buffer_view.view_id)
            .clone()
            .transform(|range| move_fn(text, range, count, behavior));
        buffer.set_selection(buffer_view.view_id, selection);
    };
    motion(cx.ui_tree);
    cx.ui_tree.last_motion = Some(Motion(Box::new(motion)));
}

fn goto_prev_paragraph(cx: &mut Context) {
    goto_para_impl(cx, movement::move_prev_paragraph)
}

fn goto_next_paragraph(cx: &mut Context) {
    goto_para_impl(cx, movement::move_next_paragraph)
}

fn goto_file_start(cx: &mut Context) {
    if ui_tree.command_multiplier.get().is_some() {
        goto_line(cx);
    } else {
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let text = buffer.text().slice(..);
        let selection = buffer
            .selection(buffer_view.view_id)
            .clone()
            .transform(|range| range.put_cursor(text, 0, cx.ui_tree.mode == Mode::Select));
        push_jump(buffer_view, buffer);
        buffer.set_selection(buffer_view.view_id, selection);
    }
}

fn goto_file_end(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);
    let pos = buffer.text().len_chars();
    let selection = buffer
        .selection(buffer_view.view_id)
        .clone()
        .transform(|range| range.put_cursor(text, pos, cx.ui_tree.mode == Mode::Select));
    push_jump(buffer_view, buffer);
    buffer.set_selection(buffer_view.view_id, selection);
}

fn goto_file(cx: &mut Context) {
    goto_file_impl(cx, Action::Replace);
}

fn goto_file_hsplit(cx: &mut Context) {
    goto_file_impl(cx, Action::HorizontalSplit);
}

fn goto_file_vsplit(cx: &mut Context) {
    goto_file_impl(cx, Action::VerticalSplit);
}

/// Goto files in selection.
fn goto_file_impl(cx: &mut Context, action: Action) {
    let (buffer_view, buffer) = current_ref!(cx.ui_tree);
    let text = buffer.text();
    let selections = buffer.selection(buffer_view.view_id);
    let mut paths: Vec<_> = selections
        .iter()
        .map(|r| text.slice(r.from()..r.to()).to_string())
        .collect();
    let primary = selections.primary();
    // Checks whether there is only one selection with a width of 1
    if selections.len() == 1 && primary.len() == 1 {
        let count = ui_tree.command_multiplier.unwrap_or_one().get();
        let text_slice = text.slice(..);
        // In this case it selects the WORD under the cursor
        let current_word = textobject::textobject_word(
            text_slice,
            primary,
            textobject::TextObject::Inside,
            count,
            true,
        );
        // Trims some surrounding chars so that the actual file is opened.
        let surrounding_chars: &[_] = &['\'', '"', '(', ')'];
        paths.clear();
        paths.push(
            current_word
                .fragment(text_slice)
                .trim_matches(surrounding_chars)
                .to_string(),
        );
    }
    for sel in paths {
        let p = sel.trim();
        if !p.is_empty() {
            if let Err(e) = cx.ui_tree.open(&PathBuf::from(p), action) {
                cx.ui_tree.set_error(format!("Open file failed: {:?}", e));
            }
        }
    }
}

fn extend_word_impl<F>(cx: &mut Context, extend_fn: F)
where
    F: Fn(RopeSlice, Range, usize) -> Range,
{
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let word = extend_fn(text, range, count);
        let pos = word.cursor(text);
        range.put_cursor(text, pos, true)
    });
    buffer.set_selection(buffer_view.view_id, selection);
}

fn extend_next_word_start(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_word_start)
}

fn extend_prev_word_start(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_word_start)
}

fn extend_next_word_end(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_word_end)
}

fn extend_prev_word_end(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_word_end)
}

fn extend_next_long_word_start(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_long_word_start)
}

fn extend_prev_long_word_start(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_long_word_start)
}

fn extend_next_long_word_end(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_long_word_end)
}

fn will_find_char<F>(cx: &mut Context, search_fn: F, inclusive: bool, extend: bool)
where
    F: Fn(RopeSlice, char, usize, usize, bool) -> Option<usize> + 'static,
{
    // TODO: count is reset when a mappable command is found, so we move it into the closure here.
    // Would be nice to carry over.
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();

    // need to wait for next key
    // TODO: should this be done by grapheme rather than char?  For example,
    // we can't properly handle the line-ending CRLF case here in terms of char.
    cx.on_next_key(move |cx, event| {
        let ch = match event {
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } =>
            // TODO: this isn't quite correct when CRLF is involved.
            // This hack will work in most cases, since buffers don't
            // usually mix line endings.  But we should fix it eventually
            // anyway.
            {
                buffer!(cx.ui_tree).line_ending.as_str().chars().next().unwrap()
            }

            KeyEvent {
                code: KeyCode::Tab, ..
            } => '\t',

            KeyEvent {
                code: KeyCode::Char(ch),
                ..
            } => ch,
            _ => return,
        };

        find_char_impl(cx.ui_tree, &search_fn, inclusive, extend, ch, count);
        cx.ui_tree.last_motion = Some(Motion(Box::new(move |ui_tree: &mut UITree| {
            find_char_impl(ui_tree, &search_fn, inclusive, true, ch, 1);
        })));
    })
}

//

#[inline]
fn find_char_impl<F, M: CharMatcher + Clone + Copy>(
    ui_tree: &mut UITree,
    search_fn: &F,
    inclusive: bool,
    extend: bool,
    char_matcher: M,
    count: usize,
) where
    F: Fn(RopeSlice, M, usize, usize, bool) -> Option<usize> + 'static,
{
    let (buffer_view, buffer) = current!(ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        // TODO: use `Range::cursor()` here instead.  However, that works in terms of
        // graphemes, whereas this function doesn't yet.  So we're doing the same logic
        // here, but just in terms of chars instead.
        let search_start_pos = if range.anchor < range.head {
            range.head - 1
        } else {
            range.head
        };

        search_fn(text, char_matcher, search_start_pos, count, inclusive).map_or(range, |pos| {
            if extend {
                range.put_cursor(text, pos, true)
            } else {
                Range::point(range.cursor(text)).put_cursor(text, pos, true)
            }
        })
    });
    buffer.set_selection(buffer_view.view_id, selection);
}

fn find_next_char_impl(
    text: RopeSlice,
    ch: char,
    pos: usize,
    n: usize,
    inclusive: bool,
) -> Option<usize> {
    let pos = (pos + 1).min(text.len_chars());
    if inclusive {
        search::find_nth_next(text, ch, pos, n)
    } else {
        let n = match text.get_char(pos) {
            Some(next_ch) if next_ch == ch => n + 1,
            _ => n,
        };
        search::find_nth_next(text, ch, pos, n).map(|n| n.saturating_sub(1))
    }
}

fn find_prev_char_impl(
    text: RopeSlice,
    ch: char,
    pos: usize,
    n: usize,
    inclusive: bool,
) -> Option<usize> {
    if inclusive {
        search::find_nth_prev(text, ch, pos, n)
    } else {
        let n = match text.get_char(pos.saturating_sub(1)) {
            Some(next_ch) if next_ch == ch => n + 1,
            _ => n,
        };
        search::find_nth_prev(text, ch, pos, n).map(|n| (n + 1).min(text.len_chars()))
    }
}

fn find_till_char(cx: &mut Context) {
    will_find_char(cx, find_next_char_impl, false, false)
}

fn find_next_char(cx: &mut Context) {
    will_find_char(cx, find_next_char_impl, true, false)
}

fn extend_till_char(cx: &mut Context) {
    will_find_char(cx, find_next_char_impl, false, true)
}

fn extend_next_char(cx: &mut Context) {
    will_find_char(cx, find_next_char_impl, true, true)
}

fn till_prev_char(cx: &mut Context) {
    will_find_char(cx, find_prev_char_impl, false, false)
}

fn find_prev_char(cx: &mut Context) {
    will_find_char(cx, find_prev_char_impl, true, false)
}

fn extend_till_prev_char(cx: &mut Context) {
    will_find_char(cx, find_prev_char_impl, false, true)
}

fn extend_prev_char(cx: &mut Context) {
    will_find_char(cx, find_prev_char_impl, true, true)
}

fn repeat_last_motion(cx: &mut Context) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let last_motion = cx.ui_tree.last_motion.take();
    if let Some(m) = &last_motion {
        for _ in 0..count {
            m.run(cx.ui_tree);
        }
        cx.ui_tree.last_motion = last_motion;
    }
}

fn replace(cx: &mut Context) {
    let mut buf = [0u8; 4]; // To hold utf8 encoded char.

    // need to wait for next key
    cx.on_next_key(move |cx, event| {
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let ch: Option<&str> = match event {
            KeyEvent {
                code: KeyCode::Char(ch),
                ..
            } => Some(ch.encode_utf8(&mut buf[..])),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Some(buffer.line_ending.as_str()),
            KeyEvent {
                code: KeyCode::Tab, ..
            } => Some("\t"),
            _ => None,
        };

        let selection = buffer.selection(buffer_view.view_id);

        if let Some(ch) = ch {
            let transaction = Transaction::change_by_selection(buffer.text(), selection, |range| {
                if !range.is_empty() {
                    let text: String =
                        RopeGraphemes::new(buffer.text().slice(range.from()..range.to()))
                            .map(|g| {
                                let cow: Cow<str> = g.into();
                                if str_is_line_ending(&cow) {
                                    cow
                                } else {
                                    ch.into()
                                }
                            })
                            .collect();

                    (range.from(), range.to(), Some(text.into()))
                } else {
                    // No change.
                    (range.from(), range.to(), None)
                }
            });

            apply_transaction(&transaction, buffer, buffer_view);
            exit_select_mode(cx);
        }
    })
}

fn switch_case_impl<F>(cx: &mut Context, change_fn: F)
where
    F: Fn(RopeSlice) -> Tendril,
{
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let selection = buffer.selection(buffer_view.view_id);
    let transaction = Transaction::change_by_selection(buffer.text(), selection, |range| {
        let text: Tendril = change_fn(range.slice(buffer.text().slice(..)));

        (range.from(), range.to(), Some(text))
    });

    apply_transaction(&transaction, buffer, buffer_view);
}

fn switch_case(cx: &mut Context) {
    switch_case_impl(cx, |string| {
        string
            .chars()
            .flat_map(|ch| {
                if ch.is_lowercase() {
                    ch.to_uppercase().collect()
                } else if ch.is_uppercase() {
                    ch.to_lowercase().collect()
                } else {
                    vec![ch]
                }
            })
            .collect()
    });
}

fn switch_to_uppercase(cx: &mut Context) {
    switch_case_impl(cx, |string| {
        string.chunks().map(|chunk| chunk.to_uppercase()).collect()
    });
}

fn switch_to_lowercase(cx: &mut Context) {
    switch_case_impl(cx, |string| {
        string.chunks().map(|chunk| chunk.to_lowercase()).collect()
    });
}

pub fn scroll(cx: &mut Context, offset: usize, direction: Direction) {
    use Direction::*;
    let config = cx.ui_tree.config();
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let range = buffer.selection(buffer_view.view_id).primary();
    let text = buffer.text().slice(..);

    let cursor = visual_coords_at_pos(text, range.cursor(text), buffer.tab_width());
    let buffer_last_line = buffer.text().len_lines().saturating_sub(1);

    let last_line = buffer_view.last_line(buffer);

    if direction == Backward && buffer_view.offset.row == 0
        || direction == Forward && last_line == buffer_last_line
    {
        return;
    }

    let height = buffer_view.inner_height();

    let scrolloff = config.scrolloff.min(height / 2);

    buffer_view.offset.row = match direction {
        Forward => buffer_view.offset.row + offset,
        Backward => buffer_view.offset.row.saturating_sub(offset),
    }
    .min(buffer_last_line);

    // recalculate last line
    let last_line = buffer_view.last_line(buffer);

    // clamp into buffer_viewport
    let line = cursor
        .row
        .max(buffer_view.offset.row + scrolloff)
        .min(last_line.saturating_sub(scrolloff));

    // If cursor needs moving, replace primary selection
    if line != cursor.row {
        let head = pos_at_visual_coords(text, Position::new(line, cursor.col), buffer.tab_width()); // this func will properly truncate to line end

        let anchor = if cx.ui_tree.mode == Mode::Select {
            range.anchor
        } else {
            head
        };

        // replace primary selection with an empty selection at cursor pos
        let prim_sel = Range::new(anchor, head);
        let mut sel = buffer.selection(buffer_view.view_id).clone();
        let idx = sel.primary_index();
        sel = sel.replace(idx, prim_sel);
        buffer.set_selection(buffer_view.view_id, sel);
    }
}

fn page_up(cx: &mut Context) {
    let buffer_view = buffer_view!(cx.ui_tree);
    let offset = buffer_view.inner_height();
    scroll(cx, offset, Direction::Backward);
}

fn page_down(cx: &mut Context) {
    let buffer_view = buffer_view!(cx.ui_tree);
    let offset = buffer_view.inner_height();
    scroll(cx, offset, Direction::Forward);
}

fn half_page_up(cx: &mut Context) {
    let buffer_view = buffer_view!(cx.ui_tree);
    let offset = buffer_view.inner_height() / 2;
    scroll(cx, offset, Direction::Backward);
}

fn half_page_down(cx: &mut Context) {
    let buffer_view = buffer_view!(cx.ui_tree);
    let offset = buffer_view.inner_height() / 2;
    scroll(cx, offset, Direction::Forward);
}

fn copy_selection_on_line(cx: &mut Context, direction: Direction) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);
    let selection = buffer.selection(buffer_view.view_id);
    let mut ranges = SmallVec::with_capacity(selection.ranges().len() * (count + 1));
    ranges.extend_from_slice(selection.ranges());
    let mut primary_index = 0;
    for range in selection.iter() {
        let is_primary = *range == selection.primary();

        // The range is always head exclusive
        let (head, anchor) = if range.anchor < range.head {
            (range.head - 1, range.anchor)
        } else {
            (range.head, range.anchor.saturating_sub(1))
        };

        let tab_width = buffer.tab_width();

        let head_pos = visual_coords_at_pos(text, head, tab_width);
        let anchor_pos = visual_coords_at_pos(text, anchor, tab_width);

        let height = std::cmp::max(head_pos.row, anchor_pos.row)
            - std::cmp::min(head_pos.row, anchor_pos.row)
            + 1;

        if is_primary {
            primary_index = ranges.len();
        }
        ranges.push(*range);

        let mut sels = 0;
        let mut i = 0;
        while sels < count {
            let offset = (i + 1) * height;

            let anchor_row = match direction {
                Direction::Forward => anchor_pos.row + offset,
                Direction::Backward => anchor_pos.row.saturating_sub(offset),
            };

            let head_row = match direction {
                Direction::Forward => head_pos.row + offset,
                Direction::Backward => head_pos.row.saturating_sub(offset),
            };

            if anchor_row >= text.len_lines() || head_row >= text.len_lines() {
                break;
            }

            let anchor =
                pos_at_visual_coords(text, Position::new(anchor_row, anchor_pos.col), tab_width);
            let head = pos_at_visual_coords(text, Position::new(head_row, head_pos.col), tab_width);

            // skip lines that are too short
            if visual_coords_at_pos(text, anchor, tab_width).col == anchor_pos.col
                && visual_coords_at_pos(text, head, tab_width).col == head_pos.col
            {
                if is_primary {
                    primary_index = ranges.len();
                }
                // This is Range::new(anchor, head), but it will place the cursor on the correct column
                ranges.push(Range::point(anchor).put_cursor(text, head, true));
                sels += 1;
            }

            i += 1;
        }
    }

    let selection = Selection::new(ranges, primary_index);
    buffer.set_selection(buffer_view.view_id, selection);
}

fn copy_selection_on_prev_line(cx: &mut Context) {
    copy_selection_on_line(cx, Direction::Backward)
}

fn copy_selection_on_next_line(cx: &mut Context) {
    copy_selection_on_line(cx, Direction::Forward)
}

fn select_all(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let end = buffer.text().len_chars();
    buffer.set_selection(buffer_view.view_id, Selection::single(0, end))
}

fn select_regex(cx: &mut Context) {
    let reg = cx.ui_tree.selected_register.unwrap_or('/');
    ui::regex_prompt(
        cx,
        "select:".into(),
        Some(reg),
        ui::completers::none,
        move |ui_tree, regex, event| {
            let (buffer_view, buffer) = current!(ui_tree);
            if !matches!(event, PromptEvent::Update | PromptEvent::Validate) {
                return;
            }
            let text = buffer.text().slice(..);
            if let Some(selection) =
                selection::select_on_matches(text, buffer.selection(buffer_view.view_id), &regex)
            {
                buffer.set_selection(buffer_view.view_id, selection);
            }
        },
    );
}

fn split_selection(cx: &mut Context) {
    let reg = cx.ui_tree.selected_register.unwrap_or('/');
    ui::regex_prompt(
        cx,
        "split:".into(),
        Some(reg),
        ui::completers::none,
        move |ui_tree, regex, event| {
            let (buffer_view, buffer) = current!(ui_tree);
            if !matches!(event, PromptEvent::Update | PromptEvent::Validate) {
                return;
            }
            let text = buffer.text().slice(..);
            let selection = selection::split_on_matches(text, buffer.selection(buffer_view.view_id), &regex);
            buffer.set_selection(buffer_view.view_id, selection);
        },
    );
}

fn split_selection_on_newline(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);
    // only compile the regex once
    #[allow(clippy::trivial_regex)]
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\r\n|[\n\r\u{000B}\u{000C}\u{0085}\u{2028}\u{2029}]").unwrap());
    let selection = selection::split_on_matches(text, buffer.selection(buffer_view.view_id), &REGEX);
    buffer.set_selection(buffer_view.view_id, selection);
}

#[allow(clippy::too_many_arguments)]
fn search_impl(
    ui_tree: &mut UITree,
    contents: &str,
    regex: &Regex,
    movement: Movement,
    direction: Direction,
    scrolloff: usize,
    wrap_around: bool,
    show_warnings: bool,
) {
    let (buffer_view, buffer) = current!(ui_tree);
    let text = buffer.text().slice(..);
    let selection = buffer.selection(buffer_view.view_id);

    // Get the right side of the primary block cursor for forward search, or the
    // grapheme before the start of the selection for reverse search.
    let start = match direction {
        Direction::Forward => text.char_to_byte(graphemes::ensure_grapheme_boundary_next(
            text,
            selection.primary().to(),
        )),
        Direction::Backward => text.char_to_byte(graphemes::ensure_grapheme_boundary_prev(
            text,
            selection.primary().from(),
        )),
    };

    // A regex::Match returns byte-positions in the str. In the case where we
    // do a reverse search and wraparound to the end, we don't need to search
    // the text before the current cursor position for matches, but by slicing
    // it out, we need to add it back to the position of the selection.
    let mut offset = 0;

    // use find_at to find the next match after the cursor, loop around the end
    // Careful, `Regex` uses `bytes` as offsets, not character indices!
    let mut mat = match direction {
        Direction::Forward => regex.find_at(contents, start),
        Direction::Backward => regex.find_iter(&contents[..start]).last(),
    };

    if mat.is_none() {
        if wrap_around {
            mat = match direction {
                Direction::Forward => regex.find(contents),
                Direction::Backward => {
                    offset = start;
                    regex.find_iter(&contents[start..]).last()
                }
            };
        }
        if show_warnings {
            if wrap_around && mat.is_some() {
                ui_tree.set_status("Wrapped around buffer");
            } else {
                ui_tree.set_error("No more matches");
            }
        }
    }

    let (buffer_view, buffer) = current!(ui_tree);
    let text = buffer.text().slice(..);
    let selection = buffer.selection(buffer_view.view_id);

    if let Some(mat) = mat {
        let start = text.byte_to_char(mat.start() + offset);
        let end = text.byte_to_char(mat.end() + offset);

        if end == 0 {
            // skip empty matches that don't make sense
            return;
        }

        // Determine range direction based on the primary range
        let primary = selection.primary();
        let range = Range::new(start, end).with_direction(primary.direction());

        let selection = match movement {
            Movement::Extend => selection.clone().push(range),
            Movement::Move => selection.clone().replace(selection.primary_index(), range),
        };

        buffer.set_selection(buffer_view.view_id, selection);
        buffer_view.ensure_cursor_in_view_center(buffer, scrolloff);
    };
}

fn search_completions(cx: &mut Context, reg: Option<char>) -> Vec<String> {
    let mut items = reg
        .and_then(|reg| cx.ui_tree.registers.get(reg))
        .map_or(Vec::new(), |reg| reg.read().iter().take(200).collect());
    items.sort_unstable();
    items.dedup();
    items.into_iter().cloned().collect()
}

fn search(cx: &mut Context) {
    searcher(cx, Direction::Forward)
}

fn rsearch(cx: &mut Context) {
    searcher(cx, Direction::Backward)
}

fn searcher(cx: &mut Context, direction: Direction) {
    let reg = cx.ui_tree.selected_register.unwrap_or('/');
    let config = cx.ui_tree.config();
    let scrolloff = config.scrolloff;
    let wrap_around = config.search.wrap_around;

    let buffer = buffer!(cx.ui_tree);

    // TODO: could probably share with select_on_matches?

    // HAXX: sadly we can't avoid allocating a single string for the whole buffer since we can't
    // feed chunks into the regex yet
    let contents = buffer.text().slice(..).to_string();
    let completions = search_completions(cx, Some(reg));

    ui::regex_prompt(
        cx,
        "search:".into(),
        Some(reg),
        move |_ui_tree: &ui_tree, input: &str| {
            completions
                .iter()
                .filter(|comp| comp.starts_with(input))
                .map(|comp| (0.., std::borrow::Cow::Owned(comp.clone())))
                .collect()
        },
        move |ui_tree, regex, event| {
            if !matches!(event, PromptEvent::Update | PromptEvent::Validate) {
                return;
            }
            search_impl(
                ui_tree,
                &contents,
                &regex,
                Movement::Move,
                direction,
                scrolloff,
                wrap_around,
                false,
            );
        },
    );
}

fn search_next_or_prev_impl(cx: &mut Context, movement: Movement, direction: Direction) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let config = cx.ui_tree.config();
    let scrolloff = config.scrolloff;
    let (_, buffer) = current!(cx.ui_tree);
    let registers = &cx.ui_tree.registers;
    if let Some(query) = registers.read('/').and_then(|query| query.last()) {
        let contents = buffer.text().slice(..).to_string();
        let search_config = &config.search;
        let case_insensitive = if search_config.smart_case {
            !query.chars().any(char::is_uppercase)
        } else {
            false
        };
        let wrap_around = search_config.wrap_around;
        if let Ok(regex) = RegexBuilder::new(query)
            .case_insensitive(case_insensitive)
            .multi_line(true)
            .build()
        {
            for _ in 0..count {
                search_impl(
                    cx.ui_tree,
                    &contents,
                    &regex,
                    movement,
                    direction,
                    scrolloff,
                    wrap_around,
                    true,
                );
            }
        } else {
            let error = format!("Invalid regex: {}", query);
            cx.ui_tree.set_error(error);
        }
    }
}

fn search_next(cx: &mut Context) {
    search_next_or_prev_impl(cx, Movement::Move, Direction::Forward);
}

fn search_prev(cx: &mut Context) {
    search_next_or_prev_impl(cx, Movement::Move, Direction::Backward);
}
fn extend_search_next(cx: &mut Context) {
    search_next_or_prev_impl(cx, Movement::Extend, Direction::Forward);
}

fn extend_search_prev(cx: &mut Context) {
    search_next_or_prev_impl(cx, Movement::Extend, Direction::Backward);
}

fn search_selection(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let contents = buffer.text().slice(..);

    let regex = buffer
        .selection(buffer_view.view_id)
        .iter()
        .map(|selection| regex::escape(&selection.fragment(contents)))
        .collect::<HashSet<_>>() // Collect into hashset to deduplicate identical regexes
        .into_iter()
        .collect::<Vec<_>>()
        .join("|");

    let msg = format!("register '{}' set to '{}'", '/', &regex);
    cx.ui_tree.registers.push('/', regex);
    cx.ui_tree.set_status(msg);
}

fn make_search_word_bounded(cx: &mut Context) {
    let regex = match cx.ui_tree.registers.last('/') {
        Some(regex) => regex,
        None => return,
    };
    let start_anchored = regex.starts_with("\\b");
    let end_anchored = regex.ends_with("\\b");

    if start_anchored && end_anchored {
        return;
    }

    let mut new_regex = String::with_capacity(
        regex.len() + if start_anchored { 0 } else { 2 } + if end_anchored { 0 } else { 2 },
    );

    if !start_anchored {
        new_regex.push_str("\\b");
    }
    new_regex.push_str(regex);
    if !end_anchored {
        new_regex.push_str("\\b");
    }

    let msg = format!("register '{}' set to '{}'", '/', &new_regex);
    cx.ui_tree.registers.push('/', new_regex);
    cx.ui_tree.set_status(msg);
}

fn global_search(cx: &mut Context) {
    #[derive(Debug)]
    struct FileResult {
        path: PathBuf,
        /// 0 indexed lines
        line_num: usize,
    }

    impl FileResult {
        fn new(path: &Path, line_num: usize) -> Self {
            Self {
                path: path.to_path_buf(),
                line_num,
            }
        }
    }

    impl ui::menu::Item for FileResult {
        type Data = Option<PathBuf>;

        fn format(&self, current_path: &Self::Data) -> Row {
            let relative_path = helix_core::path::get_relative_path(&self.path)
                .to_string_lossy()
                .into_owned();
            if current_path
                .as_ref()
                .map(|p| p == &self.path)
                .unwrap_or(false)
            {
                format!("{} (*)", relative_path).into()
            } else {
                relative_path.into()
            }
        }
    }

    let (all_matches_sx, all_matches_rx) = tokio::sync::mpsc::unbounded_channel::<FileResult>();
    let config = cx.ui_tree.config();
    let smart_case = config.search.smart_case;
    let global_search_config = config.search.global.clone();

    let reg = cx.ui_tree.selected_register.unwrap_or('/');

    let completions = search_completions(cx, Some(reg));
    ui::regex_prompt(
        cx,
        "global-search:".into(),
        Some(reg),
        move |_ui_tree: &ui_tree, input: &str| {
            completions
                .iter()
                .filter(|comp| comp.starts_with(input))
                .map(|comp| (0.., std::borrow::Cow::Owned(comp.clone())))
                .collect()
        },
        move |_ui_tree, regex, event| {
            if event != PromptEvent::Validate {
                return;
            }

            if let Ok(matcher) = RegexMatcherBuilder::new()
                .case_smart(smart_case)
                .build(regex.as_str())
            {
                let searcher = SearcherBuilder::new()
                    .binary_detection(BinaryDetection::quit(b'\x00'))
                    .build();

                let search_root = std::env::current_dir()
                    .expect("Global search error: Failed to get current dir");
                WalkBuilder::new(search_root)
                    .hidden(global_search_config.hidden)
                    .parents(global_search_config.parents)
                    .ignore(global_search_config.ignore)
                    .follow_links(global_search_config.follow_symlinks)
                    .git_ignore(global_search_config.git_ignore)
                    .git_global(global_search_config.git_global)
                    .git_exclude(global_search_config.git_exclude)
                    .max_depth(global_search_config.max_depth)
                    // We always want to ignore the .git directory, otherwise if
                    // `ignore` is turned off above, we end up with a lot of noise
                    // in our picker.
                    .filter_entry(|entry| entry.file_name() != ".git")
                    .build_parallel()
                    .run(|| {
                        let mut searcher = searcher.clone();
                        let matcher = matcher.clone();
                        let all_matches_sx = all_matches_sx.clone();
                        Box::new(move |entry: Result<DirEntry, ignore::Error>| -> WalkState {
                            let entry = match entry {
                                Ok(entry) => entry,
                                Err(_) => return WalkState::Continue,
                            };

                            match entry.file_type() {
                                Some(entry) if entry.is_file() => {}
                                // skip everything else
                                _ => return WalkState::Continue,
                            };

                            let result = searcher.search_path(
                                &matcher,
                                entry.path(),
                                sinks::UTF8(|line_num, _| {
                                    all_matches_sx
                                        .send(FileResult::new(entry.path(), line_num as usize - 1))
                                        .unwrap();

                                    Ok(true)
                                }),
                            );

                            if let Err(err) = result {
                                log::error!(
                                    "Global search error: {}, {}",
                                    entry.path().display(),
                                    err
                                );
                            }
                            WalkState::Continue
                        })
                    });
            } else {
                // Otherwise do nothing
                // log::warn!("Global Search Invalid Pattern")
            }
        },
    );

    let current_path = buffer_mut!(cx.ui_tree).path().cloned();

    let show_picker = async move {
        let all_matches: Vec<FileResult> =
            UnboundedReceiverStream::new(all_matches_rx).collect().await;
        let call: job::Callback = Callback::ui_treeCompositor(Box::new(
            move |ui_tree: &mut UITree, compositor: &mut Compositor| {
                if all_matches.is_empty() {
                    ui_tree.set_status("No matches found");
                    return;
                }

                let picker = FilePicker::new(
                    all_matches,
                    current_path,
                    move |cx, FileResult { path, line_num }, action| {
                        match cx.ui_tree.open(path, action) {
                            Ok(_) => {}
                            Err(e) => {
                                cx.ui_tree.set_error(format!(
                                    "Failed to open file '{}': {}",
                                    path.display(),
                                    e
                                ));
                                return;
                            }
                        }

                        let line_num = *line_num;
                        let (buffer_view, buffer) = current!(cx.ui_tree);
                        let text = buffer.text();
                        let start = text.line_to_char(line_num);
                        let end = text.line_to_char((line_num + 1).min(text.len_lines()));

                        buffer.set_selection(buffer_view.view_id, Selection::single(start, end));
                        align_view(buffer, buffer_view, Align::Center);
                    },
                    |_ui_tree, FileResult { path, line_num }| {
                        Some((path.clone().into(), Some((*line_num, *line_num))))
                    },
                );
                compositor.push(Box::new(overlayed(picker)));
            },
        ));
        Ok(call)
    };
    cx.jobs.callback(show_picker);
}

enum Extend {
    Above,
    Below,
}

fn extend_line(cx: &mut Context) {
    let (buffer_view, buffer) = current_ref!(cx.ui_tree);
    let extend = match buffer.selection(buffer_view.view_id).primary().direction() {
        Direction::Forward => Extend::Below,
        Direction::Backward => Extend::Above,
    };
    extend_line_impl(cx, extend);
}

fn extend_line_below(cx: &mut Context) {
    extend_line_impl(cx, Extend::Below);
}

fn extend_line_above(cx: &mut Context) {
    extend_line_impl(cx, Extend::Above);
}

fn extend_line_impl(cx: &mut Context, extend: Extend) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let text = buffer.text();
    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let (start_line, end_line) = range.line_range(text.slice(..));

        let start = text.line_to_char(match extend {
            Extend::Above => start_line.saturating_sub(count - 1),
            Extend::Below => start_line,
        });
        let end = text.line_to_char(
            match extend {
                Extend::Above => end_line + 1, // the start of next line
                Extend::Below => end_line + count,
            }
            .min(text.len_lines()),
        );

        // extend to previous/next line if current line is selected
        let (anchor, head) = if range.from() == start && range.to() == end {
            match extend {
                Extend::Above => (end, text.line_to_char(start_line.saturating_sub(count))),
                Extend::Below => (
                    start,
                    text.line_to_char((end_line + count + 1).min(text.len_lines())),
                ),
            }
        } else {
            match extend {
                Extend::Above => (end, start),
                Extend::Below => (start, end),
            }
        };

        Range::new(anchor, head)
    });

    buffer.set_selection(buffer_view.view_id, selection);
}

fn extend_to_line_bounds(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    buffer.set_selection(
        buffer_view.view_id,
        buffer.selection(buffer_view.view_id).clone().transform(|range| {
            let text = buffer.text();

            let (start_line, end_line) = range.line_range(text.slice(..));
            let start = text.line_to_char(start_line);
            let end = text.line_to_char((end_line + 1).min(text.len_lines()));

            Range::new(start, end).with_direction(range.direction())
        }),
    );
}

fn shrink_to_line_bounds(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    buffer.set_selection(
        buffer_view.view_id,
        buffer.selection(buffer_view.view_id).clone().transform(|range| {
            let text = buffer.text();

            let (start_line, end_line) = range.line_range(text.slice(..));

            // Do nothing if the selection is within one line to prevent
            // conditional logic for the behavior of this command
            if start_line == end_line {
                return range;
            }

            let mut start = text.line_to_char(start_line);

            // line_to_char gives us the start position of the line, so
            // we need to get the start position of the next line. In
            // the ui_tree, this will correspond to the cursor being on
            // the EOL whitespace character, which is what we want.
            let mut end = text.line_to_char((end_line + 1).min(text.len_lines()));

            if start != range.from() {
                start = text.line_to_char((start_line + 1).min(text.len_lines()));
            }

            if end != range.to() {
                end = text.line_to_char(end_line);
            }

            Range::new(start, end).with_direction(range.direction())
        }),
    );
}

enum Operation {
    Delete,
    Change,
}

fn delete_selection_impl(cx: &mut Context, op: Operation) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let selection = buffer.selection(buffer_view.view_id);

    if cx.ui_tree.register != Some('_') {
        // first yank the selection
        let text = buffer.text().slice(..);
        let values: Vec<String> = selection.fragments(text).map(Cow::into_owned).collect();
        let reg_name = cx.ui_tree.selected_register.unwrap_or('"');
        cx.ui_tree.registers.write(reg_name, values);
    };

    // then delete
    let transaction = Transaction::change_by_selection(buffer.text(), selection, |range| {
        (range.from(), range.to(), None)
    });
    apply_transaction(&transaction, buffer, buffer_view);

    match op {
        Operation::Delete => {
            // exit select mode, if currently in select mode
            exit_select_mode(cx);
        }
        Operation::Change => {
            enter_insert_mode(cx);
        }
    }
}

#[inline]
fn delete_selection_insert_mode(buffer: &mut Buffer, buffer_view: &mut BufferView, selection: &Selection) {
    let transaction = Transaction::change_by_selection(buffer.text(), selection, |range| {
        (range.from(), range.to(), None)
    });
    apply_transaction(&transaction, buffer, buffer_view);
}

fn delete_selection(cx: &mut Context) {
    delete_selection_impl(cx, Operation::Delete);
}

fn delete_selection_noyank(cx: &mut Context) {
    cx.ui_tree.register = Some('_');
    delete_selection_impl(cx, Operation::Delete);
}

fn change_selection(cx: &mut Context) {
    delete_selection_impl(cx, Operation::Change);
}

fn change_selection_noyank(cx: &mut Context) {
    cx.ui_tree.register = Some('_');
    delete_selection_impl(cx, Operation::Change);
}

fn collapse_selection(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let pos = range.cursor(text);
        Range::new(pos, pos)
    });
    buffer.set_selection(buffer_view.view_id, selection);
}

fn flip_selections(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let selection = buffer
        .selection(buffer_view.view_id)
        .clone()
        .transform(|range| range.flip());
    buffer.set_selection(buffer_view.view_id, selection);
}

fn ensure_selections_forward(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let selection = buffer
        .selection(buffer_view.view_id)
        .clone()
        .transform(|r| r.with_direction(Direction::Forward));

    buffer.set_selection(buffer_view.view_id, selection);
}

fn enter_insert_mode(cx: &mut Context) {
    cx.ui_tree.mode = Mode::Insert;
}

// inserts at the start of each selection
fn insert_mode(cx: &mut Context) {
    enter_insert_mode(cx);
    let (buffer_view, buffer) = current!(cx.ui_tree);

    log::trace!(
        "entering insert mode with sel: {:?}, text: {:?}",
        buffer.selection(buffer_view.view_id),
        buffer.text().to_string()
    );

    let selection = buffer
        .selection(buffer_view.view_id)
        .clone()
        .transform(|range| Range::new(range.to(), range.from()));

    buffer.set_selection(buffer_view.view_id, selection);
}

// inserts at the end of each selection
fn append_mode(cx: &mut Context) {
    enter_insert_mode(cx);
    let (buffer_view, buffer) = current!(cx.ui_tree);
    buffer.restore_cursor = true;
    let text = buffer.text().slice(..);

    // Make sure there's room at the end of the buffer if the last
    // selection butts up against it.
    let end = text.len_chars();
    let last_range = buffer
        .selection(buffer_view.view_id)
        .iter()
        .last()
        .expect("selection should always have at least one range");
    if !last_range.is_empty() && last_range.to() == end {
        let transaction = Transaction::change(
            buffer.text(),
            [(end, end, Some(buffer.line_ending.as_str().into()))].into_iter(),
        );
        apply_transaction(&transaction, buffer, buffer_view);
    }

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        Range::new(
            range.from(),
            graphemes::next_grapheme_boundary(buffer.text().slice(..), range.to()),
        )
    });
    buffer.set_selection(buffer_view.view_id, selection);
}

fn file_picker(cx: &mut Context) {
    // We don't specify language markers, root will be the root of the current
    // git repo or the current dir if we're not in a repo
    let root = find_root(None, &[]);
    let picker = ui::file_picker(root, &cx.ui_tree.config());
    cx.push_layer(Box::new(overlayed(picker)));
}

fn file_picker_in_current_directory(cx: &mut Context) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("./"));
    let picker = ui::file_picker(cwd, &cx.ui_tree.config());
    cx.push_layer(Box::new(overlayed(picker)));
}

fn buffer_picker(cx: &mut Context) {
    let current = buffer_view!(cx.ui_tree).doc;

    struct BufferMeta {
        id: BufferID,
        path: Option<PathBuf>,
        is_modified: bool,
        is_current: bool,
    }

    impl ui::menu::Item for BufferMeta {
        type Data = ();

        fn format(&self, _data: &Self::Data) -> Row {
            let path = self
                .path
                .as_deref()
                .map(helix_core::path::get_relative_path);
            let path = match path.as_deref().and_then(Path::to_str) {
                Some(path) => path,
                None => SCRATCH_BUFFER_NAME,
            };

            let mut flags = Vec::new();
            if self.is_modified {
                flags.push("+");
            }
            if self.is_current {
                flags.push("*");
            }

            let flag = if flags.is_empty() {
                "".into()
            } else {
                format!(" ({})", flags.join(""))
            };
            format!("{} {}{}", self.id, path, flag).into()
        }
    }

    let new_meta = |doc: &Buffer| BufferMeta {
        id: buffer.id(),
        path: buffer.path().cloned(),
        is_modified: buffer.is_modified(),
        is_current: buffer.id() == current,
    };

    let picker = FilePicker::new(
        cx.ui_tree
            .documents
            .values()
            .map(|doc| new_meta(buffer))
            .collect(),
        (),
        |cx, meta, action| {
            cx.ui_tree.switch(meta.id, action);
        },
        |ui_tree, meta| {
            let doc = &ui_tree.buffers.get(&meta.id)?;
            let &view_id = buffer.selections().keys().next()?;
            let line = buffer
                .selection(buffer_view_id)
                .primary()
                .cursor_line(buffer.text().slice(..));
            Some((meta.id.into(), Some((line, line))))
        },
    );
    cx.push_layer(Box::new(overlayed(picker)));
}

fn jumplist_picker(cx: &mut Context) {
    struct JumpMeta {
        id: BufferID,
        path: Option<PathBuf>,
        selection: Selection,
        text: String,
        is_current: bool,
    }

    impl ui::menu::Item for JumpMeta {
        type Data = ();

        fn format(&self, _data: &Self::Data) -> Row {
            let path = self
                .path
                .as_deref()
                .map(helix_core::path::get_relative_path);
            let path = match path.as_deref().and_then(Path::to_str) {
                Some(path) => path,
                None => SCRATCH_BUFFER_NAME,
            };

            let mut flags = Vec::new();
            if self.is_current {
                flags.push("*");
            }

            let flag = if flags.is_empty() {
                "".into()
            } else {
                format!(" ({})", flags.join(""))
            };
            format!("{} {}{} {}", self.id, path, flag, self.text).into()
        }
    }

    let new_meta = |buffer_view: &BufferView, buffer: BufferID, selection: Selection| {
        let buffer = &cx.ui_tree.documents.get(&buffer);
        let text = buffer.map_or("".into(), |d| {
            selection
                .fragments(d.text().slice(..))
                .map(Cow::into_owned)
                .collect::<Vec<_>>()
                .join(" ")
        });

        JumpMeta {
            id: buffer_id,
            path: buffer.and_then(|buffer| buffer.path().cloned()),
            selection,
            text,
            is_current: buffer_view.buffer_id == buffer_id,
        }
    };

    let picker = FilePicker::new(
        cx.ui_tree
            .tree
            .views()
            .flat_map(|(buffer_view, _)| {
                buffer_view.jumps
                    .iter()
                    .map(|(buffer_id, selection)| new_meta(buffer_view, *buffer_id, selection.clone()))
            })
            .collect(),
        (),
        |cx, meta, action| {
            cx.ui_tree.switch(meta.id, action);
            let config = cx.ui_tree.config();
            let (buffer_view, buffer) = current!(cx.ui_tree);
            buffer.set_selection(buffer_view.view_id, meta.selection.clone());
            buffer_view.ensure_cursor_in_view_center(buffer), config.scrolloff);
        },
        |ui_tree, meta| {
            let doc = &ui_tree.buffers.get(&meta.id)?;
            let line = meta.selection.primary().cursor_line(buffer.text().slice(..));
            Some((meta.path.clone()?.into(), Some((line, line))))
        },
    );
    cx.push_layer(Box::new(overlayed(picker)));
}



// NOTE: does not present aliases
impl ui::menu::Item for Command {
    type Data = CommandList;

    fn format(&self, key_events_in_command_list: &Self::Data) -> Row {
        let mut row: Vec<Cell> = vec![Cell::from(self.name), Cell::from(""), Cell::from(self.description)];
        if Some(key_events) = key_events_in_command_list.get(self.name as &String) {
            row[1] = Cell::from(format_key_events(key_events));
        }
        return Row::new(row);

        // TODO: can probably be done with a cleaner join()
        fn format_key_events(key_events: &Vec<String>) -> String {
            let mut result_string: String = String::new();
            for key_event in key_events {
                if !result_string.is_empty() {
                    result_string.push_str(", ");
                }
                result_string.push_str(key_event);
            }
            result_string
        }
    }
}

pub fn command_palette(cx: &mut Context) {
    cx.callback = Some(Box::new(
        move |compositor: &mut Compositor, cx: &mut compositor::Context| {
            let command_list_key_events = compositor.find::<ui::ui_treeView>().unwrap().keymap.command_list(&cx.ui_tree.mode);
            let command_list = helix_view::command::COMMAND_LIST.to_vec();
            let picker = Picker::new(command_list, command_list_key_events, move |cx, command, _action| {
                let mut ctx = Context {
                    ui_tree.register: None,
                    ui_tree: cx.ui_tree
                    callback: None,
                    on_next_key_callback: None,
                    jobs: ui_tree.jobs,
                };
                command.execute(&mut ctx);
            });
            compositor.push(Box::new(overlayed(picker)));
        },
    ));

}

fn last_picker(cx: &mut Context) {
    // TODO: last picker does not seem to work well with buffer_picker
    cx.callback = Some(Box::new(|compositor, cx| {
        if let Some(picker) = compositor.last_picker.take() {
            compositor.push(picker);
        } else {
            cx.ui_tree.set_error("no last picker")
        }
    }));
}

// I inserts at the first nonwhitespace character of each line with a selection
fn insert_at_line_start(cx: &mut Context) {
    goto_first_nonwhitespace(cx);
    enter_insert_mode(cx);
}

// A inserts at the end of each line with a selection
fn insert_at_line_end(cx: &mut Context) {
    enter_insert_mode(cx);
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        let text = buffer.text().slice(..);
        let line = range.cursor_line(text);
        let pos = line_end_char_index(&text, line);
        Range::new(pos, pos)
    });
    buffer.set_selection(buffer_view.view_id, selection);
}

// Creates an LspCallback that waits for formatting changes to be computed. When they're done,
// it applies them, but only if the doc hasn't changed.
//
// TODO: provide some way to cancel this, probably as part of a more general job cancellation
// scheme
async fn make_format_callback(
    buffer_id: BufferID,
    buffer_version: i32,
    buffer_view_id: BufferViewID,
    format: impl Future<Output = Result<Transaction, FormatterError>> + Send + 'static,
    write: Option<(Option<PathBuf>, bool)>,
) -> anyhow::Result<job::Callback> {
    let format = format.await;

    let call: job::Callback = Callback::ui_tree(Box::new(move |ui_tree| {
        if !ui_tree.buffers.contains_key(&buffer_id) || !ui_tree.tree.contains(buffer_view_id) {
            return;
        }

        let scrolloff = ui_tree.config().scrolloff;
        let buffer = buffer_mut!(ui_tree, &buffer_id);
        let buffer_view = buffer_view_mut!(ui_tree, buffer_view_id);

        if let Ok(format) = format {
            if buffer.version() == buffer_version {
                apply_transaction(&format, buffer, buffer_view);
                buffer.append_changes_to_history(buffer_view);
                buffer.detect_indent_and_line_ending();
                buffer_view.ensure_cursor_in_view(buffer, scrolloff);
            } else {
                log::info!("discarded formatting changes because the document changed");
            }
        }

        if let Some((path, force)) = write {
            let id = buffer.id();
            if let Err(err) = ui_tree.save(id, path, force) {
                ui_tree.set_error(format!("Error saving: {}", err));
            }
        }
    }));

    Ok(call)
}

#[derive(PartialEq, Eq)]
pub enum Open {
    Below,
    Above,
}

fn open(cx: &mut Context, open: Open) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    enter_insert_mode(cx);
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let text = buffer.text().slice(..);
    let contents = buffer.text();
    let selection = buffer.selection(buffer_view.view_id);

    let mut ranges = SmallVec::with_capacity(selection.len());
    let mut offs = 0;

    let mut transaction = Transaction::change_by_selection(contents, selection, |range| {
        let cursor_line = text.char_to_line(match open {
            Open::Below => graphemes::prev_grapheme_boundary(text, range.to()),
            Open::Above => range.from(),
        });
        let new_line = match open {
            // adjust position to the end of the line (next line - 1)
            Open::Below => cursor_line + 1,
            // adjust position to the end of the previous line (current line - 1)
            Open::Above => cursor_line,
        };

        // Index to insert newlines after, as well as the char width
        // to use to compensate for those inserted newlines.
        let (line_end_index, line_end_offset_width) = if new_line == 0 {
            (0, 0)
        } else {
            (
                line_end_char_index(&buffer.text().slice(..), new_line.saturating_sub(1)),
                buffer.line_ending.len_chars(),
            )
        };

        let indent = indent::indent_for_newline(
            buffer.language_config(),
            buffer.syntax(),
            &buffer.indent_style,
            buffer.tab_width(),
            text,
            new_line.saturating_sub(1),
            line_end_index,
            cursor_line,
        );
        let indent_len = indent.len();
        let mut text = String::with_capacity(1 + indent_len);
        text.push_str(buffer.line_ending.as_str());
        text.push_str(&indent);
        let text = text.repeat(count);

        // calculate new selection ranges
        let pos = offs + line_end_index + line_end_offset_width;
        for i in 0..count {
            // pos                    -> beginning of reference line,
            // + (i * (1+indent_len)) -> beginning of i'th line from pos
            // + indent_len ->        -> indent for i'th line
            ranges.push(Range::point(pos + (i * (1 + indent_len)) + indent_len));
        }

        offs += text.chars().count();

        (line_end_index, line_end_index, Some(text.into()))
    });

    transaction = transaction.with_selection(Selection::new(ranges, selection.primary_index()));

    apply_transaction(&transaction, buffer, buffer_view);
}

// o inserts a new line after each line with a selection
fn open_below(cx: &mut Context) {
    open(cx, Open::Below)
}

// O inserts a new line before each line with a selection
fn open_above(cx: &mut Context) {
    open(cx, Open::Above)
}

fn normal_mode(cx: &mut Context) {
    cx.ui_tree.enter_normal_mode();
}

// Store a jump on the jumplist.
fn push_jump(buffer_view: &mut BufferView, buffer: &Buffer) {
    let jump = (buffer.id(), buffer.selection(buffer_view.view_id).clone());
    buffer_view.jumps.push(jump);
}

fn goto_line(cx: &mut Context) {
    goto_line_impl(cx.ui_tree)
}

fn goto_line_impl(ui_tree: &mut UITree) {
    if let Some(command_multiplier) = ui_tree.command_multiplier.get() {
        let (buffer_view, buffer) = current!(ui_tree);
        let text = buffer.text().slice(..);
        let max_line = if text.line(text.len_lines() - 1).len_chars() == 0 {
            // If the last line is blank, don't jump to it.
            text.len_lines().saturating_sub(2)
        } else {
            text.len_lines() - 1
        };
        let line_idx = std::cmp::min(command_multiplier.get() - 1, max_line);
        let pos = text.line_to_char(line_idx);
        let selection = buffer
            .selection(buffer_view.view_id)
            .clone()
            .transform(|range| range.put_cursor(text, pos, ui_tree.mode == Mode::Select));

        push_jump(buffer_view, buffer);
        buffer.set_selection(buffer_view.view_id, selection);
    }
}

fn goto_last_line(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);
    let line_idx = if text.line(text.len_lines() - 1).len_chars() == 0 {
        // If the last line is blank, don't jump to it.
        text.len_lines().saturating_sub(2)
    } else {
        text.len_lines() - 1
    };
    let pos = text.line_to_char(line_idx);
    let selection = buffer
        .selection(buffer_view.view_id)
        .clone()
        .transform(|range| range.put_cursor(text, pos, cx.ui_tree.mode == Mode::Select));

    push_jump(buffer_view, buffer);
    buffer.set_selection(buffer_view.view_id, selection);
}

fn goto_last_accessed_file(cx: &mut Context) {
    let buffer_view = buffer_view_mut!(cx.ui_tree);
    if let Some(alt) = buffer_view.docs_access_history.pop() {
        cx.ui_tree.switch(alt, Action::Replace);
    } else {
        cx.ui_tree.set_error("no last accessed buffer")
    }
}

fn goto_last_modification(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let pos = buffer.history.get_mut().last_edit_pos();
    let text = buffer.text().slice(..);
    if let Some(pos) = pos {
        let selection = buffer
            .selection(buffer_view.view_id)
            .clone()
            .transform(|range| range.put_cursor(text, pos, cx.ui_tree.mode == Mode::Select));
        buffer.set_selection(buffer_view.view_id, selection);
    }
}

fn goto_last_modified_file(cx: &mut Context) {
    let buffer_view = buffer_view!(cx.ui_tree);
    let alternate_file = buffer_view
        .last_modified_docs
        .into_iter()
        .flatten()
        .find(|&id| id != buffer_view.doc);
    if let Some(alt) = alternate_file {
        cx.ui_tree.switch(alt, Action::Replace);
    } else {
        cx.ui_tree.set_error("no last modified buffer")
    }
}

fn select_mode(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    // Make sure end-of-document selections are also 1-width.
    // (With the exception of being in an empty document, of course.)
    let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
        if range.is_empty() && range.head == text.len_chars() {
            Range::new(
                graphemes::prev_grapheme_boundary(text, range.anchor),
                range.head,
            )
        } else {
            range
        }
    });
    buffer.set_selection(buffer_view.view_id, selection);

    cx.ui_tree.mode = Mode::Select;
}

fn exit_select_mode(cx: &mut Context) {
    if cx.ui_tree.mode == Mode::Select {
        cx.ui_tree.mode = Mode::Normal;
    }
}

fn goto_pos(ui_tree: &mut UITree, pos: usize) {
    let (buffer_view, buffer) = current!(ui_tree);

    push_jump(buffer_view, doc);
    buffer.set_selection(buffer_view.view_id, Selection::point(pos));
    align_view(buffer, buffer_view, Align::Center);
}

fn goto_first_diag(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let selection = match buffer.diagnostics().first() {
        Some(diag) => Selection::single(diag.range.start, diag.range.end),
        None => return,
    };
    buffer.set_selection(buffer_view.view_id, selection);
    align_view(buffer, buffer_view, Align::Center);
}

fn goto_last_diag(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let selection = match buffer.diagnostics().last() {
        Some(diag) => Selection::single(diag.range.start, diag.range.end),
        None => return,
    };
    buffer.set_selection(buffer_view.view_id, selection);
    align_view(buffer, buffer_view, Align::Center);
}

fn goto_next_diag(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let cursor_pos = buffer
        .selection(buffer_view.view_id)
        .primary()
        .cursor(buffer.text().slice(..));

    let diag = buffer
        .diagnostics()
        .iter()
        .find(|diag| diag.range.start > cursor_pos)
        .or_else(|| buffer.diagnostics().first());

    let selection = match diag {
        Some(diag) => Selection::single(diag.range.start, diag.range.end),
        None => return,
    };
    buffer.set_selection(buffer_view.view_id, selection);
    align_view(buffer, buffer_view, Align::Center);
}

fn goto_prev_diag(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    let cursor_pos = buffer
        .selection(buffer_view.view_id)
        .primary()
        .cursor(buffer.text().slice(..));

    let diag = buffer
        .diagnostics()
        .iter()
        .rev()
        .find(|diag| diag.range.start < cursor_pos)
        .or_else(|| buffer.diagnostics().last());

    let selection = match diag {
        // NOTE: the selection is reversed because we're jumping to the
        // previous diagnostic.
        Some(diag) => Selection::single(diag.range.end, diag.range.start),
        None => return,
    };
    buffer.set_selection(buffer_view.view_id, selection);
    align_view(buffer, buffer_view, Align::Center);
}

fn goto_first_change(cx: &mut Context) {
    goto_first_change_impl(cx, false);
}

fn goto_last_change(cx: &mut Context) {
    goto_first_change_impl(cx, true);
}

fn goto_first_change_impl(cx: &mut Context, reverse: bool) {
    let ui_tree = &mut cx.ui_tree;
    let (_, doc) = current!(ui_tree);
    if let Some(handle) = buffer.diff_handle() {
        let hunk = {
            let hunks = handle.hunks();
            let idx = if reverse {
                hunks.len().saturating_sub(1)
            } else {
                0
            };
            hunks.nth_hunk(idx)
        };
        if hunk != Hunk::NONE {
            let pos = buffer.text().line_to_char(hunk.after.start as usize);
            goto_pos(ui_tree, pos)
        }
    }
}

fn goto_next_change(cx: &mut Context) {
    goto_next_change_impl(cx, Direction::Forward)
}

fn goto_prev_change(cx: &mut Context) {
    goto_next_change_impl(cx, Direction::Backward)
}

fn goto_next_change_impl(cx: &mut Context, direction: Direction) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let motion = move |ui_tree: &mut UITree| {
        let (buffer_view, buffer) = current!(ui_tree);
        let buffer_text = buffer.text().slice(..);
        let diff_handle = if let Some(diff_handle) = buffer.diff_handle() {
            diff_handle
        } else {
            ui_tree.set_status("Diff is not available in current buffer");
            return;
        };

        let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
            let cursor_line = range.cursor_line(buffer_text) as u32;

            let hunks = diff_handle.hunks();
            let hunk_idx = match direction {
                Direction::Forward => hunks
                    .next_hunk(cursor_line)
                    .map(|idx| (idx + count).min(hunks.len() - 1)),
                Direction::Backward => hunks
                    .prev_hunk(cursor_line)
                    .map(|idx| idx.saturating_sub(count)),
            };
            // TODO refactor with let..else once MSRV reaches 1.65
            let hunk_idx = if let Some(hunk_idx) = hunk_idx {
                hunk_idx
            } else {
                return range;
            };
            let hunk = hunks.nth_hunk(hunk_idx);

            let hunk_start = buffer_text.line_to_char(hunk.after.start as usize);
            let hunk_end = if hunk.after.is_empty() {
                hunk_start + 1
            } else {
                buffer_text.line_to_char(hunk.after.end as usize)
            };
            let new_range = Range::new(hunk_start, hunk_end);
            if ui_tree.mode == Mode::Select {
                let head = if new_range.head < range.anchor {
                    new_range.anchor
                } else {
                    new_range.head
                };

                Range::new(range.anchor, head)
            } else {
                new_range.with_direction(direction)
            }
        });

        buffer.set_selection(buffer_view.view_id, selection)
    };
    motion(cx.ui_tree);
    cx.ui_tree.last_motion = Some(Motion(Box::new(motion)));
}

pub mod insert {
    use super::*;
    pub type Hook = fn(&Rope, &Selection, char) -> Option<Transaction>;
    pub type PostHook = fn(&mut Context, char);

    /// Exclude the cursor in range.
    fn exclude_cursor(text: RopeSlice, range: Range, cursor: Range) -> Range {
        if range.to() == cursor.to() && text.len_chars() != cursor.to() {
            Range::new(
                range.from(),
                graphemes::prev_grapheme_boundary(text, cursor.to()),
            )
        } else {
            range
        }
    }

    // It trigger completion when idle timer reaches deadline
    // Only trigger completion if the word under cursor is longer than n characters
    pub fn idle_completion(cx: &mut Context) {
        let config = cx.ui_tree.config();
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let text = buffer.text().slice(..);
        let cursor = buffer.selection(buffer_view.view_id).primary().cursor(text);

        use helix_core::chars::char_is_word;
        let mut iter = text.chars_at(cursor);
        iter.reverse();
        for _ in 0..config.completion_trigger_len {
            match iter.next() {
                Some(c) if char_is_word(c) => {}
                _ => return,
            }
        }
        super::completion(cx);
    }

    fn language_server_completion(cx: &mut Context, ch: char) {
        let config = cx.ui_tree.config();
        if !config.auto_completion {
            return;
        }

        use helix_lsp::lsp;
        // if ch matches completion char, trigger completion
        let buffer = buffer_mut!(cx.ui_tree);
        let language_server = match buffer.language_server() {
            Some(language_server) => language_server,
            None => return,
        };

        let capabilities = language_server.capabilities();

        if let Some(lsp::CompletionOptions {
            trigger_characters: Some(triggers),
            ..
        }) = &capabilities.completion_provider
        {
            // TODO: what if trigger is multiple chars long
            if triggers.iter().any(|trigger| trigger.contains(ch)) {
                cx.ui_tree.clear_idle_timer();
                super::completion(cx);
            }
        }
    }

    fn signature_help(cx: &mut Context, ch: char) {
        use helix_lsp::lsp;
        // if ch matches signature_help char, trigger
        let buffer = buffer_mut!(cx.ui_tree);
        // The language_server!() macro is not used here since it will
        // print an "LSP not active for current buffer" message on
        // every keypress.
        let language_server = match buffer.language_server() {
            Some(language_server) => language_server,
            None => return,
        };

        let capabilities = language_server.capabilities();

        if let lsp::ServerCapabilities {
            signature_help_provider:
                Some(lsp::SignatureHelpOptions {
                    trigger_characters: Some(triggers),
                    // TODO: retrigger_characters
                    ..
                }),
            ..
        } = capabilities
        {
            // TODO: what if trigger is multiple chars long
            let is_trigger = triggers.iter().any(|trigger| trigger.contains(ch));
            // lsp doesn't tell us when to close the signature help, so we request
            // the help information again after common close triggers which should
            // return None, which in turn closes the popup.
            let close_triggers = &[')', ';', '.'];

            if is_trigger || close_triggers.contains(&ch) {
                super::signature_help_impl(cx, SignatureHelpInvoked::Automatic);
            }
        }
    }

    // The default insert hook: simply insert the character
    #[allow(clippy::unnecessary_wraps)] // need to use Option<> because of the Hook signature
    fn insert(doc: &Rope, selection: &Selection, ch: char) -> Option<Transaction> {
        let cursors = selection.clone().cursors(buffer.slice(..));
        let mut t = Tendril::new();
        t.push(ch);
        let transaction = Transaction::insert(buffer, &cursors, t);
        Some(transaction)
    }

    use helix_core::auto_pairs;

    pub fn insert_char(cx: &mut Context, c: char) {
        let (buffer_view, buffer) = current_ref!(cx.ui_tree);
        let text = buffer.text();
        let selection = buffer.selection(buffer_view.view_id);
        let auto_pairs = buffer.auto_pairs(cx.ui_tree);

        let transaction = auto_pairs
            .as_ref()
            .and_then(|ap| auto_pairs::hook(text, selection, c, ap))
            .or_else(|| insert(text, selection, c));

        let (buffer_view, buffer) = current!(cx.ui_tree);
        if let Some(t) = transaction {
            apply_transaction(&t, buffer, buffer_view);
        }

        // TODO: need a post insert hook too for certain triggers (autocomplete, signature help, etc)
        // this could also generically look at Transaction, but it's a bit annoying to look at
        // Operation instead of Change.
        for hook in &[language_server_completion, signature_help] {
            hook(cx, c);
        }
    }

    pub fn insert_tab(cx: &mut Context) {
        let (buffer_view, buffer) = current!(cx.ui_tree);
        // TODO: round out to nearest indentation level (for example a line with 3 spaces should
        // indent by one to reach 4 spaces).

        let indent = Tendril::from(buffer.indent_style.as_str());
        let transaction = Transaction::insert(
            buffer.text(),
            &buffer.selection(buffer_view.view_id).clone().cursors(buffer.text().slice(..)),
            indent,
        );
        apply_transaction(&transaction, buffer, buffer_view);
    }

    pub fn insert_newline(cx: &mut Context) {
        let (buffer_view, buffer) = current_ref!(cx.ui_tree);
        let text = buffer.text().slice(..);

        let contents = buffer.text();
        let selection = buffer.selection(buffer_view.view_id).clone();
        let mut ranges = SmallVec::with_capacity(selection.len());

        // TODO: this is annoying, but we need to do it to properly calculate pos after edits
        let mut global_offs = 0;

        let mut transaction = Transaction::change_by_selection(contents, &selection, |range| {
            let pos = range.cursor(text);

            let prev = if pos == 0 {
                ' '
            } else {
                contents.char(pos - 1)
            };
            let curr = contents.get_char(pos).unwrap_or(' ');

            let current_line = text.char_to_line(pos);
            let line_is_only_whitespace = text
                .line(current_line)
                .chars()
                .all(|char| char.is_ascii_whitespace());

            let mut new_text = String::new();

            // If the current line is all whitespace, insert a line ending at the beginning of
            // the current line. This makes the current line empty and the new line contain the
            // indentation of the old line.
            let (from, to, local_offs) = if line_is_only_whitespace {
                let line_start = text.line_to_char(current_line);
                new_text.push_str(buffer.line_ending.as_str());

                (line_start, line_start, new_text.chars().count())
            } else {
                let indent = indent::indent_for_newline(
                    buffer.language_config(),
                    buffer.syntax(),
                    &buffer.indent_style,
                    buffer.tab_width(),
                    text,
                    current_line,
                    pos,
                    current_line,
                );

                // If we are between pairs (such as brackets), we want to
                // insert an additional line which is indented one level
                // more and place the cursor there
                let on_auto_pair = buffer
                    .auto_pairs(cx.ui_tree)
                    .and_then(|pairs| pairs.get(prev))
                    .and_then(|pair| if pair.close == curr { Some(pair) } else { None })
                    .is_some();

                let local_offs = if on_auto_pair {
                    let inner_indent = indent.clone() + buffer.indent_style.as_str();
                    new_text.reserve_exact(2 + indent.len() + inner_indent.len());
                    new_text.push_str(buffer.line_ending.as_str());
                    new_text.push_str(&inner_indent);
                    let local_offs = new_text.chars().count();
                    new_text.push_str(buffer.line_ending.as_str());
                    new_text.push_str(&indent);
                    local_offs
                } else {
                    new_text.reserve_exact(1 + indent.len());
                    new_text.push_str(buffer.line_ending.as_str());
                    new_text.push_str(&indent);
                    new_text.chars().count()
                };

                (pos, pos, local_offs)
            };

            let new_range = if buffer.restore_cursor {
                // when appending, extend the range by local_offs
                Range::new(
                    range.anchor + global_offs,
                    range.head + local_offs + global_offs,
                )
            } else {
                // when inserting, slide the range by local_offs
                Range::new(
                    range.anchor + local_offs + global_offs,
                    range.head + local_offs + global_offs,
                )
            };

            // TODO: range replace or extend
            // range.replace(|range| range.is_empty(), head); -> fn extend if cond true, new head pos
            // can be used with cx.mode to do replace or extend on most changes
            ranges.push(new_range);
            global_offs += new_text.chars().count();

            (from, to, Some(new_text.into()))
        });

        transaction = transaction.with_selection(Selection::new(ranges, selection.primary_index()));

        let (buffer_view, buffer) = current!(cx.ui_tree);
        apply_transaction(&transaction, buffer, buffer_view);
    }

    pub fn delete_char_backward(cx: &mut Context) {
        let count = ui_tree.command_multiplier.unwrap_or_one().get();
        let (buffer_view, buffer) = current_ref!(cx.ui_tree);
        let text = buffer.text().slice(..);
        let indent_unit = buffer.indent_style.as_str();
        let tab_size = buffer.tab_width();
        let auto_pairs = buffer.auto_pairs(cx.ui_tree);

        let transaction =
            Transaction::change_by_selection(buffer.text(), buffer.selection(buffer_view.view_id), |range| {
                let pos = range.cursor(text);
                if pos == 0 {
                    return (pos, pos, None);
                }
                let line_start_pos = text.line_to_char(range.cursor_line(text));
                // consider to delete by indent level if all characters before `pos` are indent units.
                let fragment = Cow::from(text.slice(line_start_pos..pos));
                if !fragment.is_empty() && fragment.chars().all(|ch| ch == ' ' || ch == '\t') {
                    if text.get_char(pos.saturating_sub(1)) == Some('\t') {
                        // fast path, delete one char
                        (
                            graphemes::nth_prev_grapheme_boundary(text, pos, 1),
                            pos,
                            None,
                        )
                    } else {
                        let unit_len = indent_unit.chars().count();
                        // NOTE: indent_unit always contains 'only spaces' or 'only tab' according to `IndentStyle` definition.
                        let unit_size = if indent_unit.starts_with('\t') {
                            tab_size * unit_len
                        } else {
                            unit_len
                        };
                        let width: usize = fragment
                            .chars()
                            .map(|ch| {
                                if ch == '\t' {
                                    tab_size
                                } else {
                                    // it can be none if it still meet control characters other than '\t'
                                    // here just set the width to 1 (or some value better?).
                                    ch.width().unwrap_or(1)
                                }
                            })
                            .sum();
                        let mut drop = width % unit_size; // round down to nearest unit
                        if drop == 0 {
                            drop = unit_size
                        }; // if it's already at a unit, consume a whole unit
                        let mut chars = fragment.chars().rev();
                        let mut start = pos;
                        for _ in 0..drop {
                            // delete up to `drop` spaces
                            match chars.next() {
                                Some(' ') => start -= 1,
                                _ => break,
                            }
                        }
                        (start, pos, None) // delete!
                    }
                } else {
                    match (
                        text.get_char(pos.saturating_sub(1)),
                        text.get_char(pos),
                        auto_pairs,
                    ) {
                        (Some(_x), Some(_y), Some(ap))
                            if range.is_single_grapheme(text)
                                && ap.get(_x).is_some()
                                && ap.get(_x).unwrap().open == _x
                                && ap.get(_x).unwrap().close == _y =>
                        // delete both autopaired characters
                        {
                            (
                                graphemes::nth_prev_grapheme_boundary(text, pos, count),
                                graphemes::nth_next_grapheme_boundary(text, pos, count),
                                None,
                            )
                        }
                        _ =>
                        // delete 1 char
                        {
                            (
                                graphemes::nth_prev_grapheme_boundary(text, pos, count),
                                pos,
                                None,
                            )
                        }
                    }
                }
            });
        let (buffer_view, buffer) = current!(cx.ui_tree);
        apply_transaction(&transaction, buffer, buffer_view);

        lsp::signature_help_impl(cx, SignatureHelpInvoked::Automatic);
    }

    pub fn delete_char_forward(cx: &mut Context) {
        let count = ui_tree.command_multiplier.unwrap_or_one().get();
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let text = buffer.text().slice(..);
        let transaction =
            Transaction::change_by_selection(buffer.text(), buffer.selection(buffer_view.view_id), |range| {
                let pos = range.cursor(text);
                (
                    pos,
                    graphemes::nth_next_grapheme_boundary(text, pos, count),
                    None,
                )
            });
        apply_transaction(&transaction, buffer, buffer_view);

        lsp::signature_help_impl(cx, SignatureHelpInvoked::Automatic);
    }

    pub fn delete_word_backward(cx: &mut Context) {
        let count = ui_tree.command_multiplier.unwrap_or_one().get();
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let text = buffer.text().slice(..);

        let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
            let anchor = movement::move_prev_word_start(text, range, count).from();
            let next = Range::new(anchor, range.cursor(text));
            exclude_cursor(text, next, range)
        });
        delete_selection_insert_mode(buffer, buffer_view, &selection);

        lsp::signature_help_impl(cx, SignatureHelpInvoked::Automatic);
    }

    pub fn delete_word_forward(cx: &mut Context) {
        let count = ui_tree.command_multiplier.unwrap_or_one().get();
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let text = buffer.text().slice(..);

        let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
            let head = movement::move_next_word_end(text, range, count).to();
            Range::new(range.cursor(text), head)
        });

        delete_selection_insert_mode(buffer, buffer_view, &selection);

        lsp::signature_help_impl(cx, SignatureHelpInvoked::Automatic);
    }
}

// Undo / Redo

fn undo(cx: &mut Context) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    for _ in 0..count {
        if !buffer.undo(buffer_view) {
            cx.ui_tree.set_status("Already at oldest change");
            break;
        }
    }
}

fn redo(cx: &mut Context) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    for _ in 0..count {
        if !buffer.redo(buffer_view) {
            cx.ui_tree.set_status("Already at newest change");
            break;
        }
    }
}

fn earlier(cx: &mut Context) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    for _ in 0..count {
        // rather than doing in batch we do this so get error halfway
        if !buffer.earlier(buffer_view, UndoKind::Steps(1)) {
            cx.ui_tree.set_status("Already at oldest change");
            break;
        }
    }
}

fn later(cx: &mut Context) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    for _ in 0..count {
        // rather than doing in batch we do this so get error halfway
        if !buffer.later(buffer_view, UndoKind::Steps(1)) {
            cx.ui_tree.set_status("Already at newest change");
            break;
        }
    }
}

fn commit_undo_checkpoint(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    buffer.append_changes_to_history(buffer_view);
}

// Yank / Paste

fn yank(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let values: Vec<String> = buffer
        .selection(buffer_view.view_id)
        .fragments(text)
        .map(Cow::into_owned)
        .collect();

    let msg = format!(
        "yanked {} selection(s) to register {}",
        values.len(),
        cx.ui_tree.selected_register.unwrap_or('"')
    );

    cx.ui_tree
        .registers
        .write(cx.ui_tree.selected_register.unwrap_or('"'), values);

    cx.ui_tree.set_status(msg);
    exit_select_mode(cx);
}

fn yank_joined_to_clipboard_impl(
    ui_tree: &mut UITree,
    separator: &str,
    clipboard_type: ClipboardType,
) -> anyhow::Result<()> {
    let (buffer_view, buffer) = current!(ui_tree);
    let text = buffer.text().slice(..);

    let values: Vec<String> = buffer
        .selection(buffer_view.view_id)
        .fragments(text)
        .map(Cow::into_owned)
        .collect();

    let clipboard_text = match clipboard_type {
        ClipboardType::Clipboard => "system clipboard",
        ClipboardType::Selection => "primary clipboard",
    };

    let msg = format!(
        "joined and yanked {} selection(s) to {}",
        values.len(),
        clipboard_text,
    );

    let joined = values.join(separator);

    ui_tree
        .clipboard_provider
        .set_contents(joined, clipboard_type)
        .context("Couldn't set system clipboard content")?;

    ui_tree.set_status(msg);

    Ok(())
}

fn yank_joined_to_clipboard(cx: &mut Context) {
    let line_ending = buffer!(cx.ui_tree).line_ending;
    let _ =
        yank_joined_to_clipboard_impl(cx.ui_tree, line_ending.as_str(), ClipboardType::Clipboard);
    exit_select_mode(cx);
}

fn yank_main_selection_to_clipboard_impl(
    ui_tree: &mut UITree,
    clipboard_type: ClipboardType,
) -> anyhow::Result<()> {
    let (buffer_view, buffer) = current!(ui_tree);
    let text = buffer.text().slice(..);

    let message_text = match clipboard_type {
        ClipboardType::Clipboard => "yanked main selection to system clipboard",
        ClipboardType::Selection => "yanked main selection to primary clipboard",
    };

    let value = buffer.selection(buffer_view.view_id).primary().fragment(text);

    if let Err(e) = ui_tree
        .clipboard_provider
        .set_contents(value.into_owned(), clipboard_type)
    {
        bail!("Couldn't set system clipboard content: {}", e);
    }

    ui_tree.set_status(message_text);
    Ok(())
}

fn yank_main_selection_to_clipboard(cx: &mut Context) {
    let _ = yank_main_selection_to_clipboard_impl(cx.ui_tree, ClipboardType::Clipboard);
}

fn yank_joined_to_primary_clipboard(cx: &mut Context) {
    let line_ending = buffer!(cx.ui_tree).line_ending;
    let _ =
        yank_joined_to_clipboard_impl(cx.ui_tree, line_ending.as_str(), ClipboardType::Selection);
}

fn yank_main_selection_to_primary_clipboard(cx: &mut Context) {
    let _ = yank_main_selection_to_clipboard_impl(cx.ui_tree, ClipboardType::Selection);
    exit_select_mode(cx);
}

#[derive(Copy, Clone)]
enum Paste {
    Before,
    After,
    Cursor,
}

fn paste_impl(
    values: &[String],
    buffer: &mut Buffer,
    buffer_view: &mut BufferView,
    action: Paste,
    count: usize,
    mode: Mode,
) {
    if values.is_empty() {
        return;
    }

    let repeat = std::iter::repeat(
        // `values` is asserted to have at least one entry above.
        values
            .last()
            .map(|value| Tendril::from(value.repeat(count)))
            .unwrap(),
    );

    // if any of values ends with a line ending, it's linewise paste
    let linewise = values
        .iter()
        .any(|value| get_line_ending_of_str(value).is_some());

    // Only compiled once.
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\r\n|\r|\n").unwrap());
    let mut values = values
        .iter()
        .map(|value| REGEX.replace_all(value, buffer.line_ending.as_str()))
        .map(|value| Tendril::from(value.as_ref().repeat(count)))
        .chain(repeat);

    let text = buffer.text();
    let selection = buffer.selection(buffer_view.view_id);

    let mut offset = 0;
    let mut ranges = SmallVec::with_capacity(selection.len());

    let mut transaction = Transaction::change_by_selection(text, selection, |range| {
        let pos = match (action, linewise) {
            // paste linewise before
            (Paste::Before, true) => text.line_to_char(text.char_to_line(range.from())),
            // paste linewise after
            (Paste::After, true) => {
                let line = range.line_range(text.slice(..)).1;
                text.line_to_char((line + 1).min(text.len_lines()))
            }
            // paste insert
            (Paste::Before, false) => range.from(),
            // paste append
            (Paste::After, false) => range.to(),
            // paste at cursor
            (Paste::Cursor, _) => range.cursor(text.slice(..)),
        };

        let value = values.next();

        let value_len = value
            .as_ref()
            .map(|content| content.chars().count())
            .unwrap_or_default();
        let anchor = offset + pos;

        let new_range = Range::new(anchor, anchor + value_len).with_direction(range.direction());
        ranges.push(new_range);
        offset += value_len;

        (pos, pos, value)
    });

    if mode == Mode::Normal {
        transaction = transaction.with_selection(Selection::new(ranges, selection.primary_index()));
    }

    apply_transaction(&transaction, buffer, buffer_view);
}

pub(crate) fn paste_bracketed_value(cx: &mut Context, contents: String) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let paste = match cx.ui_tree.mode {
        Mode::Insert | Mode::Select => Paste::Cursor,
        Mode::Normal => Paste::Before,
    };
    let (buffer_view, buffer) = current!(cx.ui_tree);
    paste_impl(&[contents], buffer, buffer_view, paste, count, cx.ui_tree.mode);
}

fn paste_clipboard_impl(
    ui_tree: &mut UITree,
    action: Paste,
    clipboard_type: ClipboardType,
    count: usize,
) -> anyhow::Result<()> {
    let (buffer_view, buffer) = current!(ui_tree);
    match ui_tree.clipboard_provider.get_contents(clipboard_type) {
        Ok(contents) => {
            paste_impl(&[contents], buffer, buffer_view, action, count, ui_tree.mode);
            Ok(())
        }
        Err(e) => Err(e.context("Couldn't get system clipboard contents")),
    }
}

fn paste_clipboard_after(cx: &mut Context) {
    let _ = paste_clipboard_impl(
        cx.ui_tree,
        Paste::After,
        ClipboardType::Clipboard,
        ui_tree.command_multiplier.unwrap_or_one().get(),
    );
}

fn paste_clipboard_before(cx: &mut Context) {
    let _ = paste_clipboard_impl(
        cx.ui_tree,
        Paste::Before,
        ClipboardType::Clipboard,
        ui_tree.command_multiplier.unwrap_or_one().get(),
    );
}

fn paste_primary_clipboard_after(cx: &mut Context) {
    let _ = paste_clipboard_impl(
        cx.ui_tree,
        Paste::After,
        ClipboardType::Selection,
        ui_tree.command_multiplier.unwrap_or_one().get(),
    );
}

fn paste_primary_clipboard_before(cx: &mut Context) {
    let _ = paste_clipboard_impl(
        cx.ui_tree,
        Paste::Before,
        ClipboardType::Selection,
        ui_tree.command_multiplier.unwrap_or_one().get(),
    );
}

fn replace_with_yanked(cx: &mut Context) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let reg_name = cx.ui_tree.selected_register.unwrap_or('"');
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let registers = &mut cx.ui_tree.registers;

    if let Some(values) = registers.read(reg_name) {
        if !values.is_empty() {
            let repeat = std::iter::repeat(
                values
                    .last()
                    .map(|value| Tendril::from(&value.repeat(count)))
                    .unwrap(),
            );
            let mut values = values
                .iter()
                .map(|value| Tendril::from(&value.repeat(count)))
                .chain(repeat);
            let selection = buffer.selection(buffer_view.view_id);
            let transaction = Transaction::change_by_selection(buffer.text(), selection, |range| {
                if !range.is_empty() {
                    (range.from(), range.to(), Some(values.next().unwrap()))
                } else {
                    (range.from(), range.to(), None)
                }
            });

            apply_transaction(&transaction, buffer, buffer_view);
            exit_select_mode(cx);
        }
    }
}

fn replace_selections_with_clipboard_impl(
    cx: &mut Context,
    clipboard_type: ClipboardType,
) -> anyhow::Result<()> {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);

    match cx.ui_tree.clipboard_provider.get_contents(clipboard_type) {
        Ok(contents) => {
            let selection = buffer.selection(buffer_view.view_id);
            let transaction = Transaction::change_by_selection(buffer.text(), selection, |range| {
                (
                    range.from(),
                    range.to(),
                    Some(contents.repeat(count).as_str().into()),
                )
            });

            apply_transaction(&transaction, buffer, buffer_view);
            buffer.append_changes_to_history(buffer_view);
        }
        Err(e) => return Err(e.context("Couldn't get system clipboard contents")),
    }

    exit_select_mode(cx);
    Ok(())
}

fn replace_selections_with_clipboard(cx: &mut Context) {
    let _ = replace_selections_with_clipboard_impl(cx, ClipboardType::Clipboard);
}

fn replace_selections_with_primary_clipboard(cx: &mut Context) {
    let _ = replace_selections_with_clipboard_impl(cx, ClipboardType::Selection);
}

fn paste(cx: &mut Context, pos: Paste) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let reg_name = cx.ui_tree.selected_register.unwrap_or('"');
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let registers = &mut cx.ui_tree.registers;

    if let Some(values) = registers.read(reg_name) {
        paste_impl(values, buffer, buffer_view, pos, count, cx.ui_tree.mode);
    }
}

fn paste_after(cx: &mut Context) {
    paste(cx, Paste::After)
}

fn paste_before(cx: &mut Context) {
    paste(cx, Paste::Before)
}

fn get_lines(doc: &Buffer, buffer_view_id: BufferViewID) -> Vec<usize> {
    let mut lines = Vec::new();

    // Get all line numbers
    for range in buffer.selection(buffer_view_id) {
        let (start, end) = range.line_range(buffer.text().slice(..));

        for line in start..=end {
            lines.push(line)
        }
    }
    lines.sort_unstable(); // sorting by usize so _unstable is preferred
    lines.dedup();
    lines
}

fn indent(cx: &mut Context) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let lines = get_lines(buffer, buffer_view.view_id);

    // Indent by one level
    let indent = Tendril::from(buffer.indent_style.as_str().repeat(count));

    let transaction = Transaction::change(
        buffer.text(),
        lines.into_iter().filter_map(|line| {
            let is_blank = buffer.text().line(line).chunks().all(|s| s.trim().is_empty());
            if is_blank {
                return None;
            }
            let pos = buffer.text().line_to_char(line);
            Some((pos, pos, Some(indent.clone())))
        }),
    );
    apply_transaction(&transaction, buffer, buffer_view);
}

fn unindent(cx: &mut Context) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let lines = get_lines(buffer, buffer_view.view_id);
    let mut changes = Vec::with_capacity(lines.len());
    let tab_width = buffer.tab_width();
    let indent_width = count * tab_width;

    for line_idx in lines {
        let line = buffer.text().line(line_idx);
        let mut width = 0;
        let mut pos = 0;

        for ch in line.chars() {
            match ch {
                ' ' => width += 1,
                '\t' => width = (width / tab_width + 1) * tab_width,
                _ => break,
            }

            pos += 1;

            if width >= indent_width {
                break;
            }
        }

        // now delete from start to first non-blank
        if pos > 0 {
            let start = buffer.text().line_to_char(line_idx);
            changes.push((start, start + pos, None))
        }
    }

    let transaction = Transaction::change(buffer.text(), changes.into_iter());

    apply_transaction(&transaction, buffer, buffer_view);
}

fn format_selections(cx: &mut Context) {
    use helix_lsp::{lsp, util::range_to_lsp_range};

    let (buffer_view, buffer) = current!(cx.ui_tree);

    // via lsp if available
    // TODO: else via tree-sitter indentation calculations

    let language_server = match buffer.language_server() {
        Some(language_server) => language_server,
        None => return,
    };

    let ranges: Vec<lsp::Range> = buffer
        .selection(buffer_view.view_id)
        .iter()
        .map(|range| range_to_lsp_range(buffer.text(), *range, language_server.offset_encoding()))
        .collect();

    if ranges.len() != 1 {
        cx.ui_tree
            .set_error("format_selections only supports a single selection for now");
        return;
    }

    // TODO: handle fails
    // TODO: concurrent map over all ranges

    let range = ranges[0];

    let request = match language_server.text_document_range_formatting(
        buffer.identifier(),
        range,
        lsp::FormattingOptions::default(),
        None,
    ) {
        Some(future) => future,
        None => {
            cx.ui_tree
                .set_error("Language server does not support range formatting");
            return;
        }
    };

    let edits = tokio::task::block_in_place(|| helix_lsp::block_on(request)).unwrap_or_default();

    let transaction = helix_lsp::util::generate_transaction_from_edits(
        buffer.text(),
        edits,
        language_server.offset_encoding(),
    );

    apply_transaction(&transaction, buffer, buffer_view);
}

fn join_selections_impl(cx: &mut Context, select_space: bool) {
    use movement::skip_while;
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text();
    let slice = buffer.text().slice(..);

    let mut changes = Vec::new();
    let fragment = Tendril::from(" ");

    for selection in buffer.selection(buffer_view.view_id) {
        let (start, mut end) = selection.line_range(slice);
        if start == end {
            end = (end + 1).min(text.len_lines() - 1);
        }
        let lines = start..end;

        changes.reserve(lines.len());

        for line in lines {
            let start = line_end_char_index(&slice, line);
            let mut end = text.line_to_char(line + 1);
            end = skip_while(slice, end, |ch| matches!(ch, ' ' | '\t')).unwrap_or(end);

            // need to skip from start, not end
            let change = (start, end, Some(fragment.clone()));
            changes.push(change);
        }
    }

    changes.sort_unstable_by_key(|(from, _to, _text)| *from);
    changes.dedup();

    // TODO: joining multiple empty lines should be replaced by a single space.
    // need to merge change ranges that touch

    // select inserted spaces
    let transaction = if select_space {
        let ranges: SmallVec<_> = changes
            .iter()
            .scan(0, |offset, change| {
                let range = Range::point(change.0 - *offset);
                *offset += change.1 - change.0 - 1; // -1 because cursor is 0-sized
                Some(range)
            })
            .collect();
        let selection = Selection::new(ranges, 0);
        Transaction::change(buffer.text(), changes.into_iter()).with_selection(selection)
    } else {
        Transaction::change(buffer.text(), changes.into_iter())
    };

    apply_transaction(&transaction, buffer, buffer_view);
}

fn keep_or_remove_selections_impl(cx: &mut Context, remove: bool) {
    // keep or remove selections matching regex
    let reg = cx.ui_tree.selected_register.unwrap_or('/');
    ui::regex_prompt(
        cx,
        if remove { "remove:" } else { "keep:" }.into(),
        Some(reg),
        ui::completers::none,
        move |ui_tree, regex, event| {
            let (buffer_view, buffer) = current!(ui_tree);
            if !matches!(event, PromptEvent::Update | PromptEvent::Validate) {
                return;
            }
            let text = buffer.text().slice(..);

            if let Some(selection) =
                selection::keep_or_remove_matches(text, buffer.selection(buffer_view.view_id), &regex, remove)
            {
                buffer.set_selection(buffer_view.view_id, selection);
            }
        },
    )
}

fn join_selections(cx: &mut Context) {
    join_selections_impl(cx, false)
}

fn join_selections_space(cx: &mut Context) {
    join_selections_impl(cx, true)
}

fn keep_selections(cx: &mut Context) {
    keep_or_remove_selections_impl(cx, false)
}

fn remove_selections(cx: &mut Context) {
    keep_or_remove_selections_impl(cx, true)
}

fn keep_primary_selection(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    // TODO: handle count

    let range = buffer.selection(buffer_view.view_id).primary();
    buffer.set_selection(buffer_view.view_id, Selection::single(range.anchor, range.head));
}

fn remove_primary_selection(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    // TODO: handle count

    let selection = buffer.selection(buffer_view.view_id);
    if selection.len() == 1 {
        cx.ui_tree.set_error("no selections remaining");
        return;
    }
    let index = selection.primary_index();
    let selection = selection.clone().remove(index);

    buffer.set_selection(buffer_view.view_id, selection);
}

pub fn completion(cx: &mut Context) {
    use helix_lsp::{lsp, util::pos_to_lsp_pos};

    let (buffer_view, buffer) = current!(cx.ui_tree);

    let language_server = match buffer.language_server() {
        Some(language_server) => language_server,
        None => return,
    };

    let offset_encoding = language_server.offset_encoding();
    let text = buffer.text().slice(..);
    let cursor = buffer.selection(buffer_view.view_id).primary().cursor(text);

    let pos = pos_to_lsp_pos(buffer.text(), cursor, offset_encoding);

    let future = match language_server.completion(buffer.identifier(), pos, None) {
        Some(future) => future,
        None => return,
    };

    let trigger_offset = cursor;

    // TODO: trigger_offset should be the cursor offset but we also need a starting offset from where we want to apply
    // completion filtering. For example logger.te| should filter the initial suggestion list with "te".

    use helix_core::chars;
    let mut iter = text.chars_at(cursor);
    iter.reverse();
    let offset = iter.take_while(|ch| chars::char_is_word(*ch)).count();
    let start_offset = cursor.saturating_sub(offset);

    cx.callback(
        future,
        move |ui_tree, compositor, response: Option<lsp::CompletionResponse>| {
            if ui_tree.mode != Mode::Insert {
                // we're not in insert mode anymore
                return;
            }

            let items = match response {
                Some(lsp::CompletionResponse::Array(items)) => items,
                // TODO: do something with is_incomplete
                Some(lsp::CompletionResponse::List(lsp::CompletionList {
                    is_incomplete: _is_incomplete,
                    items,
                })) => items,
                None => Vec::new(),
            };

            if items.is_empty() {
                // ui_tree.set_error("No completion available");
                return;
            }
            let size = compositor.size();
            let ui = compositor.find::<ui::ui_treeView>().unwrap();
            ui.set_completion(
                ui_tree,
                items,
                offset_encoding,
                start_offset,
                trigger_offset,
                size,
            );
        },
    );
}

// comments
fn toggle_comments(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let token = buffer
        .language_config()
        .and_then(|lc| lc.comment_token.as_ref())
        .map(|tc| tc.as_ref());
    let transaction = comment::toggle_line_comments(buffer.text(), buffer.selection(buffer_view.view_id), token);

    apply_transaction(&transaction, buffer, buffer_view);
    exit_select_mode(cx);
}

fn rotate_selections(cx: &mut Context, direction: Direction) {
    let count = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let mut selection = buffer.selection(buffer_view.view_id).clone();
    let index = selection.primary_index();
    let len = selection.len();
    selection.set_primary_index(match direction {
        Direction::Forward => (index + count) % len,
        Direction::Backward => (index + (len.saturating_sub(count) % len)) % len,
    });
    buffer.set_selection(buffer_view.view_id, selection);
}
fn rotate_selections_forward(cx: &mut Context) {
    rotate_selections(cx, Direction::Forward)
}
fn rotate_selections_backward(cx: &mut Context) {
    rotate_selections(cx, Direction::Backward)
}

fn rotate_selection_contents(cx: &mut Context, direction: Direction) {
    let command_multiplier = cx.ui_tree.command_multiplier;
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.view_id);
    let mut fragments: Vec<_> = selection
        .slices(text)
        .map(|fragment| fragment.chunks().collect())
        .collect();

    let group = command_multiplier
        .map(|command_multiplier| command_multiplier.get())
        .unwrap_or(fragments.len()) // default to rotating everything as one group
        .min(fragments.len());

    for chunk in fragments.chunks_mut(group) {
        // TODO: also modify main index
        match direction {
            Direction::Forward => chunk.rotate_right(1),
            Direction::Backward => chunk.rotate_left(1),
        };
    }

    let transaction = Transaction::change(
        buffer.text(),
        selection
            .ranges()
            .iter()
            .zip(fragments)
            .map(|(range, fragment)| (range.from(), range.to(), Some(fragment))),
    );

    apply_transaction(&transaction, buffer, buffer_view);
}

fn rotate_selection_contents_forward(cx: &mut Context) {
    rotate_selection_contents(cx, Direction::Forward)
}
fn rotate_selection_contents_backward(cx: &mut Context) {
    rotate_selection_contents(cx, Direction::Backward)
}

// tree sitter node selection

fn expand_selection(cx: &mut Context) {
    let motion = |ui_tree: &mut UITree| {
        let (buffer_view, buffer) = current!(ui_tree);

        if let Some(syntax) = buffer.syntax() {
            let text = buffer.text().slice(..);

            let current_selection = buffer.selection(buffer_view.view_id);
            let selection = object::expand_selection(syntax, text, current_selection.clone());

            // check if selection is different from the last one
            if *current_selection != selection {
                // save current selection so it can be restored using shrink_selection
                buffer_view.object_selections.push(current_selection.clone());

                buffer.set_selection(buffer_view.view_id, selection);
            }
        }
    };
    motion(cx.ui_tree);
    cx.ui_tree.last_motion = Some(Motion(Box::new(motion)));
}

fn shrink_selection(cx: &mut Context) {
    let motion = |ui_tree: &mut UITree| {
        let (buffer_view, buffer) = current!(ui_tree);
        let current_selection = buffer.selection(buffer_view.view_id);
        // try to restore previous selection
        if let Some(prev_selection) = buffer_view.object_selections.pop() {
            if current_selection.contains(&prev_selection) {
                // allow shrinking the selection only if current selection contains the previous object selection
                buffer.set_selection(buffer_view.view_id, prev_selection);
                return;
            } else {
                // clear existing selection as they can't be shrunk to anyway
                buffer_view.object_selections.clear();
            }
        }
        // if not previous selection, shrink to first child
        if let Some(syntax) = buffer.syntax() {
            let text = buffer.text().slice(..);
            let selection = object::shrink_selection(syntax, text, current_selection.clone());
            buffer.set_selection(buffer_view.view_id, selection);
        }
    };
    motion(cx.ui_tree);
    cx.ui_tree.last_motion = Some(Motion(Box::new(motion)));
}

fn select_sibling_impl<F>(cx: &mut Context, sibling_fn: &'static F)
where
    F: Fn(Node) -> Option<Node>,
{
    let motion = |ui_tree: &mut UITree| {
        let (buffer_view, buffer) = current!(ui_tree);

        if let Some(syntax) = buffer.syntax() {
            let text = buffer.text().slice(..);
            let current_selection = buffer.selection(buffer_view.view_id);
            let selection =
                object::select_sibling(syntax, text, current_selection.clone(), sibling_fn);
            buffer.set_selection(buffer_view.view_id, selection);
        }
    };
    motion(cx.ui_tree);
    cx.ui_tree.last_motion = Some(Motion(Box::new(motion)));
}

fn select_next_sibling(cx: &mut Context) {
    select_sibling_impl(cx, &|node| Node::next_sibling(&node))
}

fn select_prev_sibling(cx: &mut Context) {
    select_sibling_impl(cx, &|node| Node::prev_sibling(&node))
}

fn match_brackets(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);

    if let Some(syntax) = buffer.syntax() {
        let text = buffer.text().slice(..);
        let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
            if let Some(pos) =
                match_brackets::find_matching_bracket_fuzzy(syntax, buffer.text(), range.cursor(text))
            {
                range.put_cursor(text, pos, cx.ui_tree.mode == Mode::Select)
            } else {
                range
            }
        });
        buffer.set_selection(buffer_view.view_id, selection);
    }
}

//

fn jump_forward(cx: &mut Context) {
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();
    let config = cx.ui_tree.config();
    let buffer_view = buffer_view_mut!(cx.ui_tree);
    let buffer_id = buffer_view.buffer_id;

    if let Some((id, selection)) = buffer_view.jumps.forward(command_multiplier) {
        buffer_view.doc = *id;
        let selection = selection.clone();
        let (buffer_view, buffer) = current!(cx.ui); // refetch buffer

        if buffer.id() != buffer_id {
            buffer_view.add_to_history(buffer_id);
        }

        buffer.set_selection(buffer_view.view_id, selection);
        buffer_view.ensure_cursor_in_view_center(buffer, config.scrolloff);
    };
}

fn jump_backward(cx: &mut Context) {
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();
    let config = cx.ui_tree.config();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let buffer_id = buffer.id();

    if let Some((id, selection)) = buffer_view.jumps.backward(buffer_view.view_id, buffer, command_multiplier) {
        buffer_view.buffer_id = *id;
        let selection = selection.clone();
        let (buffer_view, buffer) = current!(cx.ui_tree); // refetch buffer

        if buffer.id() != buffer_id {
            buffer_view.add_to_history(buffer_id);
        }

        buffer.set_selection(buffer_view.view_id, selection);
        buffer_view.ensure_cursor_in_view_center(buffer, config.scrolloff);
    };
}

fn save_selection(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    push_jump(buffer_view, doc);
    cx.ui_tree.set_status("Selection saved to jumplist");
}

fn rotate_view(cx: &mut Context) {
    cx.ui_tree.focus_next()
}

fn jump_view_right(cx: &mut Context) {
    cx.ui_tree.focus_direction(tree::Direction::Right)
}

fn jump_view_left(cx: &mut Context) {
    cx.ui_tree.focus_direction(tree::Direction::Left)
}

fn jump_view_up(cx: &mut Context) {
    cx.ui_tree.focus_direction(tree::Direction::Up)
}

fn jump_view_down(cx: &mut Context) {
    cx.ui_tree.focus_direction(tree::Direction::Down)
}

fn swap_view_right(cx: &mut Context) {
    cx.ui_tree.swap_split_in_direction(tree::Direction::Right)
}

fn swap_view_left(cx: &mut Context) {
    cx.ui_tree.swap_split_in_direction(tree::Direction::Left)
}

fn swap_view_up(cx: &mut Context) {
    cx.ui_tree.swap_split_in_direction(tree::Direction::Up)
}

fn swap_view_down(cx: &mut Context) {
    cx.ui_tree.swap_split_in_direction(tree::Direction::Down)
}

fn transpose_view(cx: &mut Context) {
    cx.ui_tree.transpose_view()
}

// split helper, clear it later
fn split(cx: &mut Context, action: Action) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let id = buffer.id();
    let selection = buffer.selection(buffer_view.view_id).clone();
    let offset = buffer_view.offset;

    cx.ui_tree.switch(id, action);

    // match the selection in the previous buffer_view
    let (buffer_view, buffer) = current!(cx.ui_tree);
    buffer.set_selection(buffer_view.view_id, selection);
    // match the buffer_view scroll offset (switch doesn't handle this fully
    // since the selection is only matched after the split)
    buffer_view.offset = offset;
}

fn hsplit(cx: &mut Context) {
    split(cx, Action::HorizontalSplit);
}

fn hsplit_new(cx: &mut Context) {
    cx.ui_tree.new_file(Action::HorizontalSplit);
}

fn vsplit(cx: &mut Context) {
    split(cx, Action::VerticalSplit);
}

fn vsplit_new(cx: &mut Context) {
    cx.ui_tree.new_file(Action::VerticalSplit);
}

fn wclose(cx: &mut Context) {
    if cx.ui_tree.tree.views().count() == 1 {
        if let Err(err) = typed::buffers_remaining_impl(cx.ui_tree) {
            cx.ui_tree.set_error(err.to_string());
            return;
        }
    }
    let buffer_view_id = buffer_view!(cx.ui_tree).id;
    // close current split
    cx.ui_tree.close(buffer_view_id);
}

fn wonly(cx: &mut Context) {
    let buffer_views = cx
        .ui_tree
        .tree
        .views()
        .map(|(v, focus)| (v.id, focus))
        .collect::<Vec<_>>();
    for (buffer_view_id, focus) in buffer_views {
        if !focus {
            cx.ui_tree.close(buffer_view_id);
        }
    }
}

fn select_register(cx: &mut Context) {
    cx.ui_tree.autoinfo = Some(Info::from_registers(&cx.ui_tree.registers));
    cx.on_next_key(move |cx, event| {
        if let Some(ch) = event.char() {
            cx.ui_tree.autoinfo = None;
            cx.ui_tree.selected_register = Some(ch);
        }
    })
}

fn insert_register(cx: &mut Context) {
    cx.ui_tree.autoinfo = Some(Info::from_registers(&cx.ui_tree.registers));
    cx.on_next_key(move |cx, event| {
        if let Some(ch) = event.char() {
            cx.ui_tree.autoinfo = None;
            cx.ui_tree.register = Some(ch);
            paste(cx, Paste::Cursor);
        }
    })
}

fn align_view_top(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    align_view(buffer, buffer_view, Align::Top);
}

fn align_view_center(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    align_view(buffer, buffer_view, Align::Center);
}

fn align_view_bottom(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    align_view(buffer, buffer_view, Align::Bottom);
}

fn align_view_middle(cx: &mut Context) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);
    let pos = buffer.selection(buffer_view.view_id).primary().cursor(text);
    let pos = coords_at_pos(text, pos);

    buffer_view.offset.col = pos
        .col
        .saturating_sub((buffer_view.inner_area(buffer).width as usize) / 2);
}

fn scroll_up(cx: &mut Context) {
    scroll(cx, ui_tree.command_multiplier.unwrap_or_one().get(), Direction::Backward);
}

fn scroll_down(cx: &mut Context) {
    scroll(cx, ui_tree.command_multiplier.unwrap_or_one().get(), Direction::Forward);
}

fn goto_ts_object_impl(cx: &mut Context, object: &'static str, direction: Direction) {
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();
    let motion = move |ui_tree: &mut UITree| {
        let (buffer_view, buffer) = current!(ui_tree);
        if let Some((lang_config, syntax)) = buffer.language_config().zip(buffer.syntax()) {
            let text = buffer.text().slice(..);
            let root = syntax.tree().root_node();

            let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
                let new_range = movement::goto_treesitter_object(
                    text,
                    range,
                    object,
                    direction,
                    root,
                    lang_config,
                    command_multiplier,
                );

                if ui_tree.mode == Mode::Select {
                    let head = if new_range.head < range.anchor {
                        new_range.anchor
                    } else {
                        new_range.head
                    };

                    Range::new(range.anchor, head)
                } else {
                    new_range.with_direction(direction)
                }
            });

            buffer.set_selection(buffer_view.view_id, selection);
        } else {
            ui_tree.set_status("Syntax-tree is not available in current buffer");
        }
    };
    motion(cx.ui_tree);
    cx.ui_tree.last_motion = Some(Motion(Box::new(motion)));
}

fn goto_next_function(cx: &mut Context) {
    goto_ts_object_impl(cx, "function", Direction::Forward)
}

fn goto_prev_function(cx: &mut Context) {
    goto_ts_object_impl(cx, "function", Direction::Backward)
}

fn goto_next_class(cx: &mut Context) {
    goto_ts_object_impl(cx, "class", Direction::Forward)
}

fn goto_prev_class(cx: &mut Context) {
    goto_ts_object_impl(cx, "class", Direction::Backward)
}

fn goto_next_parameter(cx: &mut Context) {
    goto_ts_object_impl(cx, "parameter", Direction::Forward)
}

fn goto_prev_parameter(cx: &mut Context) {
    goto_ts_object_impl(cx, "parameter", Direction::Backward)
}

fn goto_next_comment(cx: &mut Context) {
    goto_ts_object_impl(cx, "comment", Direction::Forward)
}

fn goto_prev_comment(cx: &mut Context) {
    goto_ts_object_impl(cx, "comment", Direction::Backward)
}

fn goto_next_test(cx: &mut Context) {
    goto_ts_object_impl(cx, "test", Direction::Forward)
}

fn goto_prev_test(cx: &mut Context) {
    goto_ts_object_impl(cx, "test", Direction::Backward)
}

fn select_textobject_around(cx: &mut Context) {
    select_textobject(cx, textobject::TextObject::Around);
}

fn select_textobject_inner(cx: &mut Context) {
    select_textobject(cx, textobject::TextObject::Inside);
}

fn select_textobject(cx: &mut Context, objtype: textobject::TextObject) {
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();

    cx.on_next_key(move |cx, event| {
        cx.ui_tree.autoinfo = None;
        if let Some(ch) = event.char() {
            let textobject = move |ui_tree: &mut UITree| {
                let (buffer_view, buffer) = current!(ui_tree);
                let text = buffer.text().slice(..);

                let textobject_treesitter = |obj_name: &str, range: Range| -> Range {
                    let (lang_config, syntax) = match buffer.language_config().zip(buffer.syntax()) {
                        Some(t) => t,
                        None => return range,
                    };
                    textobject::textobject_treesitter(
                        text,
                        range,
                        objtype,
                        obj_name,
                        syntax.tree().root_node(),
                        lang_config,
                        command_multiplier,
                    )
                };

                if ch == 'g' && buffer.diff_handle().is_none() {
                    ui_tree.set_status("Diff is not available in current buffer");
                    return;
                }

                let textobject_change = |range: Range| -> Range {
                    let diff_handle = buffer.diff_handle().unwrap();
                    let hunks = diff_handle.hunks();
                    let line = range.cursor_line(text);
                    let hunk_idx = if let Some(hunk_idx) = hunks.hunk_at(line as u32, false) {
                        hunk_idx
                    } else {
                        return range;
                    };
                    let hunk = hunks.nth_hunk(hunk_idx).after;

                    let start = text.line_to_char(hunk.start as usize);
                    let end = text.line_to_char(hunk.end as usize);
                    Range::new(start, end).with_direction(range.direction())
                };

                let selection = buffer.selection(buffer_view.view_id).clone().transform(|range| {
                    match ch {
                        'w' => textobject::textobject_word(text, range, objtype, command_multiplier, false),
                        'W' => textobject::textobject_word(text, range, objtype, command_multiplier, true),
                        't' => textobject_treesitter("class", range),
                        'f' => textobject_treesitter("function", range),
                        'a' => textobject_treesitter("parameter", range),
                        'c' => textobject_treesitter("comment", range),
                        'T' => textobject_treesitter("test", range),
                        'p' => textobject::textobject_paragraph(text, range, objtype, command_multiplier),
                        'm' => textobject::textobject_pair_surround_closest(
                            text, range, objtype, command_multiplier,
                        ),
                        'g' => textobject_change(range),
                        // TODO: cancel new ranges if inconsistent surround matches across lines
                        ch if !ch.is_ascii_alphanumeric() => {
                            textobject::textobject_pair_surround(text, range, objtype, ch, command_multiplier)
                        }
                        _ => range,
                    }
                });
                buffer.set_selection(buffer_view.view_id, selection);
            };
            textobject(cx.ui_tree);
            cx.ui_tree.last_motion = Some(Motion(Box::new(textobject)));
        }
    });

    let title = match objtype {
        textobject::TextObject::Inside => "Match inside",
        textobject::TextObject::Around => "Match around",
        _ => return,
    };
    let help_text = [
        ("w", "Word"),
        ("W", "WORD"),
        ("p", "Paragraph"),
        ("t", "Type definition (tree-sitter)"),
        ("f", "Function (tree-sitter)"),
        ("a", "Argument/parameter (tree-sitter)"),
        ("c", "Comment (tree-sitter)"),
        ("T", "Test (tree-sitter)"),
        ("m", "Closest surrounding pair to cursor"),
        (" ", "... or any character acting as a pair"),
    ];

    cx.ui_tree.autoinfo = Some(Info::new(title, &help_text));
}

fn surround_add(cx: &mut Context) {
    cx.on_next_key(move |cx, event| {
        let ch = match event.char() {
            Some(ch) => ch,
            None => return,
        };
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let selection = buffer.selection(buffer_view.view_id);
        let (open, close) = surround::get_pair(ch);
        // The number of chars in get_pair
        let surround_len = 2;

        let mut changes = Vec::with_capacity(selection.len() * 2);
        let mut ranges = SmallVec::with_capacity(selection.len());
        let mut offs = 0;

        for range in selection.iter() {
            let mut o = Tendril::new();
            o.push(open);
            let mut c = Tendril::new();
            c.push(close);
            changes.push((range.from(), range.from(), Some(o)));
            changes.push((range.to(), range.to(), Some(c)));

            // Add 2 characters to the range to select them
            ranges.push(
                Range::new(offs + range.from(), offs + range.to() + surround_len)
                    .with_direction(range.direction()),
            );

            // Add 2 characters to the offset for the next ranges
            offs += surround_len;
        }

        let transaction = Transaction::change(buffer.text(), changes.into_iter())
            .with_selection(Selection::new(ranges, selection.primary_index()));
        apply_transaction(&transaction, buffer, buffer_view);
        exit_select_mode(cx);
    })
}

fn surround_replace(cx: &mut Context) {
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();
    cx.on_next_key(move |cx, event| {
        let surround_ch = match event.char() {
            Some('m') => None, // m selects the closest surround pair
            Some(ch) => Some(ch),
            None => return,
        };
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let text = buffer.text().slice(..);
        let selection = buffer.selection(buffer_view.view_id);

        let change_pos = match surround::get_surround_pos(text, selection, surround_ch, command_multiplier) {
            Ok(c) => c,
            Err(err) => {
                cx.ui_tree.set_error(err.to_string());
                return;
            }
        };

        cx.on_next_key(move |cx, event| {
            let (buffer_view, buffer) = current!(cx.ui_tree);
            let to = match event.char() {
                Some(to) => to,
                None => return,
            };
            let (open, close) = surround::get_pair(to);
            let transaction = Transaction::change(
                buffer.text(),
                change_pos.iter().enumerate().map(|(i, &pos)| {
                    let mut t = Tendril::new();
                    t.push(if i % 2 == 0 { open } else { close });
                    (pos, pos + 1, Some(t))
                }),
            );
            apply_transaction(&transaction, buffer, buffer_view);
            exit_select_mode(cx);
        });
    })
}

fn surround_delete(cx: &mut Context) {
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();
    cx.on_next_key(move |cx, event| {
        let surround_ch = match event.char() {
            Some('m') => None, // m selects the closest surround pair
            Some(ch) => Some(ch),
            None => return,
        };
        let (buffer_view, buffer) = current!(cx.ui_tree);
        let text = buffer.text().slice(..);
        let selection = buffer.selection(buffer_view.view_id);

        let change_pos = match surround::get_surround_pos(text, selection, surround_ch, command_multiplier) {
            Ok(c) => c,
            Err(err) => {
                cx.ui_tree.set_error(err.to_string());
                return;
            }
        };

        let transaction =
            Transaction::change(buffer.text(), change_pos.into_iter().map(|p| (p, p + 1, None)));
        apply_transaction(&transaction, buffer, buffer_view);
        exit_select_mode(cx);
    })
}

#[derive(Eq, PartialEq)]
enum ShellBehavior {
    Replace,
    Ignore,
    Insert,
    Append,
}

fn shell_pipe(cx: &mut Context) {
    shell_prompt(cx, "pipe:".into(), ShellBehavior::Replace);
}

fn shell_pipe_to(cx: &mut Context) {
    shell_prompt(cx, "pipe-to:".into(), ShellBehavior::Ignore);
}

fn shell_insert_output(cx: &mut Context) {
    shell_prompt(cx, "insert-output:".into(), ShellBehavior::Insert);
}

fn shell_append_output(cx: &mut Context) {
    shell_prompt(cx, "append-output:".into(), ShellBehavior::Append);
}

fn shell_keep_pipe(cx: &mut Context) {
    ui::prompt(
        cx,
        "keep-pipe:".into(),
        Some('|'),
        ui::completers::none,
        move |cx, input: &str, event: PromptEvent| {
            let shell = &cx.ui_tree.config().shell;
            if event != PromptEvent::Validate {
                return;
            }
            if input.is_empty() {
                return;
            }
            let (buffer_view, buffer) = current!(cx.ui_tree);
            let selection = buffer.selection(buffer_view.view_id);

            let mut ranges = SmallVec::with_capacity(selection.len());
            let old_index = selection.primary_index();
            let mut index: Option<usize> = None;
            let text = buffer.text().slice(..);

            for (i, range) in selection.ranges().iter().enumerate() {
                let fragment = range.slice(text);
                let (_output, success) = match shell_impl(shell, input, Some(fragment.into())) {
                    Ok(result) => result,
                    Err(err) => {
                        cx.ui_tree.set_error(err.to_string());
                        return;
                    }
                };

                // if the process exits successfully, keep the selection
                if success {
                    ranges.push(*range);
                    if i >= old_index && index.is_none() {
                        index = Some(ranges.len() - 1);
                    }
                }
            }

            if ranges.is_empty() {
                cx.ui_tree.set_error("No selections remaining");
                return;
            }

            let index = index.unwrap_or_else(|| ranges.len() - 1);
            buffer.set_selection(buffer_view.view_id, Selection::new(ranges, index));
        },
    );
}

fn shell_impl(shell: &[String], cmd: &str, input: Option<Rope>) -> anyhow::Result<(Tendril, bool)> {
    tokio::task::block_in_place(|| helix_lsp::block_on(shell_impl_async(shell, cmd, input)))
}

async fn shell_impl_async(
    shell: &[String],
    cmd: &str,
    input: Option<Rope>,
) -> anyhow::Result<(Tendril, bool)> {
    use std::process::Stdio;
    use tokio::process::Command;
    ensure!(!shell.is_empty(), "No shell set");

    let mut process = Command::new(&shell[0]);
    process
        .args(&shell[1..])
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if input.is_some() || cfg!(windows) {
        process.stdin(Stdio::piped());
    } else {
        process.stdin(Stdio::null());
    }

    let mut process = match process.spawn() {
        Ok(process) => process,
        Err(e) => {
            log::error!("Failed to start shell: {}", e);
            return Err(e.into());
        }
    };
    let output = if let Some(mut stdin) = process.stdin.take() {
        let input_task = tokio::spawn(async move {
            if let Some(input) = input {
                helix_view::buffer::to_writer(&mut stdin, encoding::UTF_8, &input).await?;
            }
            Ok::<_, anyhow::Error>(())
        });
        let (output, _) = tokio::join! {
            process.wait_with_output(),
            input_task,
        };
        output?
    } else {
        // Process has no stdin, so we just take the output
        process.wait_with_output().await?
    };

    if !output.status.success() {
        if !output.stderr.is_empty() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            log::error!("Shell error: {}", err);
            bail!("Shell error: {}", err);
        }
        bail!("Shell command failed");
    } else if !output.stderr.is_empty() {
        log::debug!(
            "Command printed to stderr: {}",
            String::from_utf8_lossy(&output.stderr).to_string()
        );
    }

    let str = std::str::from_utf8(&output.stdout)
        .map_err(|_| anyhow!("Process did not output valid UTF-8"))?;
    let tendril = Tendril::from(str);
    Ok((tendril, output.status.success()))
}

fn shell(cx: &mut compositor::Context, cmd: &str, behavior: &ShellBehavior) {
    let pipe = match behavior {
        ShellBehavior::Replace | ShellBehavior::Ignore => true,
        ShellBehavior::Insert | ShellBehavior::Append => false,
    };

    let config = cx.ui_tree.config();
    let shell = &config.shell;
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let selection = buffer.selection(buffer_view.view_id);

    let mut changes = Vec::with_capacity(selection.len());
    let mut ranges = SmallVec::with_capacity(selection.len());
    let text = buffer.text().slice(..);

    let mut shell_output: Option<Tendril> = None;
    let mut offset = 0isize;
    for range in selection.ranges() {
        let (output, success) = if let Some(output) = shell_output.as_ref() {
            (output.clone(), true)
        } else {
            let fragment = range.slice(text);
            match shell_impl(shell, cmd, pipe.then(|| fragment.into())) {
                Ok(result) => {
                    if !pipe {
                        shell_output = Some(result.0.clone());
                    }
                    result
                }
                Err(err) => {
                    cx.ui_tree.set_error(err.to_string());
                    return;
                }
            }
        };

        if !success {
            cx.ui_tree.set_error("Command failed");
            return;
        }

        let output_len = output.chars().count();

        let (from, to, deleted_len) = match behavior {
            ShellBehavior::Replace => (range.from(), range.to(), range.len()),
            ShellBehavior::Insert => (range.from(), range.from(), 0),
            ShellBehavior::Append => (range.to(), range.to(), 0),
            _ => (range.from(), range.from(), 0),
        };

        // These `usize`s cannot underflow because selection ranges cannot overlap.
        // Once the MSRV is 1.66.0 (mixed_integer_ops is stabilized), we can use checked
        // arithmetic to assert this.
        let anchor = (to as isize + offset - deleted_len as isize) as usize;
        let new_range = Range::new(anchor, anchor + output_len).with_direction(range.direction());
        ranges.push(new_range);
        offset = offset + output_len as isize - deleted_len as isize;

        changes.push((from, to, Some(output)));
    }

    if behavior != &ShellBehavior::Ignore {
        let transaction = Transaction::change(buffer.text(), changes.into_iter())
            .with_selection(Selection::new(ranges, selection.primary_index()));
        apply_transaction(&transaction, buffer, buffer_view);
        buffer.append_changes_to_history(buffer_view);
    }

    // after replace cursor may be out of bounds, do this to
    // make sure cursor is in buffer_view and update scroll as well
    buffer_view.ensure_cursor_in_view(buffer, config.scrolloff);
}

fn shell_prompt(cx: &mut Context, prompt: Cow<'static, str>, behavior: ShellBehavior) {
    ui::prompt(
        cx,
        prompt,
        Some('|'),
        ui::completers::none,
        move |cx, input: &str, event: PromptEvent| {
            if event != PromptEvent::Validate {
                return;
            }
            if input.is_empty() {
                return;
            }

            shell(cx, input, &behavior);
        },
    );
}

fn suspend(_cx: &mut Context) {
    #[cfg(not(windows))]
    signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP).unwrap();
}

fn add_newline_above(cx: &mut Context) {
    add_newline_impl(cx, Open::Above);
}

fn add_newline_below(cx: &mut Context) {
    add_newline_impl(cx, Open::Below)
}

fn add_newline_impl(cx: &mut Context, open: Open) {
    let command_multiplier = ui_tree.command_multiplier.unwrap_or_one().get();
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let selection = buffer.selection(buffer_view.view_id);
    let text = buffer.text();
    let slice = text.slice(..);

    let changes = selection.into_iter().map(|range| {
        let (start, end) = range.line_range(slice);
        let line = match open {
            Open::Above => start,
            Open::Below => end + 1,
        };
        let pos = text.line_to_char(line);
        (
            pos,
            pos,
            Some(buffer.line_ending.as_str().repeat(command_multiplier).into()),
        )
    });

    let transaction = Transaction::change(text, changes);
    apply_transaction(&transaction, buffer, buffer_view);
}

enum IncrementDirection {
    Increase,
    Decrease,
}
/// Increment object under cursor by count.
fn increment(cx: &mut Context) {
    increment_impl(cx, IncrementDirection::Increase);
}

/// Decrement object under cursor by count.
fn decrement(cx: &mut Context) {
    increment_impl(cx, IncrementDirection::Decrease);
}

/// This function differs from find_next_char_impl in that it stops searching at the newline, but also
/// starts searching at the current character, instead of the next.
/// It does not want to start at the next character because this function is used for incrementing
/// number and we don't want to move forward if we're already on a digit.
fn find_next_char_until_newline<M: CharMatcher>(
    text: RopeSlice,
    char_matcher: M,
    pos: usize,
    _count: usize,
    _inclusive: bool,
) -> Option<usize> {
    // Since we send the current line to find_nth_next instead of the whole text, we need to adjust
    // the position we send to this function so that it's relative to that line and its returned
    // position since it's expected this function returns a global position.
    let line_index = text.char_to_line(pos);
    let pos_delta = text.line_to_char(line_index);
    let pos = pos - pos_delta;
    search::find_nth_next(text.line(line_index), char_matcher, pos, 1).map(|pos| pos + pos_delta)
}

/// Decrement object under cursor by `amount`.
fn increment_impl(cx: &mut Context, increment_direction: IncrementDirection) {
    // TODO: when incrementing or decrementing a number that gets a new digit or lose one, the
    // selection is updated improperly.
    find_char_impl(
        cx.ui_tree,
        &find_next_char_until_newline,
        true,
        true,
        char::is_ascii_digit,
        1,
    );

    // Increase by 1 if `IncrementDirection` is `Increase`
    // Decrease by 1 if `IncrementDirection` is `Decrease`
    let sign = match increment_direction {
        IncrementDirection::Increase => 1,
        IncrementDirection::Decrease => -1,
    };
    let mut amount = sign * ui_tree.command_multiplier.unwrap_or_one().get() as i64;

    // If the register is `#` then increase or decrease the `amount` by 1 per element
    let increase_by = if cx.ui_tree.register == Some('#') { sign } else { 0 };

    let (buffer_view, buffer) = current!(cx.ui_tree);
    let selection = buffer.selection(buffer_view.view_id);
    let text = buffer.text().slice(..);

    let changes: Vec<_> = selection
        .ranges()
        .iter()
        .filter_map(|range| {
            let incrementor: Box<dyn Increment> =
                if let Some(incrementor) = DateTimeIncrementor::from_range(text, *range) {
                    Box::new(incrementor)
                } else if let Some(incrementor) = NumberIncrementor::from_range(text, *range) {
                    Box::new(incrementor)
                } else {
                    return None;
                };

            let (range, new_text) = incrementor.increment(amount);

            amount += increase_by;

            Some((range.from(), range.to(), Some(new_text)))
        })
        .collect();

    // Overlapping changes in a transaction will panic, so we need to find and remove them.
    // For example, if there are cursors on each of the year, month, and day of `2021-11-29`,
    // incrementing will give overlapping changes, with each change incrementing a different part of
    // the date. Since these conflict with each other we remove these changes from the transaction
    // so nothing happens.
    let mut overlapping_indexes = HashSet::new();
    for (i, changes) in changes.windows(2).enumerate() {
        if changes[0].1 > changes[1].0 {
            overlapping_indexes.insert(i);
            overlapping_indexes.insert(i + 1);
        }
    }
    let changes: Vec<_> = changes
        .into_iter()
        .enumerate()
        .filter_map(|(i, change)| {
            if overlapping_indexes.contains(&i) {
                None
            } else {
                Some(change)
            }
        })
        .collect();

    if !changes.is_empty() {
        let transaction = Transaction::change(buffer.text(), changes.into_iter());
        let transaction = transaction.with_selection(selection.clone());

        apply_transaction(&transaction, buffer, buffer_view);
    }
}

fn record_macro(cx: &mut Context) {
    if let Some((reg, mut keys)) = cx.ui_tree.macro_recording.take() {
        // Remove the keypress which ends the recording
        keys.pop();
        let s = keys
            .into_iter()
            .map(|key| {
                let s = key.to_string();
                if s.chars().count() == 1 {
                    s
                } else {
                    format!("<{}>", s)
                }
            })
            .collect::<String>();
        cx.ui_tree.registers.write(reg, vec![s]);
        cx.ui_tree
            .set_status(format!("Recorded to register [{}]", reg));
    } else {
        let reg = cx.ui_tree.register.take().unwrap_or('@');
        cx.ui_tree.macro_recording = Some((reg, Vec::new()));
        cx.ui_tree
            .set_status(format!("Recording to register [{}]", reg));
    }
}

fn replay_macro(cx: &mut Context) {
    let reg = cx.ui_tree.selected_register.unwrap_or('@');

    if cx.ui_tree.macro_replaying.contains(&reg) {
        cx.ui_tree.set_error(format!(
            "Cannot replay from register [{}] because already replaying from same register",
            reg
        ));
        return;
    }

    let keys: Vec<KeyEvent> = if let Some([keys_str]) = cx.ui_tree.registers.read(reg) {
        match helix_view::input::parse_macro(keys_str) {
            Ok(keys) => keys,
            Err(err) => {
                cx.ui_tree.set_error(format!("Invalid macro: {}", err));
                return;
            }
        }
    } else {
        cx.ui_tree.set_error(format!("Register [{}] empty", reg));
        return;
    };

    // Once the macro has been fully validated, it's marked as being under replay
    // to ensure we don't fall into infinite recursion.
    cx.ui_tree.macro_replaying.push(reg);

    let command_multiplier = cx.ui_tree.command_multiplier.unwrap_or_one().get();
    cx.callback = Some(Box::new(move |compositor, cx| {
        for _ in 0..command_multiplier {
            for &key in keys.iter() {
                compositor.handle_event(&compositor::Event::Key(key), cx);
            }
        }
        // The macro under replay is cleared at the end of the callback, not in the
        // macro replay context, or it will not correctly protect the user from
        // replaying recursively.
        cx.ui_tree.macro_replaying.pop();
    }));
}
