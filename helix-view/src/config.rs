mod keymap;
mod view;

pub use keymap::default as default_keymap;

use {keymap::default, view::ViewConfig};
use crate::{buffer::Mode, keymap::Keymap};
use serde::Deserialize;
use std::io::Error;
use toml::de::Error as TomlError;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub keys: Keymap,
    pub view: ViewConfig,
}

impl Config {
    /// Attemps to deserialize config.toml, merging it with default keymap, theme, and editor config.
    pub fn merged() -> Result<Config, Error> {
        let config_string = std::fs::read_to_string(helix_loader::config_file())?;
        // TODO: why is only the keymap explicitly merged? and not the theme and editor options?
        toml::from_str(&config_string)?.map(|config| config.keys = config.keys.merge_in_default_keymap())?
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            keys: Keymap::default(),
            view: ViewConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        commands::MappableCommand,
        config::Config,
        keymap::{
            default,
            keytrienode::KeyTrieNode,
            macros::*,
        },
    };
    use helix_core::hashmap;
    use helix_view::document::Mode;

    #[test]
    fn parses_keymap_from_toml() {
        let sample_keymaps = r#"
            [keys.insert]
            y = "move_line_down"
            S-C-a = "delete_selection"

            [keys.normal]
            A-F12 = "move_next_word_end"
        "#;

        assert_eq!(
            toml::from_str::<Config>(sample_keymaps).unwrap(),
            Config {
                keys: hashmap! {
                    Mode::Insert => keytrie!({ "Insert mode"
                        "y" => move_line_down,
                        "S-C-a" => delete_selection,
                    }),
                    Mode::Normal => keytrie!({ "Normal mode"
                        "A-F12" => move_next_word_end,
                    }),
                },
                ..Default::default()
            }
        );
    }

    #[test]
    fn keys_resolve_to_correct_defaults() {
        // From serde default
        let default_keys = toml::from_str::<Config>("").unwrap().keys;
        assert_eq!(default_keys, default::default());

        // From the Default trait
        let default_keys = Config::default().keys;
        assert_eq!(default_keys, default::default());
    }

    #[test]
    fn user_config_merges_with_default() {
        let user_config = Config {
            keys: hashmap! {
                Mode::Normal => keytrie!({ "Normal mode"
                        "i" => normal_mode,
                        "无" => insert_mode,
                        "z" => jump_backward,
                        "g" => { "Merge into goto mode"
                            "$" => goto_line_end,
                            "g" => delete_char_forward,
                        },
                    })
                
            },
            ..Default::default()
        };
        let mut merged_config = user_config.clone().merge_in_default_keymap();
        assert_ne!(
            user_config,
            merged_config,
            "Merged user keymap with default should differ from user keymap."
        );

        let keymap_normal_root_key_trie = &merged_config.keys.get_mut(&Mode::Normal).unwrap();
        assert_eq!(
            keymap_normal_root_key_trie.traverse(&[key!('i')]).unwrap(),
            KeyTrieNode::MappableCommand(MappableCommand::normal_mode),
            "User supplied mappable command should ovveride default mappable command bound to the same key event."
        );
        assert_eq!(
            keymap_normal_root_key_trie.traverse(&[key!('无')]).unwrap(),
            KeyTrieNode::MappableCommand(MappableCommand::insert_mode),
            "User supplied mappable command of new key event should be present in merged keymap."
        );
        // Assumes that z is a node in the default keymap
        assert_eq!(
            keymap_normal_root_key_trie.traverse(&[key!('z')]).unwrap(),
            KeyTrieNode::MappableCommand(MappableCommand::jump_backward),
            "User supplied mappable command should replace a sub keytrie from default keymap bound to the same key event."
        );
        // Assumes that `g` is a sub key trie in default keymap
        assert_eq!(
            keymap_normal_root_key_trie.traverse(&[key!('g'), key!('$')]).unwrap(),
            KeyTrieNode::MappableCommand(MappableCommand::goto_line_end),
            "User supplied mappable command should be inserted under the correct sub keytrie."
        );
        // Assumes that `gg` is in default keymap
        assert_eq!(
            keymap_normal_root_key_trie.traverse(&[key!('g'), key!('g')]).unwrap(),
            KeyTrieNode::MappableCommand(MappableCommand::delete_char_forward),
            "User supplied mappable command should replace default even in sub keytries."
        );
        // Assumes that `ge` is in default keymap
        assert_eq!(
            keymap_normal_root_key_trie.traverse(&[key!('g'), key!('e')]).unwrap(),
            KeyTrieNode::MappableCommand(MappableCommand::goto_last_line),
            "Default mappable commands that aren't ovveridden should exist in merged keymap."
        );

        // Huh?
        assert!(merged_config.keys.get_keytrie(&Mode::Normal).unwrap().len() > 1);
        assert!(merged_config.keys.get_keytrie(&Mode::Insert).unwrap().len() > 0);
    }
}
