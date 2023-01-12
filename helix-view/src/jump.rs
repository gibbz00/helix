const JUMP_LIST_CAPACITY: usize = 30;

type Jump = (BufferID, Selection);

#[derive(Debug, Clone)]
pub struct JumpList {
    jumps: VecDeque<Jump>,
    current: usize,
}


impl JumpList {
    pub fn new(initial: Jump) -> Self {
        let mut jumps = VecDeque::with_capacity(JUMP_LIST_CAPACITY);
        jumps.push_back(initial);
        Self { jumps, current: 0 }
    }

    pub fn push(&mut self, jump: Jump) {
        self.jumps.truncate(self.current);
        // don't push duplicates
        if self.jumps.back() != Some(&jump) {
            // If the jumplist is full, drop the oldest item.
            while self.jumps.len() >= JUMP_LIST_CAPACITY {
                self.jumps.pop_front();
            }

            self.jumps.push_back(jump);
            self.current = self.jumps.len();
        }
    }

    pub fn forward(&mut self, count: usize) -> Option<&Jump> {
        if self.current + count < self.jumps.len() {
            self.current += count;
            self.jumps.get(self.current)
        } else {
            None
        }
    }

    // Taking buffer_view and buffer to prevent unnecessary cloning when jump is not required.
    pub fn backward(&mut self, buffer_view_id: BufferViewID, buffer: &mut BufferMirror, count: usize) -> Option<&Jump> {
        if let Some(current) = self.current.checked_sub(count) {
            if self.current == self.jumps.len() {
                let jump = (buffer.id(), buffer.selection(buffer_view_id).clone());
                self.push(jump);
            }
            self.current = current;
            self.jumps.get(self.current)
        } else {
            None
        }
    }

    pub fn remove(&mut self, buffer_id: &BufferID) {
        self.jumps.retain(|(other_id, _)| other_id != buffer_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Jump> {
        self.jumps.iter()
    }

    /// Applies a [`Transaction`] of changes to the jumplist.
    /// This is necessary to ensure that changes to buffers do not leave jump-list
    /// selections pointing to parts of the text which no longer exist.
    fn apply(&mut self, transaction: &Transaction, buffer: &Buffer) {
        let text = buffer.text().slice(..);

        for (doc_id, selection) in &mut self.jumps {
            if buffer.id() == *doc_id {
                *selection = selection
                    .clone()
                    .map(transaction.changes())
                    .ensure_invariants(text);
            }
        }
    }
}