use crate::{align_view, gutter::GutterComponents, graphics::Rect, Align, Buffer, BufferID, BufferViewID, jump::JumpList};
use helix_core::{
    pos_at_visual_coords, visual_coords_at_pos, Position, RopeSlice, Selection, Transaction,
};

use std::{
    collections::{HashMap, VecDeque},
    fmt,
};


#[derive(Clone)]
pub struct BufferView {
    pub id: BufferViewID,
    pub offset: Position,
    pub area: Rect,
    pub buffer_id: BufferID,
    pub jumps: JumpList,
    // Buffers accessed from this view from the oldest one to last viewed one
    pub buffer_access_history: Vec<BufferID>,
    /// the last modified files before the current one
    /// ordered from most frequent to least frequent
    // uses two buffers because we want to be able to swap between the
    // two last modified buffers which we need to manually keep track of
    pub last_modified_buffers: [Option<BufferID>; 2],
    /// used to store previous selections of tree-sitter objects
    pub object_selections: Vec<Selection>,
    /// GutterTypes used to fetch Gutter (constructor) and width for rendering
    gutters: Vec<GutterComponents>,
    /// A mapping between buffers and the last history revision the buffer view was updated at.
    /// Changes between buffers and buffer views are synced lazily when switching windows. This
    /// mapping keeps track of the last applied history revision so that only new changes
    /// are applied.
    buffer_revisions: HashMap<BufferID, usize>,
}

impl fmt::Debug for BufferView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufferView")
            .field("view_id", &self.id)
            .field("area", &self.area)
            .field("buffer", &self.buffer_id)
            .finish()
    }
}

impl BufferView {
    pub fn new(buffer_id: BufferID, gutter_types: Vec<GutterComponents>) -> Self {
        Self {
            id: BufferViewID::default(),
            buffer_id,
            offset: Position::new(0, 0),
            area: Rect::default(), // will get calculated upon inserting into tree
            jumps: JumpList::new((buffer_id, Selection::point(0))), // TODO: use actual sel
            buffer_access_history: Vec::new(),
            last_modified_buffers: [None, None],
            object_selections: Vec::new(),
            gutters: gutter_types,
            buffer_revisions: HashMap::new(),
        }
    }

    pub fn add_to_history(&mut self, id: BufferID) {
        if let Some(pos) = self.buffer_access_history.iter().position(|&buffer_id| buffer_id == id) {
            self.buffer_access_history.remove(pos);
        }
        self.buffer_access_history.push(id);
    }

    pub fn inner_area(&self, buffer: &Buffer) -> Rect {
        self.area.clip_left(self.gutter_offset(buffer)).clip_bottom(1) // -1 for statusline
    }

    pub fn inner_height(&self) -> usize {
        self.area.clip_bottom(1).height.into() // -1 for statusline
    }

    pub fn gutters(&self) -> &[GutterComponents] {
        &self.gutters
    }

    pub fn gutter_offset(&self, buffer: &Buffer) -> u16 {
        self.gutters
            .iter()
            .map(|gutter| gutter.width(self, buffer) as u16)
            .sum()
    }

    //
    pub fn offset_coords_to_in_view(
        &self,
        buffer: &Buffer,
        scrolloff: usize,
    ) -> Option<(usize, usize)> {
        self.offset_coords_to_in_view_center(buffer, scrolloff, false)
    }

    pub fn offset_coords_to_in_view_center(
        &self,
        buffer: &Buffer,
        scrolloff: usize,
        centering: bool,
    ) -> Option<(usize, usize)> {
        let cursor = buffer
            .selection(self.id)
            .primary()
            .cursor(buffer.text().slice(..));

        let Position { col, row: line } =
            visual_coords_at_pos(buffer.text().slice(..), cursor, buffer.tab_width());

        let inner_area = self.inner_area(buffer);
        let last_line = (self.offset.row + inner_area.height as usize).saturating_sub(1);
        let last_col = self.offset.col + inner_area.width.saturating_sub(1) as usize;

        let new_offset = |scrolloff: usize| {
            // - 1 so we have at least one gap in the middle.
            // a height of 6 with padding of 3 on each side will keep shifting the bufferview back and forth
            // as we type
            let scrolloff = scrolloff.min(inner_area.height.saturating_sub(1) as usize / 2);

            let row = if line > last_line.saturating_sub(scrolloff) {
                // scroll down
                self.offset.row + line - (last_line.saturating_sub(scrolloff))
            } else if line < self.offset.row + scrolloff {
                // scroll up
                line.saturating_sub(scrolloff)
            } else {
                self.offset.row
            };

            let col = if col > last_col.saturating_sub(scrolloff) {
                // scroll right
                self.offset.col + col - (last_col.saturating_sub(scrolloff))
            } else if col < self.offset.col + scrolloff {
                // scroll left
                col.saturating_sub(scrolloff)
            } else {
                self.offset.col
            };
            (row, col)
        };
        let current_offset = (self.offset.row, self.offset.col);
        if centering {
            // return None if cursor is out of bufferview
            let offset = new_offset(0);
            (offset == current_offset).then(|| {
                if scrolloff == 0 {
                    offset
                } else {
                    new_offset(scrolloff)
                }
            })
        } else {
            // return None if cursor is in (bufferview - scrolloff)
            let offset = new_offset(scrolloff);
            (offset != current_offset).then(|| offset) // TODO: use 'then_some' when 1.62 <= MSRV
        }
    }

