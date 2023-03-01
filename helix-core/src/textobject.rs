#[cfg(test)]
mod test;

use ropey::RopeSlice;
use std::fmt::Display;

use crate::chars::{categorize_char, char_is_whitespace, CharCategory};
use crate::graphemes::next_grapheme_boundary;
use crate::line_ending::rope_is_line_ending;
use crate::movement::Direction;
use crate::selection::SelectionRule;
use crate::{surround, Range};
use CharCategory::{Eol, Whitespace};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TextObject {
    Around,
    Inside,
    /// Used for moving between objects.
    Movement,
}

impl Display for TextObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Around => "around",
            Self::Inside => "inside",
            Self::Movement => "movement",
        })
    }
}

#[allow(non_upper_case_globals)]
pub const word: SelectionRule =
    |range, rule_cx| word_impl(range, rule_cx.text, rule_cx.ts_textobject, false);

#[allow(non_upper_case_globals)]
pub const word_long: SelectionRule =
    |range, rule_cx| word_impl(range, rule_cx.text, rule_cx.ts_textobject, true);

pub fn word_impl(
    range: Range,
    text: RopeSlice,
    textobject: Option<TextObject>,
    long: bool,
) -> Range {
    let mut pos = range.cursor(text);

    // Edge case: Cursor can still be placed on a non-character (e.g EOF). If so, do not continue.
    if pos >= text.len_chars() {
        return range;
    }

    // move pos backward it char is whitespace to find first word
    // needed for a symmetry in which maw selects:
    // Both "t[e]st  sentence." and "test[ ] sentence" to
    // "[test  ]sentence"
    // +1 because range.cursor gives left side position
    for char in text.chars_at(pos + 1).reversed() {
        log::warn!("{char}, {pos}");
        if char_is_whitespace(char) {
            pos -= 1;
        } else {
            break;
        }
    }

    let (start_idx, mut end_idx) = find_word_boundary(text, pos, long);

    // TEMP: unwrap remove once specialized rule contexts are added
    if TextObject::Around == textobject.unwrap() {
        // extend next word end until first non-whitespace
        for char in text.chars_at(end_idx) {
            if char_is_whitespace(char) {
                end_idx += 1;
            } else {
                break;
            }
        }
    }

    let direction = if range.head >= range.anchor {
        Direction::Forward
    } else {
        Direction::Backward
    };
    // If point -> direction forward
    // If range.head > range.anchor -> direction forward
    // else -> direction backwards
    Range::new(start_idx, end_idx).with_direction(direction)
}

fn find_word_boundary(text: RopeSlice, curr_pos: usize, long: bool) -> (usize, usize) {
    let (mut curr_start_idx, mut curr_end_idx) = (curr_pos, curr_pos);

    // Forward
    let mut prev_category = categorize_char(text.char(curr_end_idx));
    for ch in text.chars_at(curr_end_idx) {
        match categorize_char(ch) {
            Eol | Whitespace => break,
            curr_category => {
                if curr_category != prev_category && !long || curr_end_idx == text.len_chars() {
                    break;
                } else {
                    curr_end_idx += 1;
                    prev_category = curr_category;
                }
            }
        }
    }

    // Backward
    let mut prev_category = if curr_pos == text.len_chars() {
        Whitespace
    } else {
        categorize_char(text.char(curr_pos))
    };
    for ch in text.chars_at(curr_start_idx).reversed() {
        match categorize_char(ch) {
            Eol | Whitespace => break,
            category => {
                if !long
                    && category != prev_category
                    && curr_start_idx != 0
                    && curr_start_idx != text.len_chars()
                {
                    break;
                } else {
                    curr_start_idx = curr_start_idx.saturating_sub(1);
                    prev_category = category;
                }
            }
        }
    }

    (curr_start_idx, curr_end_idx)
}

