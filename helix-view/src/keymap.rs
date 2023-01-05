// NOTE: Only pub becuase of their use in macros
pub mod macros;

mod keytrienode;
mod keytrie;
mod tests;

pub use {keytrie::KeyTrie, keytrienode::KeyTrieNode};

use crate::{document::Mode, command::Command};
use std::{sync::Arc, collections::HashMap};
use arc_swap::{access::{DynAccess, DynGuard}, ArcSwap};

use std::ops::Deref;

pub struct Keymap {
    pub keytries: Box<dyn DynAccess<HashMap<Mode, KeyTrie>>>,
}

pub type CommandList = HashMap<String, Vec<String>>;
impl Keymap {
    pub fn new(keytries: Box<dyn DynAccess<HashMap<Mode, KeyTrie>>>) -> Self {
        Self {
            keytries,
        }
    }

    pub fn load_keymaps(&self) -> DynGuard<HashMap<Mode, KeyTrie>> {
        self.keytries.load()
    }

    fn get_keytrie(&self, mode: &Mode) -> KeyTrie {
        // TODO: Unsure how I should handle this Option, panics.
        self.keytries.load().get(mode).unwrap().clone()
    }

    fn merge_in_default_keymap(mut self) -> Keymap {
        let mut delta = std::mem::replace(&mut self.keytries, crate::config::defualt_keymap());
        for (mode, keytrie) in &mut self {
            keytrie.merge_keytrie(delta.remove(mode).unwrap_or_default())
        }
        self
    }
    /// Returns a key-value list of all commands associated to a given Keymap.
    /// Keys are the node names (see KeyTrieNode documentation)
    /// Values are lists of stringified KeyEvents that triger the command.
    /// Each element in the KeyEvent list is prefixed with prefixed the ancestor KeyEvents. 
    /// For example: Stringified KeyEvent element for the 'goto_next_window' command could be "space>w>w".
    /// Ancestor KeyEvents are in this case "space" and "w".
    pub fn command_list(&self, mode: &Mode) -> CommandList {
        let mut list = HashMap::new();
        _command_list(&mut list, &KeyTrieNode::KeyTrie(self.get_keytrie(mode)), &mut String::new());
        return list;

        fn _command_list(list: &mut CommandList, node: &KeyTrieNode, prefix: &mut String) {
            match node {
                KeyTrieNode::KeyTrie(trie_node) => {
                    for (key_event, subtrie_node) in trie_node.deref() {
                        let mut temp_prefix: String = prefix.to_string();
                        if &temp_prefix != "" { 
                            temp_prefix.push_str(">");
                        }
                        temp_prefix.push_str(&key_event.to_string());
                        _command_list(list, subtrie_node, &mut temp_prefix);
                    }
                },
                KeyTrieNode::Commands(commands) => {
                    if commands.first().name() == "no_op" { return }
                    // FIX: when descriptions for multiple commands are added
                    if commands.len() == 1 {
                        list.entry(commands.first().name().to_string()).or_default().push(prefix.to_string());
                    }
                }
            };
        }
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::new(Box::new(ArcSwap::new(Arc::new(crate::config::defualt_keymap()))))
    }
}
