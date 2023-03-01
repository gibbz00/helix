use std::cell::RefCell;

use ropey::Rope;

use crate::{coords_at_pos, pos_at_coords, selection::RuleContext};

use super::*;

const SINGLE_LINE_SAMPLE: &str = "This is a simple alphabetic line";
const MULTILINE_SAMPLE: &str = "\
        Multiline\n\
        text sample\n\
        which\n\
        is merely alphabetic\n\
        and whitespaced\n\
    ";

const MULTIBYTE_CHARACTER_SAMPLE: &str = "\
        パーティーへ行かないか\n\
        The text above is Japanese\n\
    ";

#[test]
fn vertical_move() {
    let text = Rope::from("abcd\nefg\nwrs");
    let slice = text.slice(..);
    let pos = pos_at_coords(slice, (0, 4).into(), true);

    let range = Range::new(pos, pos);
    assert_eq!(
        coords_at_pos(
            slice,
            move_vertically_visual(
                slice,
                range,
                Direction::Forward,
                1,
                Movement::Move,
                &TextFormat::default(),
                &mut TextAnnotations::default(),
            )
            .head
        ),
        (1, 3).into()
    );
}

#[test]
fn horizontal_moves_through_single_line_text() {
    let text = Rope::from(SINGLE_LINE_SAMPLE);
    let slice = text.slice(..);
    let position = pos_at_coords(slice, (0, 0).into(), true);

    let mut range = Range::point(position);

    let moves_and_expected_coordinates = [
        ((Direction::Forward, 1usize), (0, 1)), // T|his is a simple alphabetic line
        ((Direction::Forward, 2usize), (0, 3)), // Thi|s is a simple alphabetic line
        ((Direction::Forward, 0usize), (0, 3)), // Thi|s is a simple alphabetic line
        ((Direction::Forward, 999usize), (0, 32)), // This is a simple alphabetic line|
        ((Direction::Forward, 999usize), (0, 32)), // This is a simple alphabetic line|
        ((Direction::Backward, 999usize), (0, 0)), // |This is a simple alphabetic line
    ];

    for ((direction, amount), coordinates) in moves_and_expected_coordinates {
        range = move_horizontally(
            slice,
            range,
            direction,
            amount,
            Movement::Move,
            &TextFormat::default(),
            &mut TextAnnotations::default(),
        );
        assert_eq!(coords_at_pos(slice, range.head), coordinates.into())
    }
}

#[test]
fn horizontal_moves_through_multiline_text() {
    let text = Rope::from(MULTILINE_SAMPLE);
    let slice = text.slice(..);
    let position = pos_at_coords(slice, (0, 0).into(), true);

    let mut range = Range::point(position);

    let moves_and_expected_coordinates = [
        ((Direction::Forward, 11usize), (1, 1)), // Multiline\nt|ext sample\n...
        ((Direction::Backward, 1usize), (1, 0)), // Multiline\n|text sample\n...
        ((Direction::Backward, 5usize), (0, 5)), // Multi|line\ntext sample\n...
        ((Direction::Backward, 999usize), (0, 0)), // |Multiline\ntext sample\n...
        ((Direction::Forward, 3usize), (0, 3)),  // Mul|tiline\ntext sample\n...
        ((Direction::Forward, 0usize), (0, 3)),  // Mul|tiline\ntext sample\n...
        ((Direction::Backward, 0usize), (0, 3)), // Mul|tiline\ntext sample\n...
        ((Direction::Forward, 999usize), (5, 0)), // ...and whitespaced\n|
        ((Direction::Forward, 999usize), (5, 0)), // ...and whitespaced\n|
    ];

    for ((direction, amount), coordinates) in moves_and_expected_coordinates {
        range = move_horizontally(
            slice,
            range,
            direction,
            amount,
            Movement::Move,
            &TextFormat::default(),
            &mut TextAnnotations::default(),
        );
        assert_eq!(coords_at_pos(slice, range.head), coordinates.into());
        assert_eq!(range.head, range.anchor);
    }
}

#[test]
fn selection_extending_moves_in_single_line_text() {
    let text = Rope::from(SINGLE_LINE_SAMPLE);
    let slice = text.slice(..);
    let position = pos_at_coords(slice, (0, 0).into(), true);

    let mut range = Range::point(position);
    let original_anchor = range.anchor;

    let moves = [
        (Direction::Forward, 1usize),
        (Direction::Forward, 5usize),
        (Direction::Backward, 3usize),
    ];

    for (direction, amount) in moves {
        range = move_horizontally(
            slice,
            range,
            direction,
            amount,
            Movement::Extend,
            &TextFormat::default(),
            &mut TextAnnotations::default(),
        );
        assert_eq!(range.anchor, original_anchor);
    }
}