    pub fn ensure_cursor_in_view(&mut self, buffer: &Buffer, scrolloff: usize) {
        if let Some((row, col)) = self.offset_coords_to_in_view_center(buffer, scrolloff, false) {
            self.offset.row = row;
            self.offset.col = col;
        }
    }

    pub fn ensure_cursor_in_view_center(&mut self, buffer: &Buffer, scrolloff: usize) {
        if let Some((row, col)) = self.offset_coords_to_in_view_center(buffer, scrolloff, true) {
            self.offset.row = row;
            self.offset.col = col;
        } else {
            align_view(buffer, self, Align::Center);
        }
    }

    pub fn is_cursor_in_view(&mut self, buffer: &Buffer, scrolloff: usize) -> bool {
        self.offset_coords_to_in_view(buffer, scrolloff).is_none()
    }

    /// Calculates the last visible line on screen
    #[inline]
    pub fn last_line(&self, buffer: &Buffer) -> usize {
        std::cmp::min(
            // Saturating subs to make it inclusive zero indexing.
            (self.offset.row + self.inner_height()).saturating_sub(1),
            buffer.text().len_lines().saturating_sub(1),
        )
    }

    /// Translates a buffer position to an absolute position in the terminal.
    /// Returns a (line, col) position if the position is visible on screen.
    // TODO: Could return width as well for the character width at cursor.
    pub fn screen_coords_at_pos(
        &self,
        buffer: &Buffer,
        text: RopeSlice,
        pos: usize,
    ) -> Option<Position> {
        let line = text.char_to_line(pos);

        if line < self.offset.row || line > self.last_line(buffer) {
            // Line is not visible on screen
            return None;
        }

        let tab_width = buffer.tab_width();
        // TODO: visual_coords_at_pos also does char_to_line which we ignore, can we reuse the call?
        let Position { col, .. } = visual_coords_at_pos(text, pos, tab_width);

        // It is possible for underflow to occur if the buffer length is larger than the terminal width.
        let row = line.saturating_sub(self.offset.row);
        let col = col.saturating_sub(self.offset.col);

        Some(Position::new(row, col))
    }

    pub fn text_pos_at_screen_coords(
        &self,
        buffer: &Buffer,
        row: u16,
        column: u16,
        tab_width: usize,
    ) -> Option<usize> {
        let text = buffer.text().slice(..);
        let inner = self.inner_area(buffer);
        // 1 for status
        if row < inner.top() || row >= inner.bottom() {
            return None;
        }

        if column < inner.left() || column > inner.right() {
            return None;
        }

        let text_row = (row - inner.y) as usize + self.offset.row;
        if text_row > text.len_lines() - 1 {
            return Some(text.len_chars());
        }

        let text_col = (column - inner.x) as usize + self.offset.col;

        Some(pos_at_visual_coords(
            text,
            Position {
                row: text_row,
                col: text_col,
            },
            tab_width,
        ))
    }

    /// Translates a screen position to position in the text buffer.
    /// Returns a usize typed position in bounds of the text if found in this bufferview, None if out of bufferview.
    pub fn pos_at_screen_coords(&self, buffer: &Buffer, row: u16, column: u16) -> Option<usize> {
        self.text_pos_at_screen_coords(buffer, row, column, buffer.tab_width())
    }

    /// Translates screen coordinates into coordinates on the gutter of the bufferview.
    /// Returns a tuple of usize typed line and column numbers starting with 0.
    /// Returns None if coordinates are not on the gutter.
    pub fn gutter_coords_at_screen_coords(&self, row: u16, column: u16) -> Option<Position> {
        // 1 for status
        if row < self.area.top() || row >= self.area.bottom() {
            return None;
        }

        if column < self.area.left() || column > self.area.right() {
            return None;
        }

        Some(Position::new(
            (row - self.area.top()) as usize,
            (column - self.area.left()) as usize,
        ))
    }

    pub fn remove_buffer(&mut self, buffer_id: &BufferID) {
        self.jumps.remove(buffer_id);
        self.buffer_access_history.retain(|buff_id| buff_id != buffer_id);
    }

