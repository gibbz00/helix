use std::num::NonZeroUsize;

use super::TextObject::*;
use super::*;

use crate::selection::RuleContext;
use crate::Range;
use ropey::Rope;

#[test]
fn word() {
    // (text, [(char position, textobject, final range), ...])
    let tests = &[
        (
            "cursor at beginning of doc",
            vec![(0, Inside, (0, 6)), (0, Around, (0, 7))],
        ),
        (
            "cursor at middle of word",
            vec![
                (13, Inside, (10, 16)),
                (10, Inside, (10, 16)),
                (15, Inside, (10, 16)),
                (13, Around, (10, 17)),
                (10, Around, (10, 17)),
                (15, Around, (10, 17)),
            ],
        ),
        (
            "cursor between word whitespace",
            vec![(6, Inside, (0, 6)), (6, Around, (0, 7))],
        ),
        (
            "cursor on word before newline\n",
            vec![
                (22, Inside, (22, 29)),
                (28, Inside, (22, 29)),
                (25, Inside, (22, 29)),
                (22, Around, (22, 29)),
                (28, Around, (22, 29)),
                (25, Around, (22, 29)),
            ],
        ),
        (
            "cursor on newline\nnext line",
            vec![(17, Inside, (17, 17)), (17, Around, (17, 17))],
        ),
        (
            "cursor on word after newline\nnext line",
            vec![
                (29, Inside, (29, 33)),
                (30, Inside, (29, 33)),
                (32, Inside, (29, 33)),
                (29, Around, (29, 34)),
                (30, Around, (29, 34)),
                (32, Around, (29, 34)),
            ],
        ),
        (
            "cursor on #$%:;* punctuation",
            vec![
                (13, Inside, (10, 16)),
                (10, Inside, (10, 16)),
                (15, Inside, (10, 16)),
                (13, Around, (10, 17)),
                (10, Around, (10, 17)),
                (15, Around, (10, 17)),
            ],
        ),
        (
            "cursor on punc%^#$:;.tuation",
            vec![
                (14, Inside, (14, 21)),
                (20, Inside, (14, 21)),
                (17, Inside, (14, 21)),
                (14, Around, (14, 21)),
                (20, Around, (14, 21)),
                (17, Around, (14, 21)),
            ],
        ),
        (
            "cursor in   extra whitespace",
            vec![
                (9, Inside, (7, 9)),
                (10, Inside, (7, 9)),
                (11, Inside, (7, 9)),
                (9, Around, (7, 12)),
                (10, Around, (7, 12)),
                (11, Around, (7, 12)),
            ],
        ),
        (
            "cursor on word   with extra whitespace",
            vec![(11, Inside, (10, 14)), (11, Around, (10, 17))],
        ),
        (
            "cursor at end with extra   whitespace",
            vec![(28, Inside, (27, 37)), (28, Around, (27, 37))],
        ),
        (
            "cursor at end of doc",
            vec![(19, Inside, (17, 20)), (19, Around, (17, 20))],
        ),
    ];

    for (sample, scenario) in tests {
        let doc = Rope::from(*sample);
        let slice = doc.slice(..);
        for &case in scenario {
            let (pos, textobject, expected_range) = case;
            // cursor is a single width selection
            let range = Range::new(pos, pos + 1);
            let result = word_impl(range, slice, Some(textobject), false);
            assert_eq!(
                result,
                expected_range.into(),
                "\nCase failed: {:?} - {:?}",
                sample,
                case
            );
        }
    }
}

#[test]
fn paragraph_inside_single() {
    let tests = [
        ("#[|]#", "#[|]#"),
        ("firs#[t|]#\n\nparagraph\n\n", "#[first\n|]#\nparagraph\n\n"),
        (
            "second\n\npa#[r|]#agraph\n\n",
            "second\n\n#[paragraph\n|]#\n",
        ),
        ("#[f|]#irst char\n\n", "#[first char\n|]#\n"),
        ("last char\n#[\n|]#", "#[last char\n|]#\n"),
        (
            "empty to line\n#[\n|]#paragraph boundary\n\n",
            "#[empty to line\n|]#\nparagraph boundary\n\n",
        ),
        (
            "line to empty\n\n#[p|]#aragraph boundary\n\n",
            "line to empty\n\n#[paragraph boundary\n|]#\n",
        ),
    ];

    for (before, expected) in tests {
        test_textobj_paragraph(before, expected, TextObject::Inside, 1);
    }
}