#[test]
fn vertical_moves_in_single_column() {
    let text = Rope::from(MULTILINE_SAMPLE);
    let slice = text.slice(..);
    let position = pos_at_coords(slice, (0, 0).into(), true);
    let mut range = Range::point(position);
    let moves_and_expected_coordinates = [
        ((Direction::Forward, 1usize), (1, 0)),
        ((Direction::Forward, 2usize), (3, 0)),
        ((Direction::Forward, 1usize), (4, 0)),
        ((Direction::Backward, 999usize), (0, 0)),
        ((Direction::Forward, 4usize), (4, 0)),
        ((Direction::Forward, 0usize), (4, 0)),
        ((Direction::Backward, 0usize), (4, 0)),
        ((Direction::Forward, 5), (5, 0)),
        ((Direction::Forward, 999usize), (5, 0)),
    ];

    for ((direction, amount), coordinates) in moves_and_expected_coordinates {
        range = move_vertically_visual(
            slice,
            range,
            direction,
            amount,
            Movement::Move,
            &TextFormat::default(),
            &mut TextAnnotations::default(),
        );
        assert_eq!(coords_at_pos(slice, range.head), coordinates.into());
        assert_eq!(range.head, range.anchor);
    }
}

#[test]
fn vertical_moves_jumping_column() {
    let text = Rope::from(MULTILINE_SAMPLE);
    let slice = text.slice(..);
    let position = pos_at_coords(slice, (0, 0).into(), true);
    let mut range = Range::point(position);

    enum Axis {
        H,
        V,
    }
    let moves_and_expected_coordinates = [
        // Places cursor at the end of line
        ((Axis::H, Direction::Forward, 8usize), (0, 8)),
        // First descent preserves column as the target line is wider
        ((Axis::V, Direction::Forward, 1usize), (1, 8)),
        // Second descent clamps column as the target line is shorter
        ((Axis::V, Direction::Forward, 1usize), (2, 5)),
        // Third descent restores the original column
        ((Axis::V, Direction::Forward, 1usize), (3, 8)),
        // Behaviour is preserved even through long jumps
        ((Axis::V, Direction::Backward, 999usize), (0, 8)),
        ((Axis::V, Direction::Forward, 4usize), (4, 8)),
        ((Axis::V, Direction::Forward, 999usize), (5, 0)),
    ];

    for ((axis, direction, amount), coordinates) in moves_and_expected_coordinates {
        range = match axis {
            Axis::H => move_horizontally(
                slice,
                range,
                direction,
                amount,
                Movement::Move,
                &TextFormat::default(),
                &mut TextAnnotations::default(),
            ),
            Axis::V => move_vertically_visual(
                slice,
                range,
                direction,
                amount,
                Movement::Move,
                &TextFormat::default(),
                &mut TextAnnotations::default(),
            ),
        };
        assert_eq!(coords_at_pos(slice, range.head), coordinates.into());
        assert_eq!(range.head, range.anchor);
    }
}

#[test]
fn multibyte_character_wide_column_jumps() {
    let text = Rope::from(MULTIBYTE_CHARACTER_SAMPLE);
    let slice = text.slice(..);
    let position = pos_at_coords(slice, (0, 0).into(), true);
    let mut range = Range::point(position);

    // FIXME: The behaviour captured in this test diverges from both Kakoune and Vim. These
    // will attempt to preserve the horizontal position of the cursor, rather than
    // placing it at the same character index.
    enum Axis {
        H,
        V,
    }
    let moves_and_expected_coordinates = [
        // Places cursor at the fourth kana.
        ((Axis::H, Direction::Forward, 4), (0, 4)),
        // Descent places cursor at the 8th character.
        ((Axis::V, Direction::Forward, 1usize), (1, 8)),
        // Moving back 2 characters.
        ((Axis::H, Direction::Backward, 2usize), (1, 6)),
        // Jumping back up 1 line.
        ((Axis::V, Direction::Backward, 1usize), (0, 3)),
    ];

    for ((axis, direction, amount), coordinates) in moves_and_expected_coordinates {
        range = match axis {
            Axis::H => move_horizontally(
                slice,
                range,
                direction,
                amount,
                Movement::Move,
                &TextFormat::default(),
                &mut TextAnnotations::default(),
            ),
            Axis::V => move_vertically_visual(
                slice,
                range,
                direction,
                amount,
                Movement::Move,
                &TextFormat::default(),
                &mut TextAnnotations::default(),
            ),
        };
        assert_eq!(coords_at_pos(slice, range.head), coordinates.into());
        assert_eq!(range.head, range.anchor);
    }
}

#[test]
#[should_panic]
fn nonsensical_ranges_panic_on_forward_movement_attempt_in_debug_mode() {
    move_next_word_start(Rope::from("Sample").slice(..), Range::point(99999999), 1);
}

#[test]
#[should_panic]
fn nonsensical_ranges_panic_on_forward_to_end_movement_attempt_in_debug_mode() {
    move_next_word_end(Rope::from("Sample").slice(..), Range::point(99999999), 1);
}

#[test]
#[should_panic]
fn nonsensical_ranges_panic_on_backwards_movement_attempt_in_debug_mode() {
    move_prev_word_start(Rope::from("Sample").slice(..), Range::point(99999999), 1);
}

