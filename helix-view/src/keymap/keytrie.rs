use crate::{keymap::KeyTrieNode, info::Info, input::KeyEvent};
use std::{collections::HashMap, ops::{Deref, DerefMut}, cmp::Ordering};
use serde::Deserialize;

/// Edges of the trie are KeyEvents and the nodes are descrbibed by KeyTrieNode
#[derive(Debug, Clone)]
pub struct KeyTrie {
    descrpition: String,
    children: HashMap<KeyEvent, KeyTrieNode>,
    sticky: bool,
}

impl KeyTrie {
    pub fn new(description: &str, children: HashMap<KeyEvent, KeyTrieNode>) -> Self {
        Self {
            descrpition: description.to_string(),
            children,
            sticky: false,
        }
    }

    pub fn new_sticky(description: &str, children: HashMap<KeyEvent, KeyTrieNode>) -> Self {
        Self {
            sticky: true,
            ..Self::new(description, children)
        }
    }

    pub fn is_sticky(&self) -> bool {
        self.sticky
    }

    // None symbolizes NotFound
    pub fn traverse(&self, key_events: &[KeyEvent]) -> Option<KeyTrieNode> {
        return _traverse(self, key_events, 0);

        fn _traverse(keytrie: &KeyTrie, key_events: &[KeyEvent], mut depth: usize) -> Option<KeyTrieNode> {
            if depth == key_events.len() {
                return Some(KeyTrieNode::KeyTrie(keytrie.clone()));
            }
            else if let Some(found_child) = keytrie.get(&key_events[depth]) {
                match found_child {
                    KeyTrieNode::KeyTrie(sub_keytrie) => {
                        depth += 1;
                        return _traverse(sub_keytrie, key_events, depth)
                    },
                    KeyTrieNode::Commands(_) => return Some(found_child.clone())
                }
            }
            return None;
        }
    }

    pub fn merge_keytrie(&mut self, mut other_keytrie: Self) {
        for (other_key_event, other_child_node) in std::mem::take(&mut other_keytrie.children) {
            match other_child_node {
                KeyTrieNode::KeyTrie(other_child_key_trie) => {
                    if let Some(KeyTrieNode::KeyTrie(self_clashing_child_key_trie)) = self.children.get_mut(&other_key_event) {
                        self_clashing_child_key_trie.merge_keytrie(other_child_key_trie);
                    }
                    else {
                        self.children.insert(other_key_event, KeyTrieNode::KeyTrie(other_child_key_trie));
                    }
                },
                KeyTrieNode::Commands(_) => {
                    self.children.insert(other_key_event, other_child_node);
                }
            }
        }
    }

    /// Prepare infobox contents for a given KeyTrie
    /// First tuple element for infobox title
    /// Second includes Keytrie children as possible KeyEvents and thier associated descriptions.
    pub fn infobox_contents(&self) -> (&String, Vec<(String, &str)>) {
        let mut body: Vec<(Vec<String>, &str)> = Vec::with_capacity(self.len());
        for (&key_event, key_trie) in self.iter() {
            let documentation: &str = match key_trie {
                KeyTrieNode::Commands(commands) => {
                    if commands.first.name() == "no_op" {
                        continue;
                    }
                    if commands.len() == 1 { commands.first().description() }
                    // NOTE: Giving same description for all command sequences will 
                    // place them on the same row.
                    else { "[Multiple commands]" }
                },
                KeyTrieNode::KeyTrie(key_trie) => &key_trie.descrpition,
            };
            match body.iter().position(|(_, existing_documentation)| &documentation == existing_documentation) {
                Some(position) =>  body[position].0.push(key_event.to_string()),
                None => {
                    let mut temp_vec: Vec<String> = Vec::new();
                    temp_vec.push(key_event.to_string());
                    body.push((temp_vec, documentation))   
                }
            }
        }

        // Shortest keyevent (as string) appears first, unless is a "C-" KeyEvent
        // Those events will always be placed after the one letter KeyEvent
        let mut sorted_body = body.iter()
            .map(|(key_events, description)| {
                let mut temp_key_events = key_events.clone();
                temp_key_events.sort_unstable_by(|a, b| {
                    if a.len() == 1 { return Ordering::Less }
                    if b.len() > a.len() && b.starts_with("C-") {
                        return Ordering::Greater
                    }
                    a.len().cmp(&b.len())
                });
                (temp_key_events, *description)
            }).collect::<Vec<(Vec<String>, &str)>>();
        sorted_body.sort_unstable_by(|a, b| a.0[0].to_lowercase().cmp(&b.0[0].to_lowercase()));
        // Consistently place lowercase before uppercase of the same letter.
        if sorted_body.len() > 1 {
            let mut x_index = 0;
            let mut y_index = 1;

            while y_index < sorted_body.len() {
                let x = &sorted_body[x_index].0[0];
                let y = &sorted_body[y_index].0[0];
                if x.to_lowercase() == y.to_lowercase() {
                    // Uppercase regarded as lower value.
                    if x < y {
                        let temp_holder = sorted_body[x_index].clone();
                        sorted_body[x_index] = sorted_body[y_index].clone();
                        sorted_body[y_index] = temp_holder;
                    }
                }
                x_index = y_index;
                y_index += 1;
            }
        }

        let stringified_key_events_body: Vec<(String, &str)> = sorted_body.iter()
            .map(|(key_events, description)| {
                let key_events_string: String = key_events.iter().fold(String::new(), |mut acc, key_event| {
                    if !acc.is_empty() { acc.push_str(", "); }
                    acc.push_str(key_event);
                    acc
                });
                (key_events_string, *description)
            }).collect();

        (&self.descrpition, &stringified_key_events_body)
    }
}

impl Default for KeyTrie {
    fn default() -> Self {
        Self::new("", HashMap::new())
    }
}

impl PartialEq for KeyTrie {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
    }
}

/// Returns the children of the KeyTrie
impl Deref for KeyTrie {
    type Target = HashMap<KeyEvent, KeyTrieNode>;

    fn deref(&self) -> &Self::Target {
        &self.children
    }
}

/// Returns the children of the KeyTrie
impl DerefMut for KeyTrie {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.children
    }
}

impl<'de> Deserialize<'de> for KeyTrie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
         Ok(Self {
            children: HashMap::<KeyEvent, KeyTrieNode>::deserialize(deserializer)?,
            ..Default::default()
        })
    }
}