/* Behavior:
    * Finding the first paragraph:
        If on new line: move upwards until first non line ending is found, or when top of document is reached

    * Select to top of paragraph:
        Increment top_line until a new line is found.
            (Never select newlines above paragraph.)
    * Selecting to bottom of paragraph:
        Increment bottom_line until new line is found
        If object type is around: Also select all trailing newlines.

    * When count is involved: (passed as pos_arg):
        if count > 1
            for _ in (0..count)
                Continue to next non-newline
                Repeat selecting to bottom of paragraph.

    * TODO: Add tests for this.
    * Wierd behaviour #1:
        If cursor on empty line, put around two paragraphs, then `mip/map` picks the next paragraph.
        But if there are two empty newlines in between paragraphs, then the `mip/map` picks the previous paragraph.#[allow(non_upper_case_globals)]
    * My take:
        Should always select upwards as trailing newlines count when doing `map`.
        So they belong in a way to the previous paragraph.

    NOTE:
        `2mip` also selects the newlines inbetween, which a singular `mip` doesn't
        It doesn't in other words split the selection into multiple smaller ranges.
*/

#[allow(non_upper_case_globals)]
pub const paragraph: SelectionRule = |range, rule_cx| {
    let text = rule_cx.text;
    let mut curr_line_nr = range.cursor_line(text);

    // Finding the paragraph/starting line:
    // Move upwards until first non-newline ending is found, or when top of document is reached.
    if rope_is_line_ending(text.line(curr_line_nr)) {
        for prev_line in text.lines_at(curr_line_nr).reversed() {
            if !rope_is_line_ending(prev_line) {
                break;
            }
            curr_line_nr -= 1;
        }
    }

    // Select to top of paragraph:
    // Increment top_line until a new line is found. (Never select newlines above paragraph.)
    let mut top_line_nr = curr_line_nr;
    let mut lines_peekable = text.lines_at(curr_line_nr).reversed().peekable();
    while lines_peekable
        .next_if(|line| !rope_is_line_ending(*line))
        .is_some()
    {
        top_line_nr -= 1;
    }

    // Selecting to bottom of paragraph:
    let mut bottom_line_nr = curr_line_nr;
    let mut lines_peekable = text.lines_at(bottom_line_nr).peekable();
    while lines_peekable
        .next_if(|line| !rope_is_line_ending(*line))
        .is_some()
    {
        bottom_line_nr += 1;
    }

    // If object type is around: Also select all trailing newlines.
    // TEMP: unwrap will be removed once RuleContext becomes a trait.
    if rule_cx.ts_textobject.unwrap() == TextObject::Around {
        let mut lines_peekable = text.lines_at(bottom_line_nr).peekable();
        while lines_peekable
            .next_if(|line| rope_is_line_ending(*line))
            .is_some()
        {
            bottom_line_nr += 1;
        }
    }

    let anchor = text.line_to_char(top_line_nr);
    let head = text.line_to_char(bottom_line_nr);
    Range::new(anchor, head)
};

#[allow(non_upper_case_globals)]
pub const vcs_change: SelectionRule = |range, rule_cx| {
    // TODO: (gibbz) add to run_condition, and remove unwrap once different RuleContexts are added
    // editor.set_status("Diff is not available in current buffer");
    let text = rule_cx.text;
    let diff_handle = rule_cx.diff_handle.unwrap();
    if let Some(new_hunk) = diff_handle
        .hunk_at(range.cursor_line(text) as u32)
        .map(|hunk| hunk.after)
    {
        let start = text.line_to_char(new_hunk.start as usize);
        let end = text.line_to_char(new_hunk.end as usize);
        Range::new(start, end).with_direction(range.direction())
    } else {
        range
    }
};

// TEMP: unwrap until refined rulecontext
// HACK: until achieves same behaviour with repeat, count is passed in as pos_arg
// Solved by checking whether the current selection is around a pair.
#[allow(non_upper_case_globals)]
pub const pair_surround: SelectionRule = |range, rule_cx| {
    pair_surround_impl(
        rule_cx.text,
        range,
        rule_cx.ts_textobject.unwrap(),
        rule_cx.find_char,
        rule_cx.pos_arg,
    )
};

#[allow(non_upper_case_globals)]
pub const pair_surround_closest: SelectionRule = |range, rule_cx| {
    pair_surround_impl(
        rule_cx.text,
        range,
        rule_cx.ts_textobject.unwrap(),
        None,
        rule_cx.pos_arg,
    )
};