#[test]
fn move_to_start_of_next_words() {
    let tests = [
            ("Basic forward motion stops at the first space",
                vec![(1, Range::new(0, 0), Range::new(0, 6))]),
            (" Starting from a boundary advances the anchor",
                vec![(1, Range::new(0, 0), Range::new(1, 10))]),
            ("Long       whitespace gap is bridged by the head",
                vec![(1, Range::new(0, 0), Range::new(0, 11))]),
            ("Previous anchor is irrelevant for forward motions",
                vec![(1, Range::new(12, 0), Range::new(0, 9))]),
            ("    Starting from whitespace moves to last space in sequence",
                vec![(1, Range::new(0, 0), Range::new(0, 4))]),
            ("Starting from mid-word leaves anchor at start position and moves head",
                vec![(1, Range::new(3, 3), Range::new(3, 9))]),
            ("Identifiers_with_underscores are considered a single word",
                vec![(1, Range::new(0, 0), Range::new(0, 29))]),
            ("Jumping\n    into starting whitespace selects the spaces before 'into'",
                vec![(1, Range::new(0, 7), Range::new(8, 12))]),
            ("alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 12)),
                    (1, Range::new(0, 12), Range::new(12, 15)),
                    (1, Range::new(12, 15), Range::new(15, 18))
                ]),
            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 6)),
                    (1, Range::new(0, 6), Range::new(6, 10)),
                ]),
            (".._.._ punctuation is not joined by underscores into a single block",
                vec![(1, Range::new(0, 0), Range::new(0, 2))]),
            ("Newlines\n\nare bridged seamlessly.",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 8)),
                    (1, Range::new(0, 8), Range::new(10, 14)),
                ]),
            ("Jumping\n\n\n\n\n\n   from newlines to whitespace selects whitespace.",
                vec![
                    (1, Range::new(0, 9), Range::new(13, 16)),
                ]),
            ("A failed motion does not modify the range",
                vec![
                    (3, Range::new(37, 41), Range::new(37, 41)),
                ]),
            ("oh oh oh two character words!",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 3)),
                    (1, Range::new(0, 3), Range::new(3, 6)),
                    (1, Range::new(0, 2), Range::new(1, 3)),
                ]),
            ("Multiple motions at once resolve correctly",
                vec![
                    (3, Range::new(0, 0), Range::new(17, 20)),
                ]),
            ("Excessive motions are performed partially",
                vec![
                    (999, Range::new(0, 0), Range::new(32, 41)),
                ]),
            ("", // Edge case of moving forward in empty string
                vec![
                    (1, Range::new(0, 0), Range::new(0, 0)),
                ]),
            ("\n\n\n\n\n", // Edge case of moving forward in all newlines
                vec![
                    (1, Range::new(0, 0), Range::new(5, 5)),
                ]),
            ("\n   \n   \n Jumping through alternated space blocks and newlines selects the space blocks",
                vec![
                    (1, Range::new(0, 0), Range::new(1, 4)),
                    (1, Range::new(1, 4), Range::new(5, 8)),
                ]),
            ("ヒーリクス multibyte characters behave as normal characters",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 6)),
                ]),
        ];

    for (sample, scenario) in tests {
        for (count, begin, expected_end) in scenario.into_iter() {
            let range = move_next_word_start(Rope::from(sample).slice(..), begin, count);
            assert_eq!(range, expected_end, "Case failed: [{}]", sample);
        }
    }
}

#[test]
fn move_to_start_of_next_long_words() {
    let tests = [
            ("Basic forward motion stops at the first space",
                vec![(1, Range::new(0, 0), Range::new(0, 6))]),
            (" Starting from a boundary advances the anchor",
                vec![(1, Range::new(0, 0), Range::new(1, 10))]),
            ("Long       whitespace gap is bridged by the head",
                vec![(1, Range::new(0, 0), Range::new(0, 11))]),
            ("Previous anchor is irrelevant for forward motions",
                vec![(1, Range::new(12, 0), Range::new(0, 9))]),
            ("    Starting from whitespace moves to last space in sequence",
                vec![(1, Range::new(0, 0), Range::new(0, 4))]),
            ("Starting from mid-word leaves anchor at start position and moves head",
                vec![(1, Range::new(3, 3), Range::new(3, 9))]),
            ("Identifiers_with_underscores are considered a single word",
                vec![(1, Range::new(0, 0), Range::new(0, 29))]),
            ("Jumping\n    into starting whitespace selects the spaces before 'into'",
                vec![(1, Range::new(0, 7), Range::new(8, 12))]),
            ("alphanumeric.!,and.?=punctuation are not treated any differently than alphanumerics",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 33)),
                ]),
            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 6)),
                    (1, Range::new(0, 6), Range::new(6, 10)),
                ]),
            (".._.._ punctuation is joined by underscores into a single word, as it behaves like alphanumerics",
                vec![(1, Range::new(0, 0), Range::new(0, 7))]),
            ("Newlines\n\nare bridged seamlessly.",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 8)),
                    (1, Range::new(0, 8), Range::new(10, 14)),
                ]),
            ("Jumping\n\n\n\n\n\n   from newlines to whitespace selects whitespace.",
                vec![
                    (1, Range::new(0, 9), Range::new(13, 16)),
                ]),
            ("A failed motion does not modify the range",
                vec![
                    (3, Range::new(37, 41), Range::new(37, 41)),
                ]),
            ("oh oh oh two character words!",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 3)),
                    (1, Range::new(0, 3), Range::new(3, 6)),
                    (1, Range::new(0, 1), Range::new(0, 3)),
                ]),
            ("Multiple motions at once resolve correctly",
                vec![
                    (3, Range::new(0, 0), Range::new(17, 20)),
                ]),
            ("Excessive motions are performed partially",
                vec![
                    (999, Range::new(0, 0), Range::new(32, 41)),
                ]),
            ("", // Edge case of moving forward in empty string
                vec![
                    (1, Range::new(0, 0), Range::new(0, 0)),
                ]),
            ("\n\n\n\n\n", // Edge case of moving forward in all newlines
                vec![
                    (1, Range::new(0, 0), Range::new(5, 5)),
                ]),
            ("\n   \n   \n Jumping through alternated space blocks and newlines selects the space blocks",
                vec![
                    (1, Range::new(0, 0), Range::new(1, 4)),
                    (1, Range::new(1, 4), Range::new(5, 8)),
                ]),
            ("ヒー..リクス multibyte characters behave as normal characters, including their interaction with punctuation",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 8)),
                ]),
        ];

    for (sample, scenario) in tests {
        for (count, begin, expected_end) in scenario.into_iter() {
            let range = move_next_long_word_start(Rope::from(sample).slice(..), begin, count);
            assert_eq!(range, expected_end, "Case failed: [{}]", sample);
        }
    }
}

