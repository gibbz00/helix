pub mod diff_provider;
mod line_cache;
mod worker;

use std::ops::Range;

use imara_diff::intern::InternedInput;
use imara_diff::Algorithm;
use ropey::RopeSlice;

use crate::{ChangeSet, Rope, Tendril, Transaction};
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::{Notify, OwnedRwLockReadGuard, RwLock};
use tokio::task::JoinHandle;

use worker::DiffWorker;

/// A `imara_diff::Sink` that builds a `ChangeSet` for a character diff of a hunk
struct CharChangeSetBuilder<'a> {
    res: &'a mut ChangeSet,
    hunk: &'a InternedInput<char>,
    pos: u32,
}

impl imara_diff::Sink for CharChangeSetBuilder<'_> {
    type Out = ();
    fn process_change(&mut self, before: Range<u32>, after: Range<u32>) {
        self.res.retain((before.start - self.pos) as usize);
        self.res.delete(before.len());
        self.pos = before.end;

        let res = self.hunk.after[after.start as usize..after.end as usize]
            .iter()
            .map(|&token| self.hunk.interner[token])
            .collect();

        self.res.insert(res);
    }

    fn finish(self) -> Self::Out {
        self.res.retain(self.hunk.before.len() - self.pos as usize);
    }
}

struct LineChangeSetBuilder<'a> {
    res: ChangeSet,
    after: RopeSlice<'a>,
    file: &'a InternedInput<RopeSlice<'a>>,
    current_hunk: InternedInput<char>,
    pos: u32,
}

impl imara_diff::Sink for LineChangeSetBuilder<'_> {
    type Out = ChangeSet;

    fn process_change(&mut self, before: Range<u32>, after: Range<u32>) {
        let len = self.file.before[self.pos as usize..before.start as usize]
            .iter()
            .map(|&it| self.file.interner[it].len_chars())
            .sum();
        self.res.retain(len);
        self.pos = before.end;

        // do not perform diffs on large hunks
        let len_before = before.end - before.start;
        let len_after = after.end - after.start;

        // Pure insertions/removals do not require a character diff.
        // Very large changes are ignored because their character diff is expensive to compute
        // TODO adjust heuristic to detect large changes?
        if len_before == 0
            || len_after == 0
            || len_after > 5 * len_before
            || 5 * len_after < len_before && len_before > 10
            || len_before + len_after > 200
        {
            let remove = self.file.before[before.start as usize..before.end as usize]
                .iter()
                .map(|&it| self.file.interner[it].len_chars())
                .sum();
            self.res.delete(remove);
            let mut fragment = Tendril::new();
            if len_after > 500 {
                // copying a rope line by line is slower then copying the entire
                // rope. Use to_string for very large changes instead..
                if self.file.after.len() == after.end as usize {
                    if after.start == 0 {
                        fragment = self.after.to_string().into();
                    } else {
                        let start = self.after.line_to_char(after.start as usize);
                        fragment = self.after.slice(start..).to_string().into();
                    }
                } else if after.start == 0 {
                    let end = self.after.line_to_char(after.end as usize);
                    fragment = self.after.slice(..end).to_string().into();
                } else {
                    let start = self.after.line_to_char(after.start as usize);
                    let end = self.after.line_to_char(after.end as usize);
                    fragment = self.after.slice(start..end).to_string().into();
                }
            } else {
                for &line in &self.file.after[after.start as usize..after.end as usize] {
                    for chunk in self.file.interner[line].chunks() {
                        fragment.push_str(chunk)
                    }
                }
            };
            self.res.insert(fragment);
        } else {
            // for reasonably small hunks, generating a ChangeSet from char diff can save memory
            // TODO use a tokenizer (word diff?) for improved performance
            let hunk_before = self.file.before[before.start as usize..before.end as usize]
                .iter()
                .flat_map(|&it| self.file.interner[it].chars());
            let hunk_after = self.file.after[after.start as usize..after.end as usize]
                .iter()
                .flat_map(|&it| self.file.interner[it].chars());
            self.current_hunk.update_before(hunk_before);
            self.current_hunk.update_after(hunk_after);

            // the histogram heuristic does not work as well
            // for characters because the same characters often reoccur
            // use myer diff instead
            imara_diff::diff(
                Algorithm::Myers,
                &self.current_hunk,
                CharChangeSetBuilder {
                    res: &mut self.res,
                    hunk: &self.current_hunk,
                    pos: 0,
                },
            );

            self.current_hunk.clear();
        }
    }

    fn finish(mut self) -> Self::Out {
        let len = self.file.before[self.pos as usize..]
            .iter()
            .map(|&it| self.file.interner[it].len_chars())
            .sum();

        self.res.retain(len);
        self.res
    }
}

