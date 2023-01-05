#[macro_export]
macro_rules! key {
    ($key:ident) => {
        $crate::input::KeyEvent {
            code: $crate::keyboard::KeyCode::$key,
            modifiers: $crate::keyboard::KeyModifiers::NONE,
        }
    };
    ($($ch:tt)*) => {
        $crate::input::KeyEvent {
            code: $crate::keyboard::KeyCode::Char($($ch)*),
            modifiers: $crate::keyboard::KeyModifiers::NONE,
        }
    };
}

#[macro_export]
macro_rules! shift {
    ($key:ident) => {
        $crate::input::KeyEvent {
            code: $crate::keyboard::KeyCode::$key,
            modifiers: $crate::keyboard::KeyModifiers::SHIFT,
        }
    };
    ($($ch:tt)*) => {
        $crate::input::KeyEvent {
            code: $crate::keyboard::KeyCode::Char($($ch)*),
            modifiers: $crate::keyboard::KeyModifiers::SHIFT,
        }
    };
}

#[macro_export]
macro_rules! ctrl {
    ($key:ident) => {
        $crate::input::KeyEvent {
            code: $crate::keyboard::KeyCode::$key,
            modifiers: $crate::keyboard::KeyModifiers::CONTROL,
        }
    };
    ($($ch:tt)*) => {
        $crate::input::KeyEvent {
            code: $crate::keyboard::KeyCode::Char($($ch)*),
            modifiers: $crate::keyboard::KeyModifiers::CONTROL,
        }
    };
}

#[macro_export]
macro_rules! alt {
    ($key:ident) => {
        $crate::input::KeyEvent {
            code: $crate::keyboard::KeyCode::$key,
            modifiers: $crate::keyboard::KeyModifiers::ALT,
        }
    };
    ($($ch:tt)*) => {
        $crate::input::KeyEvent {
            code: $crate::keyboard::KeyCode::Char($($ch)*),
            modifiers: $crate::keyboard::KeyModifiers::ALT,
        }
    };
}

/// Macro for defining the root of a `KeyTrie` object. Example:
///
/// ```
/// # use helix_core::hashmap;
/// # use helix_view::config::keymap::{keytrie::KeyTrie, macros::keytrie};
/// let normal_mode = keytrie!({ "Normal mode"
///     "i" => insert_mode,
///     "g" => { "Goto"
///         "g" => goto_file_start,
///         "e" => goto_file_end,
///     },
///     "j" | "down" => move_line_down,
/// });
/// let keymap = normal_mode;
/// ```
#[macro_export]
macro_rules! keytrie {
    ({ $label:literal $(sticky=$sticky:literal)? $($($key:literal)|+ => $value:tt,)+ }) => {
        // modified from the hashmap! macro
        {
            let _cap = hashmap!(@count $($($key),+),*);
            let mut _map: std::collections::HashMap<$crate::input::KeyEvent, $crate::keymap::KeyTrieNode> = 
                std::collections::HashMap::with_capacity(_cap);
            $(
                $(
                    let _key = $key.parse::<$crate::input::KeyEvent>().unwrap();
                    let _potential_duplicate = _map.insert(_key,keytrie!(@trie $value));
                    assert!(_potential_duplicate.is_none(), "Duplicate key found: {:?}", _potential_duplicate.unwrap());
                )+
            )*
            let mut _node = $crate::keymap::KeyTrie::new($label, _map);
            $( _node.is_sticky = $sticky; )?
            _node
        }
    };

    (@trie {$label:literal $(sticky=$sticky:literal)? $($($key:literal)|+ => $value:tt,)+ }) => {
        $crate::keymap::KeyTrieNode::KeyTrie(keytrie!({ $label $(sticky=$sticky)? $($($key)|+ => $value,)+ }))
    };

    (@trie $cmd:ident) => {
        $crate::keymap::KeyTrieNode::Commands($crate::command::Command::$cmd)
    };

    (@trie [$($cmd:ident),* $(,)?]) => {
        $crate::keymap::KeyTrieNode::Commands(vec![$($crate::command:Command::$cmd),*])
    };
}

pub use key;
pub use shift;
pub use ctrl;
pub use alt;
pub use keytrie;