#[test]
fn move_to_start_of_previous_words() {
    let tests = [
            ("Basic backward motion from the middle of a word",
                vec![(1, Range::new(3, 3), Range::new(4, 0))]),

            // // Why do we want this behavior?  The current behavior fails this
            // // test, but seems better and more consistent.
            // ("Starting from after boundary retreats the anchor",
            //     vec![(1, Range::new(0, 9), Range::new(8, 0))]),

            ("    Jump to start of a word preceded by whitespace",
                vec![(1, Range::new(5, 5), Range::new(6, 4))]),
            ("    Jump to start of line from start of word preceded by whitespace",
                vec![(1, Range::new(4, 4), Range::new(4, 0))]),
            ("Previous anchor is irrelevant for backward motions",
                vec![(1, Range::new(12, 5), Range::new(6, 0))]),
            ("    Starting from whitespace moves to first space in sequence",
                vec![(1, Range::new(0, 4), Range::new(4, 0))]),
            ("Identifiers_with_underscores are considered a single word",
                vec![(1, Range::new(0, 20), Range::new(20, 0))]),
            ("Jumping\n    \nback through a newline selects whitespace",
                vec![(1, Range::new(0, 13), Range::new(12, 8))]),
            ("Jumping to start of word from the end selects the word",
                vec![(1, Range::new(6, 7), Range::new(7, 0))]),
            ("alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion",
                vec![
                    (1, Range::new(29, 30), Range::new(30, 21)),
                    (1, Range::new(30, 21), Range::new(21, 18)),
                    (1, Range::new(21, 18), Range::new(18, 15))
                ]),
            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 10), Range::new(10, 6)),
                    (1, Range::new(10, 6), Range::new(6, 0)),
                ]),
            (".._.._ punctuation is not joined by underscores into a single block",
                vec![(1, Range::new(0, 6), Range::new(5, 3))]),
            ("Newlines\n\nare bridged seamlessly.",
                vec![
                    (1, Range::new(0, 10), Range::new(8, 0)),
                ]),
            ("Jumping    \n\n\n\n\nback from within a newline group selects previous block",
                vec![
                    (1, Range::new(0, 13), Range::new(11, 0)),
                ]),
            ("Failed motions do not modify the range",
                vec![
                    (0, Range::new(3, 0), Range::new(3, 0)),
                ]),
            ("Multiple motions at once resolve correctly",
                vec![
                    (3, Range::new(18, 18), Range::new(9, 0)),
                ]),
            ("Excessive motions are performed partially",
                vec![
                    (999, Range::new(40, 40), Range::new(10, 0)),
                ]),
            ("", // Edge case of moving backwards in empty string
                vec![
                    (1, Range::new(0, 0), Range::new(0, 0)),
                ]),
            ("\n\n\n\n\n", // Edge case of moving backwards in all newlines
                vec![
                    (1, Range::new(5, 5), Range::new(0, 0)),
                ]),
            ("   \n   \nJumping back through alternated space blocks and newlines selects the space blocks",
                vec![
                    (1, Range::new(0, 8), Range::new(7, 4)),
                    (1, Range::new(7, 4), Range::new(3, 0)),
                ]),
            ("ヒーリクス multibyte characters behave as normal characters",
                vec![
                    (1, Range::new(0, 6), Range::new(6, 0)),
                ]),
        ];

    for (sample, scenario) in tests {
        for (count, begin, expected_end) in scenario.into_iter() {
            let range = move_prev_word_start(Rope::from(sample).slice(..), begin, count);
            assert_eq!(range, expected_end, "Case failed: [{}]", sample);
        }
    }
}

