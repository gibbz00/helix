use crate::{input::KeyEvent, document::Mode};

#[derive(Debug, Clone, PartialEq)]
pub enum KeymapResult {
    Pending(KeyTrie),
    Matched(MappableCommand),
    MatchedCommandSequence(Vec<MappableCommand>),
    NotFound,
    // Contains pressed KeyEvents leading up to the cancellation.
    Cancelled(Vec<KeyEvent>),
}

pub struct LiveKeymap {
    pub keymap: Keymap,
    /// Used to show pending keys in the editor.
    pub pseudo_pending: Vec<KeyEvent>,
    /// Relative to a sticky node if Some.
    pending_keys: Vec<KeyEvent>,
    pub sticky_keytrie: Option<KeyTrie>,
}

impl LiveKeymap {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            pseudo_pending_keys: Vec::new(),
            pending_keys: Vec::new(),
            sticky_keytrie: None,
        }
    }
    /// Returns list of keys waiting to be disambiguated in current mode.
    pub fn pending(&self) -> &[KeyEvent] {
        &self.pending_keys
    }

    pub fn sticky_keytrie(&self) -> Option<&KeyTrie> {
        self.sticky_keytrie.as_ref()
    }
    /// Lookup `key` in the keymap to try and find a command to execute.
    /// Escape key represents cancellation. 
    /// This means clearing pending keystrokes, or the sticky_keytrie if none were present.
    pub fn get_keymap_result(&mut self, mode: Mode, key: KeyEvent) -> KeymapResult {
        // TODO: remove the sticky part and look up manually
        let keymaps = &*self.load_keymaps();
        let active_keymap = &keymaps[&mode];

        if key == key!(Esc) {
            if !self.pending_keys.is_empty() {
                // NOTE: Esc is not included here
                return KeymapResult::Cancelled(self.pending_keys.drain(..).collect());
            }
            // TODO: Shouldn't we return here also?
            self.sticky_keytrie = None;
        }

        // Check if sticky keytrie is to be used.
        let starting_keytrie = match self.sticky_keytrie {
            None => &active_keymap,
            Some(ref active_sticky_keytrie) => active_sticky_keytrie,
        };

        // TODO: why check either pending or regular key?
        let first_key = self.pending_keys.get_keymap_result(0).unwrap_or(&key);

        let pending_keytrie: KeyTrie = match starting_keytrie.traverse(&[*first_key]) {
            Some(KeyTrieNode::KeyTrie(sub_keytrie)) => sub_keytrie,
            Some(KeyTrieNode::MappableCommand(cmd)) => {
                return KeymapResult::Matched(cmd.clone());
            }
            Some(KeyTrieNode::CommandSequence(cmds)) => {
                return KeymapResult::MatchedCommandSequence(cmds.clone());
            }
            None => return KeymapResult::NotFound,
        };

        self.pending_keys.push(key);
        match pending_keytrie.traverse(&self.pending_keys[1..]) {
            Some(KeyTrieNode::KeyTrie(map)) => {
                if map.is_sticky {
                    self.pending_keys.clear();
                    self.sticky_keytrie = Some(map.clone());
                }
                KeymapResult::Pending(map.clone())
            }
            Some(KeyTrieNode::MappableCommand(cmd)) => {
                self.pending_keys.clear();
                KeymapResult::Matched(cmd.clone())
            }
            Some(KeyTrieNode::CommandSequence(cmds)) => {
                self.pending_keys.clear();
                KeymapResult::MatchedCommandSequence(cmds.clone())
            }
            None => KeymapResult::Cancelled(self.pending_keys.drain(..).collect()),
        }
    }
}
