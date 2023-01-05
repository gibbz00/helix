use crate::{input::KeyEvent, keyboard::KeyCode, document::Mode, keymap::{KeyTrie, KeyTrieNode, Keymap}, command::Commands};

pub struct EventHandler {
    pub keymap: Keymap,
    // New vec is created when traversing to a sticky keytrie
    // Each Vec in Vec is in other words a sticky level.
    pending_keys: Vec<Vec<KeyEvent>>,
}

impl EventHandler {
    pub fn start(keymap: Keymap) -> Self {
        Self {
            keymap,
            pending_keys: vec![Vec::new()]
        }
    }
    // TODO: UI tree subscribes to pending key
    pub fn pending_keys(&self) -> &[KeyEvent] {
        &self.pending_keys
    }

    pub fn handle_key_event(&mut self, mode: Mode, key_event: KeyEvent) -> Option<Commands> {
        let mut key_trie_path = self.pending_keys.into_flattened();
        key_trie_path.push(key_event.clone());
        let found_key_trie = self.get_keytrie(mode).traverse(key_trie_path);
        
        match found_key_trie { 
            None => { 
                if key_event == KeyEvent::Esc {
                    self.pending_keys.pop();
                } else {
                    *self.pending_keys.last_mut() = Vec::new();
                }
                return None
            },
            KeyTrieNode::Commands(commands) => {
                *self.pending_keys.last_mut() = Vec::new();
                return Some(commands)
            },
            KeyTrieNode::KeyTrie(keytrie) => {
                self.pending_keys.last_mut().push(key_event);
                if keytrie.is_sticky() {
                    self.pending_keys.push(Vec::new());
                }
            }
        }
    }
}