#[test]
fn move_to_start_of_previous_long_words() {
    let tests = [
            (
                "Basic backward motion from the middle of a word",
                vec![(1, Range::new(3, 3), Range::new(4, 0))],
            ),

            // // Why do we want this behavior?  The current behavior fails this
            // // test, but seems better and more consistent.
            // ("Starting from after boundary retreats the anchor",
            //     vec![(1, Range::new(0, 9), Range::new(8, 0))]),

            (
                "    Jump to start of a word preceded by whitespace",
                vec![(1, Range::new(5, 5), Range::new(6, 4))],
            ),
            (
                "    Jump to start of line from start of word preceded by whitespace",
                vec![(1, Range::new(3, 4), Range::new(4, 0))],
            ),
            ("Previous anchor is irrelevant for backward motions",
                vec![(1, Range::new(12, 5), Range::new(6, 0))]),
            (
                "    Starting from whitespace moves to first space in sequence",
                vec![(1, Range::new(0, 4), Range::new(4, 0))],
            ),
            ("Identifiers_with_underscores are considered a single word",
                vec![(1, Range::new(0, 20), Range::new(20, 0))]),
            (
                "Jumping\n    \nback through a newline selects whitespace",
                vec![(1, Range::new(0, 13), Range::new(12, 8))],
            ),
            (
                "Jumping to start of word from the end selects the word",
                vec![(1, Range::new(6, 7), Range::new(7, 0))],
            ),
            (
                "alphanumeric.!,and.?=punctuation are treated exactly the same",
                vec![(1, Range::new(29, 30), Range::new(30, 0))],
            ),
            (
                "...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 10), Range::new(10, 6)),
                    (1, Range::new(10, 6), Range::new(6, 0)),
                ],
            ),
            (".._.._ punctuation is joined by underscores into a single block",
                vec![(1, Range::new(0, 6), Range::new(6, 0))]),
            (
                "Newlines\n\nare bridged seamlessly.",
                vec![(1, Range::new(0, 10), Range::new(8, 0))],
            ),
            (
                "Jumping    \n\n\n\n\nback from within a newline group selects previous block",
                vec![(1, Range::new(0, 13), Range::new(11, 0))],
            ),
            (
                "Failed motions do not modify the range",
                vec![(0, Range::new(3, 0), Range::new(3, 0))],
            ),
            (
                "Multiple motions at once resolve correctly",
                vec![(3, Range::new(19, 19), Range::new(9, 0))],
            ),
            (
                "Excessive motions are performed partially",
                vec![(999, Range::new(40, 40), Range::new(10, 0))],
            ),
            (
                "", // Edge case of moving backwards in empty string
                vec![(1, Range::new(0, 0), Range::new(0, 0))],
            ),
            (
                "\n\n\n\n\n", // Edge case of moving backwards in all newlines
                vec![(1, Range::new(5, 5), Range::new(0, 0))],
            ),
            ("   \n   \nJumping back through alternated space blocks and newlines selects the space blocks",
                vec![
                    (1, Range::new(0, 8), Range::new(7, 4)),
                    (1, Range::new(7, 4), Range::new(3, 0)),
                ]),
            ("ヒーリ..クス multibyte characters behave as normal characters, including when interacting with punctuation",
                vec![
                    (1, Range::new(0, 8), Range::new(8, 0)),
                ]),
        ];

    for (sample, scenario) in tests {
        for (count, begin, expected_end) in scenario.into_iter() {
            let range = move_prev_long_word_start(Rope::from(sample).slice(..), begin, count);
            assert_eq!(range, expected_end, "Case failed: [{}]", sample);
        }
    }
}