struct RopeLines<'a>(RopeSlice<'a>);

impl<'a> imara_diff::intern::TokenSource for RopeLines<'a> {
    type Token = RopeSlice<'a>;
    type Tokenizer = ropey::iter::Lines<'a>;

    fn tokenize(&self) -> Self::Tokenizer {
        self.0.lines()
    }

    fn estimate_tokens(&self) -> u32 {
        // we can provide a perfect estimate which is very nice for performance
        self.0.len_lines() as u32
    }
}

/// Compares `old` and `new` to generate a [`Transaction`] describing
/// the steps required to get from `old` to `new`.
pub fn compare_ropes(before: &Rope, after: &Rope) -> Transaction {
    let start = std::time::Instant::now();
    let res = ChangeSet::with_capacity(32);
    let after = after.slice(..);
    let file = InternedInput::new(RopeLines(before.slice(..)), RopeLines(after));
    let builder = LineChangeSetBuilder {
        res,
        file: &file,
        after,
        pos: 0,
        current_hunk: InternedInput::default(),
    };

    let res = imara_diff::diff(Algorithm::Histogram, &file, builder).into();

    log::debug!(
        "rope diff took {}s",
        std::time::Instant::now()
            .duration_since(start)
            .as_secs_f64()
    );
    res
}

type RedrawHandle = (Arc<Notify>, Arc<RwLock<()>>);

/// A rendering lock passed to the differ the prevents redraws from occurring
struct RenderLock {
    pub lock: OwnedRwLockReadGuard<()>,
    pub timeout: Option<tokio::time::Instant>,
}

struct Event {
    text: Rope,
    is_base: bool,
    render_lock: Option<RenderLock>,
}

#[derive(Clone, Debug)]
pub struct DiffHandle {
    channel: UnboundedSender<Event>,
    render_lock: Arc<RwLock<()>>,
    hunks: Arc<Mutex<Vec<Hunk>>>,
    inverted: bool,
}

impl DiffHandle {
    pub fn new(diff_base: Rope, doc: Rope, redraw_handle: RedrawHandle) -> DiffHandle {
        DiffHandle::new_with_handle(diff_base, doc, redraw_handle).0
    }

    fn new_with_handle(
        diff_base: Rope,
        doc: Rope,
        redraw_handle: RedrawHandle,
    ) -> (DiffHandle, JoinHandle<()>) {
        let (sender, receiver) = unbounded_channel();
        let hunks: Arc<Mutex<Vec<Hunk>>> = Arc::default();
        let worker = DiffWorker {
            channel: receiver,
            hunks: hunks.clone(),
            new_hunks: Vec::default(),
            redraw_notify: redraw_handle.0,
            diff_finished_notify: Arc::default(),
        };
        let handle = tokio::spawn(worker.run(diff_base, doc));
        let differ = DiffHandle {
            channel: sender,
            hunks,
            inverted: false,
            render_lock: redraw_handle.1,
        };
        (differ, handle)
    }

    pub fn invert(&mut self) {
        self.inverted = !self.inverted;
    }

