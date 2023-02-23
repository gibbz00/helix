#[cfg(test)]
mod test;

use std::iter;

use ropey::iter::Chars;

use crate::{
    char_idx_at_visual_offset,
    chars::{categorize_char, char_is_line_ending, CharCategory},
    doc_formatter::TextFormat,
    graphemes::{
        next_grapheme_boundary, nth_next_grapheme_boundary, nth_prev_grapheme_boundary,
        prev_grapheme_boundary,
    },
    line_ending::rope_is_line_ending,
    position::char_idx_at_visual_block_offset,
    selection::SelectionRule,
    text_annotations::TextAnnotations,
    visual_offset_from_block, Range, RopeSlice,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Movement {
    Extend,
    Move,
}

pub fn move_horizontally(
    slice: RopeSlice,
    range: Range,
    dir: Direction,
    count: usize,
    behaviour: Movement,
    _: &TextFormat,
    _: &mut TextAnnotations,
) -> Range {
    let pos = range.cursor(slice);

    // Compute the new position.
    let new_pos = match dir {
        Direction::Forward => nth_next_grapheme_boundary(slice, pos, count),
        Direction::Backward => nth_prev_grapheme_boundary(slice, pos, count),
    };

    // Compute the final new range.
    range.put_cursor(slice, new_pos, behaviour == Movement::Extend)
}

pub fn move_vertically_visual(
    slice: RopeSlice,
    range: Range,
    dir: Direction,
    count: usize,
    behaviour: Movement,
    text_fmt: &TextFormat,
    annotations: &mut TextAnnotations,
) -> Range {
    if !text_fmt.soft_wrap {
        move_vertically(slice, range, dir, count, behaviour, text_fmt, annotations);
    }
    annotations.clear_line_annotations();
    let pos = range.cursor(slice);

    // Compute the current position's 2d coordinates.
    let (visual_pos, block_off) = visual_offset_from_block(slice, pos, pos, text_fmt, annotations);
    let new_col = range
        .old_visual_position
        .map_or(visual_pos.col as u32, |(_, col)| col);

    // Compute the new position.
    let mut row_off = match dir {
        Direction::Forward => count as isize,
        Direction::Backward => -(count as isize),
    };

    // TODO how to handle inline annotations that span an entire visual line (very unlikely).

    // Compute visual offset relative to block start to avoid trasversing the block twice
    row_off += visual_pos.row as isize;
    let new_pos = char_idx_at_visual_offset(
        slice,
        block_off,
        row_off,
        new_col as usize,
        text_fmt,
        annotations,
    )
    .0;

    // Special-case to avoid moving to the end of the last non-empty line.
    if behaviour == Movement::Extend && slice.line(slice.char_to_line(new_pos)).len_chars() == 0 {
        return range;
    }

    let mut new_range = range.put_cursor(slice, new_pos, behaviour == Movement::Extend);
    new_range.old_visual_position = Some((0, new_col));
    new_range
}

pub fn move_vertically(
    slice: RopeSlice,
    range: Range,
    dir: Direction,
    count: usize,
    behaviour: Movement,
    text_fmt: &TextFormat,
    annotations: &mut TextAnnotations,
) -> Range {
    annotations.clear_line_annotations();
    let pos = range.cursor(slice);
    let line_idx = slice.char_to_line(pos);
    let line_start = slice.line_to_char(line_idx);

    // Compute the current position's 2d coordinates.
    let visual_pos = visual_offset_from_block(slice, line_start, pos, text_fmt, annotations).0;
    let (mut new_row, new_col) = range
        .old_visual_position
        .map_or((visual_pos.row as u32, visual_pos.col as u32), |pos| pos);
    new_row = new_row.max(visual_pos.row as u32);
    let line_idx = slice.char_to_line(pos);

    // Compute the new position.
    let mut new_line_idx = match dir {
        Direction::Forward => line_idx.saturating_add(count),
        Direction::Backward => line_idx.saturating_sub(count),
    };

    let line = if new_line_idx >= slice.len_lines() - 1 {
        // there is no line terminator for the last line
        // so the logic below is not necessary here
        new_line_idx = slice.len_lines() - 1;
        slice
    } else {
        // char_idx_at_visual_block_offset returns a one-past-the-end index
        // in case it reaches the end of the slice
        // to avoid moving to the nextline in that case the line terminator is removed from the line
        let new_line_end = prev_grapheme_boundary(slice, slice.line_to_char(new_line_idx + 1));
        slice.slice(..new_line_end)
    };

    let new_line_start = line.line_to_char(new_line_idx);

    let (new_pos, _) = char_idx_at_visual_block_offset(
        line,
        new_line_start,
        new_row as usize,
        new_col as usize,
        text_fmt,
        annotations,
    );

    // Special-case to avoid moving to the end of the last non-empty line.
    if behaviour == Movement::Extend && slice.line(new_line_idx).len_chars() == 0 {
        return range;
    }

    let mut new_range = range.put_cursor(slice, new_pos, behaviour == Movement::Extend);
    new_range.old_visual_position = Some((new_row, new_col));
    new_range
}

pub fn move_next_word_start(slice: RopeSlice, range: Range, count: usize) -> Range {
    word_move(slice, range, count, WordMotionTarget::NextWordStart)
}

pub fn move_next_word_end(slice: RopeSlice, range: Range, count: usize) -> Range {
    word_move(slice, range, count, WordMotionTarget::NextWordEnd)
}

pub fn move_prev_word_start(slice: RopeSlice, range: Range, count: usize) -> Range {
    word_move(slice, range, count, WordMotionTarget::PrevWordStart)
}

pub fn move_next_long_word_start(slice: RopeSlice, range: Range, count: usize) -> Range {
    word_move(slice, range, count, WordMotionTarget::NextLongWordStart)
}

pub fn move_next_long_word_end(slice: RopeSlice, range: Range, count: usize) -> Range {
    word_move(slice, range, count, WordMotionTarget::NextLongWordEnd)
}

pub fn move_prev_long_word_start(slice: RopeSlice, range: Range, count: usize) -> Range {
    word_move(slice, range, count, WordMotionTarget::PrevLongWordStart)
}

pub fn move_prev_word_end(slice: RopeSlice, range: Range, count: usize) -> Range {
    word_move(slice, range, count, WordMotionTarget::PrevWordEnd)
}

fn word_move(slice: RopeSlice, range: Range, count: usize, target: WordMotionTarget) -> Range {
    let is_prev = matches!(
        target,
        WordMotionTarget::PrevWordStart
            | WordMotionTarget::PrevLongWordStart
            | WordMotionTarget::PrevWordEnd
    );

    // Special-case early-out.
    if (is_prev && range.head == 0) || (!is_prev && range.head == slice.len_chars()) {
        return range;
    }

    // Prepare the range appropriately based on the target movement
    // direction.  This is addressing two things at once:
    //
    //   1. Block-cursor semantics.
    //   2. The anchor position being irrelevant to the output result.
    #[allow(clippy::collapsible_else_if)] // Makes the structure clearer in this case.
    let start_range = if is_prev {
        if range.anchor < range.head {
            Range::new(range.head, prev_grapheme_boundary(slice, range.head))
        } else {
            Range::new(next_grapheme_boundary(slice, range.head), range.head)
        }
    } else {
        if range.anchor < range.head {
            Range::new(prev_grapheme_boundary(slice, range.head), range.head)
        } else {
            Range::new(range.head, next_grapheme_boundary(slice, range.head))
        }
    };

    // Do the main work.
    let mut range = start_range;
    for _ in 0..count {
        let next_range = slice.chars_at(range.head).range_to_target(target, range);
        if range == next_range {
            break;
        }
        range = next_range;
    }
    range
}

#[allow(non_upper_case_globals)]
pub const move_prev_paragraph: SelectionRule = |range, rule_cx| {
    let text = rule_cx.text;
    let mut curr_line_nr = range.cursor_line(text);
    let on_first_char = text.line_to_char(curr_line_nr) == range.cursor(text);
    let prev_line_empty = rope_is_line_ending(text.line(curr_line_nr.saturating_sub(1)));
    let curr_line_empty = rope_is_line_ending(text.line(curr_line_nr));
    let prev_empty_to_line = prev_line_empty && !curr_line_empty;

    // skip character before paragraph boundary
    if prev_empty_to_line && !on_first_char {
        curr_line_nr += 1;
    }
    let mut lines = text.lines_at(curr_line_nr);
    lines.reverse();
    let mut lines_peekable = lines.peekable();
    while lines_peekable
        .next_if(|&rope| rope_is_line_ending(rope))
        .is_some()
    {
        curr_line_nr -= 1;
    }
    while lines_peekable
        .next_if(|&rope| !rope_is_line_ending(rope))
        .is_some()
    {
        curr_line_nr -= 1;
    }

    let head = text.line_to_char(curr_line_nr);
    let anchor = if !rule_cx.extend {
        // exclude first character after paragraph boundary
        if prev_empty_to_line && on_first_char {
            range.cursor(text)
        } else {
            range.head
        }
    } else {
        range.put_cursor(text, head, true).anchor
    };
    Range::new(anchor, head)
};

#[allow(non_upper_case_globals)]
pub const move_next_paragraph: SelectionRule = |range: Range, rule_cx| -> Range {
    let text = rule_cx.text;
    let mut curr_line_nr = range.cursor_line(text);

    let on_last_char =
        prev_grapheme_boundary(text, text.line_to_char(curr_line_nr + 1)) == range.cursor(text);

    let curr_line_empty = rope_is_line_ending(text.line(curr_line_nr));

    // diffs
    let next_line_empty =
        rope_is_line_ending(text.line(text.len_lines().saturating_sub(1).min(curr_line_nr + 1)));

    let curr_empty_to_line = curr_line_empty && !next_line_empty;

    // skip character after paragraph boundary
    if curr_empty_to_line && on_last_char {
        curr_line_nr += 1;
    }
    let mut lines_peekable = text.lines_at(curr_line_nr).peekable();

    while lines_peekable
        .next_if(|&rope| !rope_is_line_ending(rope))
        .is_some()
    {
        curr_line_nr += 1;
    }
    while lines_peekable
        .next_if(|&rope| rope_is_line_ending(rope))
        .is_some()
    {
        curr_line_nr += 1;
    }

    let head = text.line_to_char(curr_line_nr);
    let anchor = if !rule_cx.extend {
        if curr_empty_to_line && on_last_char {
            range.head
        } else {
            range.cursor(text)
        }
    } else {
        range.put_cursor(text, head, true).anchor
    };
    Range::new(anchor, head)
};

// ---- util ------------

#[inline]
/// Returns first index that doesn't satisfy a given predicate when
/// advancing the character index.
///
/// Returns none if all characters satisfy the predicate.
pub fn skip_while<F>(slice: RopeSlice, pos: usize, fun: F) -> Option<usize>
where
    F: Fn(char) -> bool,
{
    let mut chars = slice.chars_at(pos).enumerate();
    chars.find_map(|(i, c)| if !fun(c) { Some(pos + i) } else { None })
}

#[inline]
/// Returns first index that doesn't satisfy a given predicate when
/// retreating the character index, saturating if all elements satisfy
/// the condition.
pub fn backwards_skip_while<F>(slice: RopeSlice, pos: usize, fun: F) -> Option<usize>
where
    F: Fn(char) -> bool,
{
    let mut chars_starting_from_next = slice.chars_at(pos);
    let mut backwards = iter::from_fn(|| chars_starting_from_next.prev()).enumerate();
    backwards.find_map(|(i, c)| {
        if !fun(c) {
            Some(pos.saturating_sub(i))
        } else {
            None
        }
    })
}

/// Possible targets of a word motion
#[derive(Copy, Clone, Debug)]
pub enum WordMotionTarget {
    NextWordStart,
    NextWordEnd,
    PrevWordStart,
    PrevWordEnd,
    // A "Long word" (also known as a WORD in Vim/Kakoune) is strictly
    // delimited by whitespace, and can consist of punctuation as well
    // as alphanumerics.
    NextLongWordStart,
    NextLongWordEnd,
    PrevLongWordStart,
}

pub trait CharHelpers {
    fn range_to_target(&mut self, target: WordMotionTarget, origin: Range) -> Range;
}

impl CharHelpers for Chars<'_> {
    /// Note: this only changes the anchor of the range if the head is effectively
    /// starting on a boundary (either directly or after skipping newline characters).
    /// Any other changes to the anchor should be handled by the calling code.
    fn range_to_target(&mut self, target: WordMotionTarget, origin: Range) -> Range {
        let is_prev = matches!(
            target,
            WordMotionTarget::PrevWordStart
                | WordMotionTarget::PrevLongWordStart
                | WordMotionTarget::PrevWordEnd
        );

        // Reverse the iterator if needed for the motion direction.
        if is_prev {
            self.reverse();
        }

        // Function to advance index in the appropriate motion direction.
        let advance: &dyn Fn(&mut usize) = if is_prev {
            &|idx| *idx = idx.saturating_sub(1)
        } else {
            &|idx| *idx += 1
        };

        // Initialize state variables.
        let mut anchor = origin.anchor;
        let mut head = origin.head;
        let mut prev_ch = {
            let ch = self.prev();
            if ch.is_some() {
                self.next();
            }
            ch
        };

        // Skip any initial newline characters.
        while let Some(ch) = self.next() {
            if char_is_line_ending(ch) {
                prev_ch = Some(ch);
                advance(&mut head);
            } else {
                self.prev();
                break;
            }
        }
        if prev_ch.map(char_is_line_ending).unwrap_or(false) {
            anchor = head;
        }

        // Find our target position(s).
        let head_start = head;
        #[allow(clippy::while_let_on_iterator)] // Clippy's suggestion to fix doesn't work here.
        while let Some(next_ch) = self.next() {
            if prev_ch.is_none() || reached_target(target, prev_ch.unwrap(), next_ch) {
                if head == head_start {
                    anchor = head;
                } else {
                    break;
                }
            }
            prev_ch = Some(next_ch);
            advance(&mut head);
        }

        // Un-reverse the iterator if needed.
        if is_prev {
            self.reverse();
        }

        Range::new(anchor, head)
    }
}