#[test]
fn move_to_end_of_next_words() {
    let tests = [
            ("Basic forward motion from the start of a word to the end of it",
                vec![(1, Range::new(0, 0), Range::new(0, 5))]),
            ("Basic forward motion from the end of a word to the end of the next",
                vec![(1, Range::new(0, 5), Range::new(5, 13))]),
            ("Basic forward motion from the middle of a word to the end of it",
                vec![(1, Range::new(2, 2), Range::new(2, 5))]),
            ("    Jumping to end of a word preceded by whitespace",
                vec![(1, Range::new(0, 0), Range::new(0, 11))]),

            // // Why do we want this behavior?  The current behavior fails this
            // // test, but seems better and more consistent.
            // (" Starting from a boundary advances the anchor",
            //     vec![(1, Range::new(0, 0), Range::new(1, 9))]),

            ("Previous anchor is irrelevant for end of word motion",
                vec![(1, Range::new(12, 2), Range::new(2, 8))]),
            ("Identifiers_with_underscores are considered a single word",
                vec![(1, Range::new(0, 0), Range::new(0, 28))]),
            ("Jumping\n    into starting whitespace selects up to the end of next word",
                vec![(1, Range::new(0, 7), Range::new(8, 16))]),
            ("alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 12)),
                    (1, Range::new(0, 12), Range::new(12, 15)),
                    (1, Range::new(12, 15), Range::new(15, 18))
                ]),
            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 3)),
                    (1, Range::new(0, 3), Range::new(3, 9)),
                ]),
            (".._.._ punctuation is not joined by underscores into a single block",
                vec![(1, Range::new(0, 0), Range::new(0, 2))]),
            ("Newlines\n\nare bridged seamlessly.",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 8)),
                    (1, Range::new(0, 8), Range::new(10, 13)),
                ]),
            ("Jumping\n\n\n\n\n\n   from newlines to whitespace selects to end of next word.",
                vec![
                    (1, Range::new(0, 8), Range::new(13, 20)),
                ]),
            ("A failed motion does not modify the range",
                vec![
                    (3, Range::new(37, 41), Range::new(37, 41)),
                ]),
            ("Multiple motions at once resolve correctly",
                vec![
                    (3, Range::new(0, 0), Range::new(16, 19)),
                ]),
            ("Excessive motions are performed partially",
                vec![
                    (999, Range::new(0, 0), Range::new(31, 41)),
                ]),
            ("", // Edge case of moving forward in empty string
                vec![
                    (1, Range::new(0, 0), Range::new(0, 0)),
                ]),
            ("\n\n\n\n\n", // Edge case of moving forward in all newlines
                vec![
                    (1, Range::new(0, 0), Range::new(5, 5)),
                ]),
            ("\n   \n   \n Jumping through alternated space blocks and newlines selects the space blocks",
                vec![
                    (1, Range::new(0, 0), Range::new(1, 4)),
                    (1, Range::new(1, 4), Range::new(5, 8)),
                ]),
            ("ヒーリクス multibyte characters behave as normal characters",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 5)),
                ]),
        ];

    for (sample, scenario) in tests {
        for (count, begin, expected_end) in scenario.into_iter() {
            let range = move_next_word_end(Rope::from(sample).slice(..), begin, count);
            assert_eq!(range, expected_end, "Case failed: [{}]", sample);
        }
    }
}

#[test]
fn move_to_end_of_previous_words() {
    let tests = [
            ("Basic backward motion from the middle of a word",
                vec![(1, Range::new(9, 9), Range::new(10, 5))]),
            ("Starting from after boundary retreats the anchor",
                vec![(1, Range::new(0, 14), Range::new(13, 8))]),
            ("Jump     to end of a word succeeded by whitespace",
                vec![(1, Range::new(11, 11), Range::new(11, 4))]),
            ("    Jump to start of line from end of word preceded by whitespace",
                vec![(1, Range::new(8, 8), Range::new(8, 0))]),
            ("Previous anchor is irrelevant for backward motions",
                vec![(1, Range::new(26, 12), Range::new(13, 8))]),
            ("    Starting from whitespace moves to first space in sequence",
                vec![(1, Range::new(0, 4), Range::new(4, 0))]),
            ("Test identifiers_with_underscores are considered a single word",
                vec![(1, Range::new(0, 25), Range::new(25, 4))]),
            ("Jumping\n    \nback through a newline selects whitespace",
                vec![(1, Range::new(0, 13), Range::new(12, 8))]),
            ("Jumping to start of word from the end selects the whole word",
                vec![(1, Range::new(16, 16), Range::new(16, 10))]),
            ("alphanumeric.!,and.?=punctuation are considered 'words' for the purposes of word motion",
                vec![
                    (1, Range::new(30, 30), Range::new(31, 21)),
                    (1, Range::new(31, 21), Range::new(21, 18)),
                    (1, Range::new(21, 18), Range::new(18, 15))
                ]),

            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 10), Range::new(9, 3)),
                    (1, Range::new(9, 3), Range::new(3, 0)),
                ]),
            (".._.._ punctuation is not joined by underscores into a single block",
                vec![(1, Range::new(0, 5), Range::new(5, 3))]),
            ("Newlines\n\nare bridged seamlessly.",
                vec![
                    (1, Range::new(0, 10), Range::new(8, 0)),
                ]),
            ("Jumping    \n\n\n\n\nback from within a newline group selects previous block",
                vec![
                    (1, Range::new(0, 13), Range::new(11, 7)),
                ]),
            ("Failed motions do not modify the range",
                vec![
                    (0, Range::new(3, 0), Range::new(3, 0)),
                ]),
            ("Multiple motions at once resolve correctly",
                vec![
                    (3, Range::new(24, 24), Range::new(16, 8)),
                ]),
            ("Excessive motions are performed partially",
                vec![
                    (999, Range::new(40, 40), Range::new(9, 0)),
                ]),
            ("", // Edge case of moving backwards in empty string
                vec![
                    (1, Range::new(0, 0), Range::new(0, 0)),
                ]),
            ("\n\n\n\n\n", // Edge case of moving backwards in all newlines
                vec![
                    (1, Range::new(5, 5), Range::new(0, 0)),
                ]),
            ("   \n   \nJumping back through alternated space blocks and newlines selects the space blocks",
                vec![
                    (1, Range::new(0, 8), Range::new(7, 4)),
                    (1, Range::new(7, 4), Range::new(3, 0)),
                ]),
            ("Test ヒーリクス multibyte characters behave as normal characters",
                vec![
                    (1, Range::new(0, 10), Range::new(10, 4)),
                ]),
        ];

    for (sample, scenario) in tests {
        for (count, begin, expected_end) in scenario.into_iter() {
            let range = move_prev_word_end(Rope::from(sample).slice(..), begin, count);
            assert_eq!(range, expected_end, "Case failed: [{}]", sample);
        }
    }
}

