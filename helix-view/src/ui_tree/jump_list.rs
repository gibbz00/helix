use std::collections::VecDeque;

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
        if self.jumps.back() != Some(&jump) {
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
        } else { None }
    }

    pub fn backward(&mut self, count: usize) -> Option<&Jump> {
        if let Some(current) = self.current.checked_sub(count) {
            self.current = current;
            self.jumps.get(self.current)
        } else { None }
    }

    pub fn remove(&mut self, buffer_id: &BufferID) {
        self.jumps.retain(|(other_id, _)| other_id != buffer_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Jump> {
        self.jumps.iter()
    }

    /// Otherwise jump_list selections might point to text which no longer exist.
    fn apply_transaction(&mut self, transaction: &Transaction, buffer_mirror: &BufferMirror) {
        for (buffer_id, selection) in &mut self.jumps {
            if buffer_mirror.id() == *buffer_id {
                *selection = selection
                    .clone()
                    .map(transaction.changes())
                    .ensure_invariants(buffer_mirror.text().slice(..));
            }
        }
    }
}