    pub fn hunks(&self) -> FileHunks {
        FileHunks {
            hunks: self.hunks.lock(),
            inverted: self.inverted,
        }
    }

    /// Updates the document associated with this redraw handle
    /// This function is only intended to be called from within the rendering loop
    /// if called from elsewhere it may fail to acquire the render lock and panic
    pub fn update_document(&self, doc: Rope, block: bool) -> bool {
        // unwrap is ok here because the rendering lock is
        // only exclusively locked during redraw.
        // This function is only intended to be called
        // from the core rendering loop where no redraw can happen in parallel
        let lock = self.render_lock.clone().try_read_owned().unwrap();
        let timeout = if block {
            None
        } else {
            Some(
                tokio::time::Instant::now() + tokio::time::Duration::from_millis(SYNC_DIFF_TIMEOUT),
            )
        };
        self.update_document_impl(doc, self.inverted, Some(RenderLock { lock, timeout }))
    }

    pub fn update_diff_base(&self, diff_base: Rope) -> bool {
        self.update_document_impl(diff_base, !self.inverted, None)
    }

    fn update_document_impl(
        &self,
        text: Rope,
        is_base: bool,
        render_lock: Option<RenderLock>,
    ) -> bool {
        let event = Event {
            text,
            is_base,
            render_lock,
        };
        self.channel.send(event).is_ok()
    }
}

/// synchronous debounce value should be low
/// so we can update synchronously most of the time
const DIFF_DEBOUNCE_TIME_SYNC: u64 = 1;
/// maximum time that rendering should be blocked until the diff finishes
const SYNC_DIFF_TIMEOUT: u64 = 12;
const DIFF_DEBOUNCE_TIME_ASYNC: u64 = 96;
const ALGORITHM: Algorithm = Algorithm::Histogram;
const MAX_DIFF_LINES: usize = 64 * u16::MAX as usize;
// cap average line length to 128 for files with MAX_DIFF_LINES
const MAX_DIFF_BYTES: usize = MAX_DIFF_LINES * 128;

/// A single change in a file potentially spanning multiple lines
/// Hunks produced by the differs are always ordered by their position
/// in the file and non-overlapping.
/// Specifically for any two hunks `x` and `y` the following properties hold:
///
/// ``` no_compile
/// assert!(x.before.end <= y.before.start);
/// assert!(x.after.end <= y.after.start);
/// ```
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Hunk {
    pub before: Range<u32>,
    pub after: Range<u32>,
}

impl Hunk {
    /// Can be used instead of `Option::None` for better performance
    /// because lines larger then `i32::MAX` are not supported by `imara-diff` anyways.
    /// Has some nice properties where it usually is not necessary to check for `None` separately:
    /// Empty ranges fail contains checks and also fails smaller then checks.
    pub const NONE: Hunk = Hunk {
        before: u32::MAX..u32::MAX,
        after: u32::MAX..u32::MAX,
    };

    /// Inverts a change so that `before`
    pub fn invert(&self) -> Hunk {
        Hunk {
            before: self.after.clone(),
            after: self.before.clone(),
        }
    }

    pub fn is_pure_insertion(&self) -> bool {
        self.before.is_empty()
    }

    pub fn is_pure_removal(&self) -> bool {
        self.after.is_empty()
    }
}

/// A list of changes in a file sorted in ascending
/// non-overlapping order
#[derive(Debug)]
pub struct FileHunks<'a> {
    hunks: MutexGuard<'a, Vec<Hunk>>,
    inverted: bool,
}