#[test]
fn move_to_end_of_next_long_words() {
    let tests = [
            ("Basic forward motion from the start of a word to the end of it",
                vec![(1, Range::new(0, 0), Range::new(0, 5))]),
            ("Basic forward motion from the end of a word to the end of the next",
                vec![(1, Range::new(0, 5), Range::new(5, 13))]),
            ("Basic forward motion from the middle of a word to the end of it",
                vec![(1, Range::new(2, 2), Range::new(2, 5))]),
            ("    Jumping to end of a word preceded by whitespace",
                vec![(1, Range::new(0, 0), Range::new(0, 11))]),

            // // Why do we want this behavior?  The current behavior fails this
            // // test, but seems better and more consistent.
            // (" Starting from a boundary advances the anchor",
            //     vec![(1, Range::new(0, 0), Range::new(1, 9))]),

            ("Previous anchor is irrelevant for end of word motion",
                vec![(1, Range::new(12, 2), Range::new(2, 8))]),
            ("Identifiers_with_underscores are considered a single word",
                vec![(1, Range::new(0, 0), Range::new(0, 28))]),
            ("Jumping\n    into starting whitespace selects up to the end of next word",
                vec![(1, Range::new(0, 7), Range::new(8, 16))]),
            ("alphanumeric.!,and.?=punctuation are treated the same way",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 32)),
                ]),
            ("...   ... punctuation and spaces behave as expected",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 3)),
                    (1, Range::new(0, 3), Range::new(3, 9)),
                ]),
            (".._.._ punctuation is joined by underscores into a single block",
                vec![(1, Range::new(0, 0), Range::new(0, 6))]),
            ("Newlines\n\nare bridged seamlessly.",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 8)),
                    (1, Range::new(0, 8), Range::new(10, 13)),
                ]),
            ("Jumping\n\n\n\n\n\n   from newlines to whitespace selects to end of next word.",
                vec![
                    (1, Range::new(0, 9), Range::new(13, 20)),
                ]),
            ("A failed motion does not modify the range",
                vec![
                    (3, Range::new(37, 41), Range::new(37, 41)),
                ]),
            ("Multiple motions at once resolve correctly",
                vec![
                    (3, Range::new(0, 0), Range::new(16, 19)),
                ]),
            ("Excessive motions are performed partially",
                vec![
                    (999, Range::new(0, 0), Range::new(31, 41)),
                ]),
            ("", // Edge case of moving forward in empty string
                vec![
                    (1, Range::new(0, 0), Range::new(0, 0)),
                ]),
            ("\n\n\n\n\n", // Edge case of moving forward in all newlines
                vec![
                    (1, Range::new(0, 0), Range::new(5, 5)),
                ]),
            ("\n   \n   \n Jumping through alternated space blocks and newlines selects the space blocks",
                vec![
                    (1, Range::new(0, 0), Range::new(1, 4)),
                    (1, Range::new(1, 4), Range::new(5, 8)),
                ]),
            ("ヒーリ..クス multibyte characters behave as normal characters, including  when they interact with punctuation",
                vec![
                    (1, Range::new(0, 0), Range::new(0, 7)),
                ]),
        ];

    for (sample, scenario) in tests {
        for (count, begin, expected_end) in scenario.into_iter() {
            let range = move_next_long_word_end(Rope::from(sample).slice(..), begin, count);
            assert_eq!(range, expected_end, "Case failed: [{}]", sample);
        }
    }
}

#[test]
fn move_to_prev_paragraph_single() {
    let tests = [
        ("#[|]#", "#[|]#"),
        ("#[s|]#tart at\nfirst char\n", "#[|s]#tart at\nfirst char\n"),
        ("start at\nlast char#[\n|]#", "#[|start at\nlast char\n]#"),
        (
            "goto\nfirst\n\n#[p|]#aragraph",
            "#[|goto\nfirst\n\n]#paragraph",
        ),
        (
            "goto\nfirst\n#[\n|]#paragraph",
            "#[|goto\nfirst\n\n]#paragraph",
        ),
        (
            "goto\nsecond\n\np#[a|]#ragraph",
            "goto\nsecond\n\n#[|pa]#ragraph",
        ),
        (
            "here\n\nhave\nmultiple\nparagraph\n\n\n\n\n#[|]#",
            "here\n\n#[|have\nmultiple\nparagraph\n\n\n\n\n]#",
        ),
    ];

    for (before, expected) in tests {
        test_move_paragraph(before, expected, false, Direction::Backward, 1)
    }
}

#[test]
fn move_to_prev_paragraph_double() {
    let tests = [
        (
            "on#[e|]#\n\ntwo\n\nthree\n\n",
            "#[|]#one\n\ntwo\n\nthree\n\n",
        ),
        (
            "one\n\ntwo\n\nth#[r|]#ee\n\n",
            "one\n\n#[|two\n\n]#three\n\n",
        ),
    ];

    for (before, expected) in tests {
        test_move_paragraph(before, expected, false, Direction::Backward, 2)
    }
}

