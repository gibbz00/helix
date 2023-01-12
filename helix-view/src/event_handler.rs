use crate::{input::KeyEvent, keyboard::KeyCode, mode::Mode, keymap::{KeyTrie, KeyTrieNode}, command::MappableCommands, UITree::UITree};

pub struct EventHandler {
    ui_tree: UITree,
}

impl EventHandler {
    pub fn start(ui_tree: &UITree) -> Self {
        Self {
            ui_tree, 
        }
    }

    pub fn handle_key_event(&self, mode: Mode, key_event: KeyEvent) {
        if Some(mappable_commands) = get_keymapped_command(self, mode, keyevent) {
            for mappable_command in mappable_commands {
                    await mappable_command.execute();
            }    
            ui_tree.command_multiplier.clear()
        }
    }

    pub fn get_keymapped_command(&mut self, mode: Mode, key_event: KeyEvent) -> Option<MappableCommands> {
        let mut key_trie_path = self.pending_keys.into_flattened();
        key_trie_path.push(key_event.clone());

        // Allows keybindings to digits whilst also allowing the typing of 
        // multiple digits for commands that make use of prefixed numbers.
        let found_key_trie = match ui_tree.command_multiplier.get() {
            None => ui_tree.keymap.get_keytrie(mode).traverse(key_trie_path),
            Some(command_multiplier) => None
        };
        
        match found_key_trie { 
            None => { 
                if key_event == KeyEvent::Esc {
                    self.pending_keys.pop();
                    self.pending_keys.pop();
                } else if let Some(digit) = key_event.char().and_then(|char| char.to_digit(10)) {
                    if !(digit == 0 && ui_tree.command_multiplier.get().is_none()) {
                        ui_tree.command_multiplier.push_digit(digit);
                    }
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
                return None
            }
        }
    }
}