impl FileHunks<'_> {
    pub fn is_inverted(&self) -> bool {
        self.inverted
    }

    /// Returns the `Hunk` for the `n`th change in this file.
    /// if there is no `n`th change  `Hunk::NONE` is returned instead.
    pub fn nth_hunk(&self, n: u32) -> Hunk {
        match self.hunks.get(n as usize) {
            Some(hunk) if self.inverted => hunk.invert(),
            Some(hunk) => hunk.clone(),
            None => Hunk::NONE,
        }
    }

    pub fn len(&self) -> u32 {
        self.hunks.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn next_hunk(&self, line: u32) -> Option<Hunk> {
        let hunk_range = if self.inverted {
            |hunk: &Hunk| hunk.before.clone()
        } else {
            |hunk: &Hunk| hunk.after.clone()
        };

        let res = self
            .hunks
            .binary_search_by_key(&line, |hunk| hunk_range(hunk).start);

        let index = match res {
            // Search found a hunk that starts exactly at this line, return the next hunk if it exists.
            Ok(pos) if pos + 1 == self.hunks.len() => return None,
            Ok(pos) => pos as u32,

            // No hunk starts exactly at this line, so the search returns
            // the position where a hunk starting at this line should be inserted.
            // That position is exactly the position of the next hunk or the end
            // of the list if no such hunk exists
            Err(pos) if pos == self.hunks.len() => return None,
            Err(pos) => pos as u32 - 1,
        };

        Some(self.nth_hunk(index))
    }

    pub fn prev_hunk(&self, line: u32) -> Option<Hunk> {
        let hunk_range = if self.inverted {
            |hunk: &Hunk| hunk.before.clone()
        } else {
            |hunk: &Hunk| hunk.after.clone()
        };
        let res = self
            .hunks
            .binary_search_by_key(&line, |hunk| hunk_range(hunk).end);

        let index = match res {
            // Search found a hunk that ends exactly at this line (so it does not include the current line).
            // We can usually just return that hunk, however a special case for empty hunk is necessary
            // which represents a pure removal.
            // Removals are technically empty but are still shown as single line hunks
            // and as such we must jump to the previous hunk (if it exists) if we are already inside the removal
            Ok(pos) if !hunk_range(&self.hunks[pos]).is_empty() => pos as u32,

            // No hunk ends exactly at this line, so the search returns
            // the position where a hunk ending at this line should be inserted.
            // That position before this one is exactly the position of the previous hunk
            Err(0) | Ok(0) => return None,
            Err(pos) | Ok(pos) => pos as u32 - 1,
        };
        Some(self.nth_hunk(index))
    }

    pub fn hunk_at(&self, line: u32, include_removal: bool) -> Option<u32> {
        let hunk_range = if self.inverted {
            |hunk: &Hunk| hunk.before.clone()
        } else {
            |hunk: &Hunk| hunk.after.clone()
        };

        let res = self
            .hunks
            .binary_search_by_key(&line, |hunk| hunk_range(hunk).start);

        match res {
            // Search found a hunk that starts exactly at this line, return it
            Ok(pos) => Some(pos as u32),

            // No hunk starts exactly at this line, so the search returns
            // the position where a hunk starting at this line should be inserted.
            // The previous hunk contains this hunk if it exists and doesn't end before this line
            Err(0) => None,
            Err(pos) => {
                let hunk = hunk_range(&self.hunks[pos - 1]);
                if hunk.end > line || include_removal && hunk.start == line && hunk.is_empty() {
                    Some(pos as u32 - 1)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_identity(a: &str, b: &str) {
        let mut old = Rope::from(a);
        let new = Rope::from(b);
        compare_ropes(&old, &new).apply(&mut old);
        assert_eq!(old, new);
    }

    quickcheck::quickcheck! {
        fn test_compare_ropes(a: String, b: String) -> bool {
            let mut old = Rope::from(a);
            let new = Rope::from(b);
            compare_ropes(&old, &new).apply(&mut old);
            old == new
        }
    }

    #[test]
    fn equal_files() {
        test_identity("foo", "foo");
    }

    #[test]
    fn trailing_newline() {
        test_identity("foo\n", "foo");
        test_identity("foo", "foo\n");
    }

    #[test]
    fn new_file() {
        test_identity("", "foo");
    }

    #[test]
    fn deleted_file() {
        test_identity("foo", "");
    }
}