#[test]
fn extend_to_prev_paragraph_double() {
    let tests = [
        (
            "on#[e|]#\n\ntwo\n\nthree\n\n",
            "#[|one]#\n\ntwo\n\nthree\n\n",
        ),
        (
            "one\n\ntwo\n\nth#[r|]#ee\n\n",
            "one\n\n#[|two\n\nthr]#ee\n\n",
        ),
    ];

    for (before, expected) in tests {
        test_move_paragraph(before, expected, true, Direction::Backward, 2)
    }
}

#[test]
fn move_to_prev_paragraph_extend() {
    let tests = [
        (
            "one\n\n#[|two\n\n]#three\n\n",
            "#[|one\n\ntwo\n\n]#three\n\n",
        ),
        (
            "#[|one\n\ntwo\n\n]#three\n\n",
            "#[|one\n\ntwo\n\n]#three\n\n",
        ),
    ];

    for (before, expected) in tests {
        test_move_paragraph(before, expected, true, Direction::Backward, 1)
    }
}

#[test]
fn move_to_next_paragraph_single() {
    let tests = [
        ("#[|]#", "#[|]#"),
        ("#[s|]#tart at\nfirst char\n", "#[start at\nfirst char\n|]#"),
        ("start at\nlast char#[\n|]#", "start at\nlast char#[\n|]#"),
        (
            "a\nb\n\n#[g|]#oto\nthird\n\nparagraph",
            "a\nb\n\n#[goto\nthird\n\n|]#paragraph",
        ),
        (
            "a\nb\n#[\n|]#goto\nthird\n\nparagraph",
            "a\nb\n\n#[goto\nthird\n\n|]#paragraph",
        ),
        (
            "a\nb#[\n|]#\ngoto\nsecond\n\nparagraph",
            "a\nb#[\n\n|]#goto\nsecond\n\nparagraph",
        ),
        (
            "here\n\nhave\n#[m|]#ultiple\nparagraph\n\n\n\n\n",
            "here\n\nhave\n#[multiple\nparagraph\n\n\n\n\n|]#",
        ),
        (
            "#[t|]#ext\n\n\nafter two blank lines\n\nmore text\n",
            "#[text\n\n\n|]#after two blank lines\n\nmore text\n",
        ),
        (
            "#[text\n\n\n|]#after two blank lines\n\nmore text\n",
            "text\n\n\n#[after two blank lines\n\n|]#more text\n",
        ),
    ];

    for (before, expected) in tests {
        test_move_paragraph(before, expected, false, Direction::Forward, 1)
    }
}

#[test]
fn move_to_next_paragraph_double() {
    let tests = [
        (
            "one\n\ntwo\n\nth#[r|]#ee\n\n",
            "one\n\ntwo\n\nthree\n#[\n|]#",
        ),
        (
            "on#[e|]#\n\ntwo\n\nthree\n\n",
            "one\n\n#[two\n\n|]#three\n\n",
        ),
    ];

    for (before, expected) in tests {
        test_move_paragraph(before, expected, false, Direction::Forward, 2)
    }
}

#[test]
fn extend_to_next_paragraph_double() {
    let tests = [
        (
            "one\n\ntwo\n\nth#[r|]#ee\n\n",
            "one\n\ntwo\n\nth#[ree\n\n|]#",
        ),
        (
            "on#[e|]#\n\ntwo\n\nthree\n\n",
            "on#[e\n\ntwo\n\n|]#three\n\n",
        ),
    ];

    for (before, expected) in tests {
        test_move_paragraph(before, expected, true, Direction::Forward, 2)
    }
}

#[test]
fn test_behaviour_when_moving_to_next_paragraph_extend() {
    let tests = [
        (
            "one\n\n#[two\n\n|]#three\n\n",
            "one\n\n#[two\n\nthree\n\n|]#",
        ),
        (
            "one\n\n#[two\n\nthree\n\n|]#",
            "one\n\n#[two\n\nthree\n\n|]#",
        ),
    ];

    for (before, expected) in tests {
        test_move_paragraph(before, expected, true, Direction::Forward, 1)
    }
}

fn test_move_paragraph(
    before: &str,
    expected: &str,
    extend: bool,
    direction: Direction,
    count: usize,
) {
    let (s, selection) = crate::test::print(before);
    let selection = selection.transform_pure(
        match direction {
            Direction::Forward => move_next_paragraph,
            Direction::Backward => move_prev_paragraph,
        },
        RuleContext {
            extend,
            text: ropey::Rope::from(s.as_str()).slice(..),
            pos_arg: 0,
            syntax: None,
            direction: None,
            diff_handle: None,
            syntax_find_node_fn: None,
            ts_node: None,
            lang_confing: None,
            find_char: None,
            find_inclusive: None,
            ts_textobject: None,
        },
        std::num::NonZeroUsize::new(count).expect("count should be non-zero usize"),
        None,
        &RefCell::new(Vec::new()),
    );
    let actual = crate::test::plain(&s, selection);
    assert_eq!(actual, expected, "\nbefore: `{:?}`", before);
}