fn pair_surround_impl(
    slice: RopeSlice,
    range: Range,
    textobject: TextObject,
    ch: Option<char>,
    count: usize,
) -> Range {
    let pair_pos = match ch {
        Some(ch) => surround::find_nth_pairs_pos(slice, ch, range, count),
        // Automatically find the closest surround pairs
        None => surround::find_nth_closest_pairs_pos(slice, range, count),
    };
    pair_pos
        .map(|(anchor, head)| match textobject {
            TextObject::Inside => {
                if anchor < head {
                    Range::new(next_grapheme_boundary(slice, anchor), head)
                } else {
                    Range::new(anchor, next_grapheme_boundary(slice, head))
                }
            }
            TextObject::Around => {
                if anchor < head {
                    Range::new(anchor, next_grapheme_boundary(slice, head))
                } else {
                    Range::new(next_grapheme_boundary(slice, anchor), head)
                }
            }
            TextObject::Movement => unreachable!(),
        })
        .unwrap_or(range)
}

/// `ts_node_name` is a query capture base name like "function", "class", etc.
/// Finds the range of the next or previous textobject in the node if `rule_cx.direction` is `Some(Direction)`.
/// Grab texobject which the cursor is placed on the field is `None`.
/// Returns the range in the forwards direction.
#[allow(non_upper_case_globals)]
pub const treesitter: SelectionRule = |range, rule_cx| {
    let ts_object_query = rule_cx
        .lang_confing
        .and_then(|lang_config| lang_config.textobject_query())
        .expect("run_condition should check for document lang_config textobject support");

    let syntax_root_node = rule_cx
        .syntax
        .expect("run_condition shuld check for an active tree-sitter syntax tree")
        .tree()
        .root_node();

    let ts_node = rule_cx
        .ts_node
        .expect("ts_node name should be set when calling goto_tree_sitter_object");

    let capture_name = |text_object: TextObject| format!("{}.{}", ts_node, text_object);
    let capture_names = match rule_cx.ts_textobject {
        Some(text_object) => vec![capture_name(text_object)],
        None => vec![
            capture_name(TextObject::Movement),
            capture_name(TextObject::Around),
        ],
    };
    let capture_names: &[&str] = &capture_names
        .iter()
        .map(|capture_name| capture_name.as_str())
        .collect::<Vec<&str>>()[..];

    let text = rule_cx.text;
    let curr_line_nr = range.cursor_line(text);
    let found_byte_range = match rule_cx.direction {
        Some(direction) => match direction {
            Direction::Forward => {
                if curr_line_nr < text.len_lines() {
                    let next_line_byte_pos = text.line_to_byte(curr_line_nr + 1);
                    Some(next_line_byte_pos..text.len_bytes())
                } else {
                    return range;
                }
            }
            Direction::Backward => {
                if curr_line_nr > 1 {
                    Some(0..text.line_to_byte(curr_line_nr - 1))
                } else {
                    return range;
                }
            }
        },
        None => None,
    };
    let found_nodes =
        ts_object_query.capture_nodes_any(capture_names, &syntax_root_node, text, found_byte_range);
    let found_node: Option<&tree_sitter::Node> = match rule_cx.direction {
        Some(direction) => match direction {
            Direction::Forward => found_nodes.first(),
            Direction::Backward => found_nodes.last(),
        },
        None => {
            // TODO: find nearest for m{i,a}, probably similar to how expand/shrink was done with binary sort
            todo!()
        }
    };
    // (Returning the input range in a SelectionRule is equivalent to letting
    // the caller know that the count repeat should be cancelled.)
    let Some(node) = found_node else {
        return range;
    };

    let start_char = text.byte_to_char(node.start_byte());
    let end_char = text.byte_to_char(node.end_byte());
    let new_range: Range = Range::new(start_char, end_char);
    match rule_cx.direction {
        Some(direction) => {
            if rule_cx.extend {
                let head = if new_range.head < range.anchor {
                    new_range.anchor
                } else {
                    new_range.head
                };

                Range::new(range.anchor, head)
            } else {
                new_range.with_direction(direction)
            }
        }
        None => new_range,
    }
};
