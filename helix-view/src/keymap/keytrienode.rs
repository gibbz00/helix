use super::keytrie::KeyTrie;
use crate::{input::KeyEvent, command::{Command, Commands}};
use std::collections::HashMap;
use serde::{Deserialize, de::Visitor};

/// Commands for the added ability to map a sequence commands to one Key Event
/// Each variant includes a description property.
/// For the Commands, the property is self explanatory.
/// For KeyTrie, the description is used for respective infobox titles,
#[derive(Debug, Clone, PartialEq)]
pub enum KeyTrieNode {
    Commands(Commands),
    KeyTrie(KeyTrie),
}

impl<'de> Deserialize<'de> for KeyTrieNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(KeyTrieNodeVisitor)
    }
}

struct KeyTrieNodeVisitor;

impl<'de> Visitor<'de> for KeyTrieNodeVisitor {
    type Value = KeyTrieNode;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a KeyTrieNode")
    }

    fn visit_str<S: serde::de::Error>(self, command: &str) -> Result<Self::Value, S>
    {
        let mut commands = Vec::new();
        commands.push(command.parse::<Command>().map_err(serde::de::Error::custom)?);
        Ok(KeyTrieNode::Commands(commands))
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: serde::de::SeqAccess<'de>
    {
        let mut commands = Vec::new();
        while let Some(command) = seq.next_element::<&str>()? {
            commands.push(command.parse::<Command>().map_err(serde::de::Error::custom)?)
        }
        Ok(KeyTrieNode::Commands(commands))
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>
    {
        let mut sub_key_trie = HashMap::new();
        while let Some((key_event, key_trie_node)) = map.next_entry::<KeyEvent, KeyTrieNode>()? {
            sub_key_trie.insert(key_event, key_trie_node);
        }
        Ok(KeyTrieNode::KeyTrie(KeyTrie::new("", sub_key_trie)))
    }
}