#[test]
fn paragraph_around_single() {
    let tests = [
        ("#[|]#", "#[|]#"),
        ("firs#[t|]#\n\nparagraph\n\n", "#[first\n\n|]#paragraph\n\n"),
        (
            "second\n\npa#[r|]#agraph\n\n",
            "second\n\n#[paragraph\n\n|]#",
        ),
        ("#[f|]#irst char\n\n", "#[first char\n\n|]#"),
        ("last char\n#[\n|]#", "#[last char\n\n|]#"),
        (
            "empty to line\n#[\n|]#paragraph boundary\n\n",
            "#[empty to line\n\n|]#paragraph boundary\n\n",
        ),
        (
            "line to empty\n\n#[p|]#aragraph boundary\n\n",
            "line to empty\n\n#[paragraph boundary\n\n|]#",
        ),
    ];

    for (before, expected) in tests {
        test_textobj_paragraph(before, expected, TextObject::Around, 1);
    }
}

#[test]
fn surround() {
    // (text, [(cursor position, textobject, final range, surround char, count), ...])
    let tests = &[
        (
            "simple (single) surround pairs",
            vec![
                (3, Inside, (3, 3), '(', 1),
                (7, Inside, (8, 14), ')', 1),
                (10, Inside, (8, 14), '(', 1),
                (14, Inside, (8, 14), ')', 1),
                (3, Around, (3, 3), '(', 1),
                (7, Around, (7, 15), ')', 1),
                (10, Around, (7, 15), '(', 1),
                (14, Around, (7, 15), ')', 1),
            ],
        ),
        (
            "samexx 'single' surround pairs",
            vec![
                (3, Inside, (3, 3), '\'', 1),
                (7, Inside, (7, 7), '\'', 1),
                (10, Inside, (8, 14), '\'', 1),
                (14, Inside, (14, 14), '\'', 1),
                (3, Around, (3, 3), '\'', 1),
                (7, Around, (7, 7), '\'', 1),
                (10, Around, (7, 15), '\'', 1),
                (14, Around, (14, 14), '\'', 1),
            ],
        ),
        (
            "(nested (surround (pairs)) 3 levels)",
            vec![
                (0, Inside, (1, 35), '(', 1),
                (6, Inside, (1, 35), ')', 1),
                (8, Inside, (9, 25), '(', 1),
                (8, Inside, (9, 35), ')', 2),
                (20, Inside, (9, 25), '(', 2),
                (20, Inside, (1, 35), ')', 3),
                (0, Around, (0, 36), '(', 1),
                (6, Around, (0, 36), ')', 1),
                (8, Around, (8, 26), '(', 1),
                (8, Around, (8, 36), ')', 2),
                (20, Around, (8, 26), '(', 2),
                (20, Around, (0, 36), ')', 3),
            ],
        ),
        (
            "(mixed {surround [pair] same} line)",
            vec![
                (2, Inside, (1, 34), '(', 1),
                (9, Inside, (8, 28), '{', 1),
                (18, Inside, (18, 22), '[', 1),
                (2, Around, (0, 35), '(', 1),
                (9, Around, (7, 29), '{', 1),
                (18, Around, (17, 23), '[', 1),
            ],
        ),
        (
            "(stepped (surround) pairs (should) skip)",
            vec![(22, Inside, (1, 39), '(', 1), (22, Around, (0, 40), '(', 1)],
        ),
        (
            "[surround pairs{\non different]\nlines}",
            vec![
                (7, Inside, (1, 29), '[', 1),
                (15, Inside, (16, 36), '{', 1),
                (7, Around, (0, 30), '[', 1),
                (15, Around, (15, 37), '{', 1),
            ],
        ),
    ];

    for (sample, scenario) in tests {
        let doc = Rope::from(*sample);
        let slice = doc.slice(..);
        for &case in scenario {
            let (pos, objtype, expected_range, ch, count) = case;
            // NOTE: unwrapped pair_surround()
            let result = pair_surround_impl(slice, Range::point(pos), objtype, Some(ch), count);
            assert_eq!(
                result,
                expected_range.into(),
                "\nCase failed: {:?} - {:?}",
                sample,
                case
            );
        }
    }
}

// TODO: (gibbz) generalize with test_move_papagraph?
fn test_textobj_paragraph(before: &str, expected: &str, textobject: TextObject, count: usize) {
    let (s, selection) = crate::test::print(before);
    let selection = selection.transform_pure(
        paragraph,
        RuleContext {
            text: Rope::from(s.as_str()).slice(..),
            extend: false,
            direction: None,
            // See textobject_paragraph documentation
            pos_arg: count,
            syntax: None,
            ts_node: None,
            ts_textobject: Some(textobject),
            diff_handle: None,
            lang_confing: None,
            find_char: None,
            find_inclusive: None,
            syntax_find_node_fn: None,
        },
        unsafe { NonZeroUsize::new_unchecked(1) },
    );
    let actual = crate::test::plain(&s, selection);
    assert_eq!(actual, expected, "\nbefore: `{:?}`", before);
}