    /// Applies a [`Transaction`] to the bufferview.
    /// Instead of calling this function directly, use [crate::apply_transaction]
    /// which applies a transaction to the [`Buffer`] and bufferview together.
    pub fn apply(&mut self, transaction: &Transaction, buffer: &mut Buffer) {
        self.jumps.apply(transaction, buffer);
        self.buffer_revisions
            .insert(buffer.id(), buffer.get_current_revision());
    }

    pub fn sync_changes(&mut self, buffer: &mut Buffer) {
        let latest_revision = buffer.get_current_revision();
        let current_revision = *self
            .buffer_revisions
            .entry(buffer.id())
            .or_insert(latest_revision);

        if current_revision == latest_revision {
            return;
        }

        log::debug!(
            "Syncing buffer_view {:?} between {} and {}",
            self.id,
            current_revision,
            latest_revision
        );

        if let Some(transaction) = buffer.history.get_mut().changes_since(current_revision) {
            self.apply(&transaction, buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use helix_core::Rope;
    const OFFSET: u16 = 3; // 1 diagnostic + 2 linenr (< 100 lines)
    const OFFSET_WITHOUT_LINE_NUMBERS: u16 = 1; // 1 diagnostic
                                                // const OFFSET: u16 = GUTTERS.iter().map(|(_, width)| *width as u16).sum();
    use crate::buffer::Buffer;
    use crate::gutter::GutterComponents;

    #[test]
    fn test_text_pos_at_screen_coords() {
        let mut buffer_view = BufferView::new(
            BufferID::default(),
            vec![GutterComponents::Diagnostics, GutterComponents::LineNumbers],
        );
        buffer_view.area = Rect::new(40, 40, 40, 40);
        let rope = Rope::from_str("abc\n\tdef");
        let buffer = Buffer::from(rope, None);

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 40, 2, 4), None);

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 40, 41, 4), None);

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 0, 2, 4), None);

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 0, 49, 4), None);

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 0, 41, 4), None);

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 40, 81, 4), None);

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 78, 41, 4), None);

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 3, 4),
            Some(3)
        );

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 40, 80, 4), Some(3));

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 41, 40 + OFFSET + 1, 4),
            Some(4)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 41, 40 + OFFSET + 4, 4),
            Some(5)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 41, 40 + OFFSET + 7, 4),
            Some(8)
        );

        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 41, 80, 4), Some(8));
    }

    #[test]
    fn test_text_pos_at_screen_coords_without_line_numbers_gutter() {
        let mut buffer_view = BufferView::new(BufferID::default(), vec![GutterComponents::Diagnostics]);
        buffer_view.area = Rect::new(40, 40, 40, 40);
        let rope = Rope::from_str("abc\n\tdef");
        let buffer = Buffer::from(rope, None);
        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 41, 40 + OFFSET_WITHOUT_LINE_NUMBERS + 1, 4),
            Some(4)
        );
    }

    #[test]
    fn test_text_pos_at_screen_coords_without_any_gutters() {
        let mut buffer_view = BufferView::new(BufferID::default(), vec![]);
        buffer_view.area = Rect::new(40, 40, 40, 40);
        let rope = Rope::from_str("abc\n\tdef");
        let buffer = Buffer::from(rope, None);
        assert_eq!(buffer_view.text_pos_at_screen_coords(&buffer, 41, 40 + 1, 4), Some(4));
    }

    #[test]
    fn test_text_pos_at_screen_coords_cjk() {
        let mut buffer_view = BufferView::new(
            BufferID::default(),
            vec![GutterComponents::Diagnostics, GutterComponents::LineNumbers],
        );
        buffer_view.area = Rect::new(40, 40, 40, 40);
        let rope = Rope::from_str("Hi! こんにちは皆さん");
        let buffer = Buffer::from(rope, None);

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET, 4),
            Some(0)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 4, 4),
            Some(4)
        );
        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 5, 4),
            Some(4)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 6, 4),
            Some(5)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 7, 4),
            Some(5)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 8, 4),
            Some(6)
        );
    }

    #[test]
    fn test_text_pos_at_screen_coords_graphemes() {
        let mut buffer_view = BufferView::new(
            BufferID::default(),
            vec![GutterComponents::Diagnostics, GutterComponents::LineNumbers],
        );
        buffer_view.area = Rect::new(40, 40, 40, 40);
        let rope = Rope::from_str("Hèl̀l̀ò world!");
        let buffer = Buffer::from(rope, None);

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET, 4),
            Some(0)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 1, 4),
            Some(1)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 2, 4),
            Some(3)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 3, 4),
            Some(5)
        );

        assert_eq!(
            buffer_view.text_pos_at_screen_coords(&buffer, 40, 40 + OFFSET + 4, 4),
            Some(7)
        );
    }
}