fn is_word_boundary(a: char, b: char) -> bool {
    categorize_char(a) != categorize_char(b)
}

fn is_long_word_boundary(a: char, b: char) -> bool {
    match (categorize_char(a), categorize_char(b)) {
        (CharCategory::Word, CharCategory::Punctuation)
        | (CharCategory::Punctuation, CharCategory::Word) => false,
        (a, b) if a != b => true,
        _ => false,
    }
}

fn reached_target(target: WordMotionTarget, prev_ch: char, next_ch: char) -> bool {
    match target {
        WordMotionTarget::NextWordStart | WordMotionTarget::PrevWordEnd => {
            is_word_boundary(prev_ch, next_ch)
                && (char_is_line_ending(next_ch) || !next_ch.is_whitespace())
        }
        WordMotionTarget::NextWordEnd | WordMotionTarget::PrevWordStart => {
            is_word_boundary(prev_ch, next_ch)
                && (!prev_ch.is_whitespace() || char_is_line_ending(next_ch))
        }
        WordMotionTarget::NextLongWordStart => {
            is_long_word_boundary(prev_ch, next_ch)
                && (char_is_line_ending(next_ch) || !next_ch.is_whitespace())
        }
        WordMotionTarget::NextLongWordEnd | WordMotionTarget::PrevLongWordStart => {
            is_long_word_boundary(prev_ch, next_ch)
                && (!prev_ch.is_whitespace() || char_is_line_ending(next_ch))
        }
    }
}
