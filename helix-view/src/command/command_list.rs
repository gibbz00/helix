use super::{Command, CommandArgument, client::*};

/// Access elements in COMMAND_LIST by aliases too.
pub static COMMAND_MAP: Lazy<HashMap<&'static str, &'static Command>> =
    Lazy::new(|| COMMAND_LIST.iter()
        .flat_map(|command| {
            std::iter::once((command.name, command))
                .chain(command.aliases.iter().map(move |&alias| (alias, command)))
        }).collect()
    );

pub const COMMAND_LIST: &'static[Command] = &[
    // ## CLIENT
    // Config
    Command {
        name: "tree-sitter-subtree",
        aliases: &["ts-subtree"],
        description: "Display tree sitter subtree under cursor, primarily for debugging queries.",
        args: &[],
        function: client::
    },
    Command {
        name: "config-reload",
        aliases: &[],
        description: "Refresh user config.",
        args: &[],
        function: client::
    },
    Command {
        name: "config-open",
        aliases: &[],
        description: "Open the user config.toml file.",
        args: &[],
        function: client::
    },
    Command {
        name: "theme",
        aliases: &[],
        description: "Change the editor theme (show current theme if no name specified).",
        args: &[Optional(Theme)],
        function: client::
    },
    Command {
        name: "set-option",
        aliases: &["set"],
        description: "Set a config option at runtime.\n To disable smart case search for example; `:set search.smart-case false`.",
        args: &[Required(ConfigOptions)],
        function: client::
    },
    Command {
        name: "get-option",
        aliases: &["get"],
        description: "Get the current value of a config option.",
        args: &[Required(ConfigOptions)],
        function: client::
    },
    Command {
        name: "show-clipboard-provider",
        aliases: &[],
        description: "Show clipboard provider name in status bar.",
        args: &[],
        function: client::
    },
    Command {
            name: "no_op",
            aliases: &[],
            description: "Do nothing",
            args: &[]
        function: client::
    },
    // Selection movement
        Command {
                name: "repeat_last_motion",
                aliases: &[],
                description: "Repeat last motion",
                args: &[]
        function: client::
        },
    Command {
        name: "goto",
        aliases: &["g"],
        description: "Goto line number.",
        args: &[],
        function: client::
    },
        Command {
            name: "goto_line_start",
            aliases: &[],
            description: "Goto line start",
            args: &[]
        function: client::
    },
        Command {
            name: "extend_to_line_start",
            aliases: &[],
            description: "Extend to line start",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_line_end",
            aliases: &[],
            description: "Goto line end",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_to_line_end",
            aliases: &[],
            description: "Extend to line end",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_line_end_newline",
            aliases: &[],
            description: "Goto newline at line end",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_to_line_end_newline",
            aliases: &[],
            description: "Extend to line end",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_line",
            aliases: &[],
            description: "Select current line, if already selected, extend to another line based on the anchor",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_line_below",
            aliases: &[],
            description: "Select current line, if already selected, extend to next line",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_line_above",
            aliases: &[],
            description: "Select current line, if already selected, extend to previous line",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_to_line_bounds",
            aliases: &[],
            description: "Extend selection to line bounds",
            args: &[]
        function: client::
        },
        Command {
            name: "move_char_left",
            aliases: &[],
            description: "Move left",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_char_left",
            aliases: &[],
            description: "Extend left",
            args: &[]
        function: client::
        },
        Command {
            name: "move_char_right",
            aliases: &[],
            description: "Move right",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_char_right",
            aliases: &[],
            description: "Extend right",
            args: &[]
        function: client::
        },
        Command {
            name: "move_line_up",
            aliases: &[],
            description: "Move up",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_line_up",
            aliases: &[],
            description: "Extend up",
            args: &[]
        function: client::
        },
        Command {
            name: "move_line_down",
            aliases: &[],
            description: "Move down",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_line_down",
            aliases: &[],
            description: "Extend down",
            args: &[]
        function: client::
        },
        Command {
            name: "move_next_word_start",
            aliases: &[],
            description: "Move to start of next word",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_next_word_start",
            aliases: &[],
            description: "Extend to start of next word",
            args: &[]
        function: client::
        },
        Command {
            name: "move_prev_word_start",
            aliases: &[],
            description: "Move to start of previous word",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_prev_word_start",
            aliases: &[],
            description: "Extend to start of previous word",
            args: &[]
        function: client::
        },
        Command {
            name: "move_next_word_end",
            aliases: &[],
            description: "Move to end of next word",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_next_word_end",
            aliases: &[],
            description: "Extend to end of next word",
            args: &[]
        function: client::
        },
        Command {
            name: "move_prev_word_end",
            aliases: &[],
            description: "Move to end of previous word",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_prev_word_end",
            aliases: &[],
            description: "Extend to end of previous word",
            args: &[]
        function: client::
        },
        Command {
            name: "move_next_long_word_start",
            aliases: &[],
            description: "Move to start of next long word",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_next_long_word_start",
            aliases: &[],
            description: "Extend to start of next long word",
            args: &[]
        function: client::
        },
        Command {
            name: "move_prev_long_word_start",
            aliases: &[],
            description: "Move to start of previous long word",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_prev_long_word_start",
            aliases: &[],
            description: "Extend to start of previous long word",
            args: &[]
        function: client::
        },
        Command {
            name: "move_next_long_word_end",
            aliases: &[],
            description: "Move to end of next long word",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_next_long_word_end",
            aliases: &[],
            description: "Extend to end of next long word",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_first_nonwhitespace",
            aliases: &[],
            description: "Goto first non-blank in line",
            args: &[]
        function: client::
        },
        Command {
            name: "find_till_char",
            aliases: &[],
            description: "Move till next occurrence of char",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_till_char",
            aliases: &[],
            description: "Extend till next occurrence of char",
            args: &[]
        function: client::
        },      
        Command {
            name: "find_next_char",
            aliases: &[],
            description: "Move to next occurrence of char",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_next_char",
            aliases: &[],
            description: "Extend to next occurrence of char",
            args: &[]
        function: client::
        },      
        Command {
            name: "till_prev_char",
            aliases: &[],
            description: "Move till previous occurrence of char",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_till_prev_char",
            aliases: &[],
            description: "Extend till previous occurrence of char",
            args: &[]
        function: client::
        },      
        Command {
            name: "find_prev_char",
            aliases: &[],
            description: "Move to previous occurrence of char",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_prev_char",
            aliases: &[],
            description: "Extend to previous occurrence of char",
            args: &[]
        function: client::
        },      
        Command {
            name: "copy_selection_on_next_line",
            aliases: &[],
            description: "Copy selection on next line",
            args: &[]
        function: client::
        },
        Command {
            name: "copy_selection_on_prev_line",
            aliases: &[],
            description: "Copy selection on previous line",
            args: &[]
        function: client::
        },
        Command {
            name: "select_all",
            aliases: &[],
            description: "Select whole document",
            args: &[]
        function: client::
        },
        Command {
            name: "select_regex",
            aliases: &[],
            description: "Select all regex matches inside selections",
            args: &[]
        function: client::
        },
        Command {
            name: "keep_selections",
            aliases: &[],
            description: "Keep selections matching regex",
            args: &[]
        function: client::
        },
        Command {
            name: "remove_selections",
            aliases: &[],
            description: "Remove selections matching regex",
            args: &[]
        function: client::
        },
        Command {
            name: "split_selection",
            aliases: &[],
            description: "Split selections on regex matches",
            args: &[]
        function: client::
        },
        Command {
            name: "split_selection_on_newline",
            aliases: &[],
            description: "Split selection on newlines",
            args: &[]
        function: client::
        },
        Command {
            name: "shrink_to_line_bounds",
            aliases: &[],
            description: "Shrink selection to line bounds",
            args: &[]
        function: client::
        },
        Command {
            name: "collapse_selection",
            aliases: &[],
            description: "Collapse selection into single cursor",
            args: &[]
        function: client::
        },
        Command {
            name: "flip_selections",
            aliases: &[],
            description: "Flip selection cursor and anchor",
            args: &[]
        function: client::
        },
        Command {
            name: "ensure_selections_forward",
            aliases: &[],
            description: "Ensure all selections face forward",
            args: &[]
        function: client::
        },
        Command {
            name: "keep_primary_selection",
            aliases: &[],
            description: "Keep primary selection",
            args: &[]
        function: client::
        },
        Command {
            name: "remove_primary_selection",
            aliases: &[],
            description: "Remove primary selection",
            args: &[]
        function: client::
        },
        Command {
            name: "rotate_selections_forward",
            aliases: &[],
            description: "Rotate selections forward",
            args: &[]
        function: client::
        },
        Command {
            name: "rotate_selections_backward",
            aliases: &[],
            description: "Rotate selections backward",
            args: &[]
        function: client::
        },
        // Remove duplicates
        Command {
            name: "yank",
            aliases: &[],
            description: "Yank selection",
            args: &[]
        function: client::
        },
        Command {
            name: "yank_joined_to_clipboard",
            aliases: &[],
            description: "Join and yank selections to clipboard",
            args: &[]
        function: client::
        },
        Command {
            name: "yank_main_selection_to_clipboard",
            aliases: &[],
            description: "Yank main selection to clipboard",
            args: &[]
        function: client::
        },
        Command {
            name: "yank_joined_to_primary_clipboard",
            aliases: &[],
            description: "Join and yank selections to primary clipboard",
            args: &[]
        function: client::
        },
        Command {
            name: "yank_main_selection_to_primary_clipboard",
            aliases: &[],
            description: "Yank main selection to primary clipboard",
            args: &[]
        function: client::
        },
        Command {
            name: "clipboard-yank",
            aliases: &[],
            description: "Yank main selection into system clipboard.",
            args: &[],
            function: client::
        },
        Command {
            name: "clipboard-yank-join",
            aliases: &[],
            description: "Yank joined selections into system clipboard. A separator can be provided as first argument. Default value is newline.", // FIXME: current UI can't display long doc.
            args: &[],
            function: client::
        },
        Command {
            name: "primary-clipboard-yank",
            aliases: &[],
            description: "Yank main selection into system primary clipboard.",
            args: &[],
            function: client::
        },
        Command {
            name: "primary-clipboard-yank-join",
            aliases: &[],
            description: "Yank joined selections into system primary clipboard. A separator can be provided as first argument. Default value is newline.", // FIXME: current UI can't display long doc.
            args: &[],
            function: client::
        },
        // Treesitter objects
        Command {
            name: "tree-sitter-scopes",
            aliases: &[],
            description: "Display tree sitter scopes, primarily for theming and development.",
            args: &[],
            function: client::
       },
        Command {
            name: "match_brackets",
            aliases: &[],
            description: "Goto matching bracket",
            args: &[]
        function: client::
        },
        Command {
            name: "select_textobject_around",
            aliases: &[],
            description: "Select around object",
            args: &[]
        function: client::
        },
        Command {
            name: "select_textobject_inner",
            aliases: &[],
            description: "Select inside object",
            args: &[]
        function: client::
        },
        Command {
            name: "expand_selection",
            aliases: &[],
            description: "Expand selection to parent syntax node",
            args: &[]
        function: client::
        },
        Command {
            name: "shrink_selection",
            aliases: &[],
            description: "Shrink selection to previously expanded syntax node",
            args: &[]
        function: client::
        },
        Command {
            name: "select_next_sibling",
            aliases: &[],
            description: "Select next sibling in syntax tree",
            args: &[]
        function: client::
        },
        Command {
            name: "select_prev_sibling",
            aliases: &[],
            description: "Select previous sibling in syntax tree",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_next_function",
            aliases: &[],
            description: "Goto next function",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_prev_function",
            aliases: &[],
            description: "Goto previous function",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_next_class",
            aliases: &[],
            description: "Goto next type definition",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_prev_class",
            aliases: &[],
            description: "Goto previous type definition",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_next_parameter",
            aliases: &[],
            description: "Goto next parameter",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_prev_parameter",
            aliases: &[],
            description: "Goto previous parameter",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_next_comment",
            aliases: &[],
            description: "Goto next comment",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_prev_comment",
            aliases: &[],
            description: "Goto previous comment",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_next_test",
            aliases: &[],
            description: "Goto next test",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_prev_test",
            aliases: &[],
            description: "Goto previous test",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_next_paragraph",
            aliases: &[],
            description: "Goto next paragraph",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_prev_paragraph",
            aliases: &[],
            description: "Goto previous paragraph",
            args: &[]
        function: client::
        },

        // Search
        Command {
            name: "search",
            aliases: &[],
            description: "Search for regex pattern",
            args: &[]
        function: client::
        },
        Command {
            name: "rsearch",
            aliases: &[],
            description: "Reverse search for regex pattern",
            args: &[]
        function: client::
        },
        Command {
            name: "search_next",
            aliases: &[],
            description: "Select next search match",
            args: &[]
        function: client::
        },
        Command {
            name: "search_prev",
            aliases: &[],
            description: "Select previous search match",
            args: &[]
        function: client::
        },
        Command {
            name: "search_selection",
            aliases: &[],
            description: "Use current selection as search pattern",
            args: &[]
        function: client::
        },
        Command {
            name: "make_search_word_bounded",
            aliases: &[],
            description: "Modify current search to make it word bounded",
            args: &[]
        function: client::
        },
        Command {
            name: "global_search",
            aliases: &[],
            description: "Global search in workspace folder",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_search_next",
            aliases: &[],
            description: "Add next search match to selection",
            args: &[]
        function: client::
        },
        Command {
            name: "extend_search_prev",
            aliases: &[],
            description: "Add previous search match to selection",
            args: &[]
        function: client::
        },

        // Document movement/panning
        Command {
            name: "align_view_middle",
            aliases: &[],
            description: "Align view middle",
            args: &[]
        function: client::
        },
        Command {
            name: "align_view_top",
            aliases: &[],
            description: "Align view top",
            args: &[]
        function: client::
        },
        Command {
            name: "align_view_center",
            aliases: &[],
            description: "Align view center",
            args: &[]
        function: client::
        },
        Command {
            name: "align_view_bottom",
            aliases: &[],
            description: "Align view bottom",
            args: &[]
        function: client::
        },
        Command {
            name: "scroll_up",
            aliases: &[],
            description: "Scroll view up",
            args: &[]
        function: client::
        },
        Command {
            name: "scroll_down",
            aliases: &[],
            description: "Scroll view down",
            args: &[]
        function: client::
        },
        Command {
            name: "page_up",
            aliases: &[],
            description: "Move page up",
            args: &[]
        function: client::
        },
        Command {
            name: "page_down",
            aliases: &[],
            description: "Move page down",
            args: &[]
        function: client::
        },
        Command {
            name: "half_page_up",
            aliases: &[],
            description: "Move half page up",
            args: &[]
        function: client::
        },
        Command {
            name: "half_page_down",
            aliases: &[],
            description: "Move half page down",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_file_start",
            aliases: &[],
            description: "Goto line number <n> else file start",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_file_end",
            aliases: &[],
            description: "Goto file end",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_window_top",
            aliases: &[],
            description: "Goto window top",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_window_center",
            aliases: &[],
            description: "Goto window center",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_window_bottom",
            aliases: &[],
            description: "Goto window bottom",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_line",
            aliases: &[],
            description: "Goto line",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_last_line",
            aliases: &[],
            description: "Goto last line",
            args: &[]
        function: client::
        },
        // Window
            // movement
        Command {
            name: "jump_view_right",
            aliases: &[],
            description: "Jump to right split",
            args: &[]
        function: client::
        },
        Command {
            name: "jump_view_left",
            aliases: &[],
            description: "Jump to left split",
            args: &[]
        function: client::
        },
        Command {
            name: "jump_view_up",
            aliases: &[],
            description: "Jump to split above",
            args: &[]
        function: client::
        },
        Command {
            name: "jump_view_down",
            aliases: &[],
            description: "Jump to split below",
            args: &[]
        function: client::
        },
        Command {
            name: "swap_view_right",
            aliases: &[],
            description: "Swap with right split",
            args: &[]
        function: client::
        },
        Command {
            name: "swap_view_left",
            aliases: &[],
            description: "Swap with left split",
            args: &[]
        function: client::
        },
        Command {
            name: "swap_view_up",
            aliases: &[],
            description: "Swap with split above",
            args: &[]
        function: client::
        },
        Command {
            name: "swap_view_down",
            aliases: &[],
            description: "Swap with split below",
            args: &[]
        function: client::
        },
        Command {
            name: "transpose_view",
            aliases: &[],
            description: "Transpose splits",
            args: &[]
        function: client::
        },
        Command {
            name: "rotate_view",
            aliases: &[],
            description: "Goto next window",
            args: &[]
        function: client::
        },
        // Open / Close
        Command {
            name: "quit",
            aliases: &["q"],
            description: "Close document view. Exit helix if none remain.",
            args: &[],
            function: client::
        },
        Command {
            name: "quit!",
            aliases: &["q!"],
            description: "Force close document view, ignoring unsaved changes. Exit helix if none remain.",
            args: &[],
            function: client::
        },
        Command {
            name: "wclose",
            aliases: &[],
            description: "Close window",
            args: &[]
        function: client::
        },
        Command {
            name: "wonly",
            aliases: &[],
            description: "Close windows except current",
            args: &[]
        function: client::
        },
        // MODE change
        Command {
            name: "command_mode",
            aliases: &[],
            description: "Enter command mode",
            args: &[]
        function: client::
        },
        Command {
            name: "normal_mode",
            aliases: &[],
            description: "Enter normal mode",
            args: &[]
        function: client::
        },
        Command {
            name: "select_mode",
            aliases: &[],
            description: "Enter selection extend mode",
            args: &[]
        function: client::
        },
        Command {
            name: "exit_select_mode",
            aliases: &[],
            description: "Exit selection mode",
            args: &[]
        function: client::
        },

        Command {
            name: "append_mode",
            aliases: &[],
            description: "Append after selection",
            args: &[]
        function: client::
        },
        Command {
            name: "append_at_line_end",
            aliases: &[],
            description: "Insert at end of line",
            args: &[]
        function: client::
        },

        Command {
            name: "insert_mode",
            aliases: &[],
            description: "Insert before selection",
            args: &[]
        function: client::
        },
        Command {
            name: "insert_at_line_start",
            aliases: &[],
            description: "Insert at start of line",
            args: &[]
        function: client::
        },
        // Register
        Command {
            name: "select_register",
            aliases: &[],
            description: "Select register",
            args: &[]
        function: client::
        },
        Command {
            name: "insert_register",
            aliases: &[],
            description: "Insert register",
            args: &[]
        function: client::
        },
        Command {
            name: "record_macro",
            aliases: &[],
            description: "Record macro",
            args: &[]
        function: client::
        },
        Command {
            name: "replay_macro",
            aliases: &[],
            description: "Replay macro",
            args: &[]
        function: client::
        },
    // Commandrow/shell
    Command {
        name: "pipe",
        aliases: &[],
        description: "Pipe each selection to the shell command.",
        args: &[Required(ShellCommand)],
        function: client::
    },
    Command {
        name: "pipe-to",
        aliases: &[],
        description: "Pipe each selection to the shell command, ignoring output.",
        args: &[Required(ShellCommand)],
        function: client::
    },
    Command {
        name: "run-shell-command",
        aliases: &["sh"],
        description: "Run a shell command",
        args: &[Required(ShellCommand)],
        function: client::
    },
            Command {
            name: "shell_pipe",
            aliases: &[],
            description: "Pipe selections through shell command",
            args: &[]
        function: client::
        },
        Command {
            name: "shell_pipe_to",
            aliases: &[],
            description: "Pipe selections into shell command ignoring output",
            args: &[]
        function: client::
        },
        Command {
            name: "shell_keep_pipe",
            aliases: &[],
            description: "Filter selections with shell predicate",
            args: &[]
        function: client::
        },
        Command {
            name: "suspend",
            aliases: &[],
            description: "Suspend and return to shell",
            args: &[]
        function: client::
        },

        // Window manipulation
        // TODO: remove duplicate buffer next commands
        Command {
            name: "cquit",
            aliases: &["cq"],
            description: "Quit with exit code (default 1). Accepts an optional integer exit code (:cq 2).",
            args: &[],
            function: client::
        },
        Command {
            name: "cquit!",
            aliases: &["cq!"],
            description: "Force quit with exit code (default 1) ignoring unsaved changes. Accepts an optional integer exit code (:cq! 2).",
            args: &[],
            function: client::
        },
        Command {
            name: "quit-all",
            aliases: &["qa"],
            description: "Close all views.",
            args: &[],
            function: client::
        },
        Command {
            name: "quit-all!",
            aliases: &["qa!"],
            description: "Force close all views ignoring unsaved changes.",
            args: &[],
            function: client::
        },
        Command {
            name: "buffer-next",
            aliases: &["bn", "bnext"],
            description: "Goto next buffer.",
            args: &[],
            function: client::
        },
        Command {
            name: "buffer-previous",
            aliases: &["bp", "bprev"],
            description: "Goto previous buffer.",
            args: &[],
            function: client::
        },
        Command {
            name: "goto_next_buffer",
            aliases: &[],
            description: "Goto next buffer",
            args: &[]
            function: client::
        },
        Command {
            name: "goto_previous_buffer",
            aliases: &[],
            description: "Goto previous buffer",
            args: &[]
            function: client::
        },
        Command {
            name: "goto_last_accessed_file",
            aliases: &[],
            description: "Goto last accessed file",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_last_modified_file",
            aliases: &[],
            description: "Goto last modified file",
            args: &[]
        function: client::
        },
        Command {
            name: "goto_last_modification",
            aliases: &[],
            description: "Goto last modification",
            args: &[]
        function: client::
        },
        // UI modes
            // Pickers
        Command {
            name: "last_picker",
            aliases: &[],
            description: "Open last picker",
            args: &[]
        function: client::
        },
        // Jumplist
        Command {
            name: "jumplist_picker",
            aliases: &[],
            description: "Open jumplist picker",
            args: &[]
        function: client::
        },
                    // jumplist commands
        Command {
            name: "jump_forward",
            aliases: &[],
            description: "Jump forward on jumplist",
            args: &[]
        function: client::
        },
        Command {
            name: "jump_backward",
            aliases: &[],
            description: "Jump backward on jumplist",
            args: &[]
        function: client::
        },
        Command {
            name: "save_selection",
            aliases: &[],
            description: "Save current selection to jumplist",
            args: &[]
        function: client::
        },
    // Command palette
        Command {
            name: "command_palette",
            aliases: &[],
            description: "Open command palette",
            args: &[]
        function: client::
        },
    // Server
    // Buffer CRUD

    Command {
        name: "log-open",
        aliases: &[],
        description: "Open the helix log file.",
        args: &[],
        function: client::
    },
    Command {
        name: "insert-output",
        aliases: &[],
        description: "Run shell command, inserting output before each selection.",
        args: &[Required(ShellCommand)],
        function: client::
    },
    Command {
        name: "append-output",
        aliases: &[],
        description: "Run shell command, appending output after each selection.",
        args: &[Required(ShellCommand)],
        function: client::
    },
    Command {
        name: "reflow",
        aliases: &[],
        description: "Hard-wrap the current selection of lines to a given width.",
        args: &[],
        function: client::
    },
        Command {
        name: "sort",
        aliases: &[],
        description: "Sort ranges in selection.",
        args: &[],
        function: client::
    },
    Command {
        name: "rsort",
        aliases: &[],
        description: "Sort ranges in selection in reverse order.",
        args: &[],
        function: client::
    },
        Command {
            name: "shell_insert_output",
            aliases: &[],
            description: "Insert shell command output before selections",
            args: &[]
        function: client::
        },
        Command {
            name: "shell_append_output",
            aliases: &[],
            description: "Append shell command output after selections",
            args: &[]
        function: client::
        },
            Command {
            name: "tutor",
            aliases: &[],
            description: "Open the tutorial.",
            args: &[],
            function: client::
        },
        Command {
            name: "vsplit",
            aliases: &["vs"],
            description: "Open file(s) in vertical splits.",
            args: &[Required(FilePaths)],
            function: client::
        },
        Command {
            name: "vsplit-new",
            aliases: &["vnew"],
            description: "Open a scratch buffer in a vertical split.",
            args: &[],
            function: client::
        },
        Command {
            name: "hsplit",
            aliases: &["hs", "sp"],
            description: "Open file(s) in horizontal splits.",
            args: &[Required(FilePaths)],
            function: client::
        },
        Command {
            name: "hsplit-new",
            aliases: &["hnew"],
            description: "Open a scratch buffer in a horizontal split.",
            args: &[],
            function: client::
        },        Command {
            name: "hsplit",
            aliases: &[],
            description: "Horizontal bottom split",
            args: &[]
        function: client::
        },
        Command {
            name: "hsplit_new",
            aliases: &[],
            description: "Horizontal bottom split scratch buffer",
            args: &[]
        function: client::
        },
        Command {
            name: "vsplit",
            aliases: &[],
            description: "Vertical right split",
            args: &[]
        function: client::
        },
        Command {
            name: "vsplit_new",
            aliases: &[],
            description: "Vertical right split scratch buffer",
            args: &[]
        function: client::
        },
Command {
        name: "change-current-directory",
        aliases: &["cd"],
        description: "Change the current working directory.",
        args: &[Required(DirectoryPath)],
        function: client::
    },
    Command {
        name: "show-directory",
        aliases: &["pwd"],
        description: "Show the current working directory.",
        args: &[],
        function: client::
    },
    Command {
        name: "encoding",
        aliases: &[],
        description: "Set encoding. Based on `https://encoding.spec.whatwg.org`.",
        args: &[],
        function: client::
    },
    Command {
        name: "reload",
        aliases: &[],
        description: "Discard changes and reload from the source file.",
        args: &[],
        function: client::
    },
    Command {
        name: "reload-all",
        aliases: &[],
        description: "Discard changes and reload all documents from the source files.",
        args: &[],
        function: client::
    },
    Command {
        name: "update",
        aliases: &[],
        description: "Write changes only if the file has been modified.",
        args: &[],
        function: client::
    },
    Command {
        name: "write-quit",
        aliases: &["wq", "x"],
        description: "Write changes to disk and close the current view. Accepts an optional path (:wq some/path.txt)",
        args: &[Optional(FilePath)],
        function: client::
    },
    Command {
        name: "write-quit!",
        aliases: &["wq!", "x!"],
        description: "Write changes to disk and close the current view forcefully. Accepts an optional path (:wq! some/path.txt)",
        args: &[Optional(FilePath)],
        function: client::
    },
    Command {
        name: "write-all",
        aliases: &["wa"],
        description: "Write changes from all buffers to disk.",
        args: &[],
        function: client::
    },
    Command {
        name: "write-quit-all",
        aliases: &["wqa", "xa"],
        description: "Write changes from all buffers to disk and close all views.",
        args: &[],
        function: client::
    },
    Command {
        name: "write-quit-all!",
        aliases: &["wqa!", "xa!"],
        description: "Write changes from all buffers to disk and close all views forcefully (ignoring unsaved changes).",
        args: &[],
        function: client::
    },
    Command {
        name: "earlier",
        aliases: &["ear"],
        description: "Jump back to an earlier point in edit history. Optional(ly) accepts a number of steps or a time duration.",
        args: &[Optional(UndoKind)],
        function: client::
    },
    Command {
        name: "later",
        aliases: &["lat"],
        description: "Jump to a later point in edit history. Accepts a number of steps or a time span.",
        args: &[Optional(UndoKind)],
        function: client::
    },
    Command {
        name: "indent-style",
        aliases: &[],
        description: "Set the indentation style. Syntax: [s,t] number. t for tabs, s for spaces. If neither s or t is supplied, number is assumed to be in spaces.",
        args: &[Required(IndentStyle)],
        function: server::
    },
    Command {
        name: "new",
        aliases: &["n"],
        description: "Create a new scratch buffer.",
        args: &[],
        function: server::
    },
    Command {
        name: "line-ending",
        aliases: &[],
        #[cfg(not(feature = "unicode-lines"))]
        description: "Set the document's default line ending. Options: crlf, lf.",
        #[cfg(feature = "unicode-lines")]
        description: "Set the document's default line ending. Options: crlf, lf, cr, ff, nel.",
        args: &[Required(LineEnding)],
        function: server::
    },
    Command {
        name: "write",
        aliases: &["w"],
        description: "Write changes to disk. Accepts an optional path (:write some/path.txt)",
        args: &[Optional(FilePath)],
        function: server::
    },
    Command {
        name: "write!",
        aliases: &["w!"],
        description: "Forcefully write changes to disk by creating necessary parent directories. Accepts an optional path (:write some/path.txt)",
        args: &[Optional(FilePath)]
        function: server::
    },
    Command {
        name: "open",
        aliases: &["o"],
        description: "Open file(s)",
        args: &[Required(FilePaths)],
        function: server::
    },
    Command {
        name: "buffer-close",
        aliases: &["bc", "bclose"],
        description: "Close buffer(s).",
        args: &[Optional(Buffers)],
        function: server::
    },
    Command {
        name: "buffer-close!",
        aliases: &["bc!", "bclose!"],
        description: "Close buffer(s) forcefully, ignoring unsaved changes.",
        args: &[Optional(Buffers)],
        function: server::
    },
    Command {
        name: "buffer-close-others",
        aliases: &["bco", "bcloseother"],
        description: "Close all buffers exept the one in focus.",
        args: &[],
        function: server::
    },
    Command {
        name: "buffer-close-others!",
        aliases: &["bco!", "bcloseother!"],
        description: "Forcefully close all buffers exept the one in focus.",
        args: &[],
        function: server::
    },
    Command {
        name: "buffer-close-all",
        aliases: &["bca", "bcloseall"],
        description: "Close all buffers.",
        args: &[],
        function: server::
    },
    Command {
        name: "buffer-close-all!",
        aliases: &["bca!", "bcloseall!"],
        description: "Forcefully close all buffers, ignoring unsaved.",
        args: &[],
        function: server::
    },
    // EDITS
        // Selections
        Command {
            name: "rotate_selection_contents_forward",
            aliases: &[],
            description: "Rotate selection contents forward",
            args: &[]
        function: server::
        },
        Command {
            name: "rotate_selection_contents_backward",
            aliases: &[],
            description: "Rotate selections contents backward",
            args: &[]
        function: server::
        },
        Command {
            name: "align_selections",
            aliases: &[],
            description: "Align selections in column",
            args: &[]
            function: server::
        },
        Command {
            name: "trim_selections",
            aliases: &[],
            description: "Trim whitespace from selections",
            args: &[]
            function: server::
        },
        Command {
            name: "join_selections",
            aliases: &[],
            description: "Join lines inside selection",
            args: &[]
        function: server::
        },
        Command {
            name: "join_selections_space",
            aliases: &[],
            description: "Join lines inside selection and select spaces",
            args: &[]
        function: server::
        },
        // Caseing
        Command {
            name: "switch_case",
            aliases: &[],
            description: "Switch (toggle) case",
            args: &[]
        function: server::
        },
        Command {
            name: "switch_to_uppercase",
            aliases: &[],
            description: "Switch to uppercase",
            args: &[]
        function: server::
        },
        Command {
            name: "switch_to_lowercase",
            aliases: &[],
            description: "Switch to lowercase",
            args: &[]
        function: server::
        },
        // Surround
        Command {
            name: "surround_add",
            aliases: &[],
            description: "Surround add",
            args: &[]
        function: server::
        },
        Command {
            name: "surround_replace",
            aliases: &[],
            description: "Surround replace",
            args: &[]
        function: server::
        },
        Command {
            name: "surround_delete",
            aliases: &[],
            description: "Surround delete",
            args: &[]
        function: server::
        },
        // Transaction history
        Command {
            name: "commit_undo_checkpoint",
            aliases: &[],
            description: "Commit changes to new checkpoint",
            args: &[]
        function: server::
        },
        Command {
            name: "undo",
            aliases: &[],
            description: "Undo change",
            args: &[]
        function: server::
        },
        Command {
            name: "redo",
            aliases: &[],
            description: "Redo change",
            args: &[]
        function: server::
        },
        Command {
            name: "earlier",
            aliases: &[],
            description: "Move backward in history",
            args: &[]
        function: server::
        },
        Command {
            name: "later",
            aliases: &[],
            description: "Move forward in history",
            args: &[]
        function: server::
        },
         // Change
        Command {
            name: "change_selection",
            aliases: &[],
            description: "Change selection",
            args: &[]
        function: server::
        },
        Command {
            name: "change_selection_noyank",
            aliases: &[],
            description: "Change selection without yanking",
            args: &[]
        function: server::
        },
        // Delete
        Command {
            name: "delete_selection",
            aliases: &[],
            description: "Delete selection",
            args: &[]
        function: server::
        },
        Command {
            name: "delete_selection_noyank",
            aliases: &[],
            description: "Delete selection without yanking",
            args: &[]
        function: server::
        },
        Command {
            name: "delete_char_backward",
            aliases: &[],
            description: "Delete previous char",
            args: &[]
        function: server::
        },
        Command {
            name: "delete_char_forward",
            aliases: &[],
            description: "Delete next char",
            args: &[]
        function: server::
        },
        Command {
            name: "delete_word_backward",
            aliases: &[],
            description: "Delete previous word",
            args: &[]
        function: server::
        },
        Command {
            name: "delete_word_forward",
            aliases: &[],
            description: "Delete next word",
            args: &[]
        function: server::
        },
        Command {
            name: "kill_to_line_start",
            aliases: &[],
            description: "Delete till start of line",
            args: &[]
        function: server::
        },
        Command {
            name: "kill_to_line_end",
            aliases: &[],
            description: "Delete till end of line",
            args: &[]
        function: server::
        },
        // Replace
        Command {
            name: "replace",
            aliases: &[],
            description: "Replace with new char",
            args: &[]
        function: server::
        },
        Command {
            name: "replace_with_yanked",
            aliases: &[],
            description: "Replace with yanked text",
            args: &[]
        function: server::
        },
        Command {
            name: "replace_selections_with_clipboard",
            aliases: &[],
            description: "Replace selections by clipboard content",
            args: &[]
        function: server::
        },
        Command {
            name: "replace_selections_with_primary_clipboard",
            aliases: &[],
            description: "Replace selections by primary clipboard",
            args: &[]
        function: server::
        },
        // Paste
        Command {
            name: "paste_after",
            aliases: &[],
            description: "Paste after selection",
            args: &[]
        function: server::
        },
        Command {
            name: "paste_before",
            aliases: &[],
            description: "Paste before selection",
            args: &[]
        function: server::
        },
        Command {
            name: "paste_clipboard_after",
            aliases: &[],
            description: "Paste clipboard after selections",
            args: &[]
        function: server::
        },
        Command {
            name: "paste_clipboard_before",
            aliases: &[],
            description: "Paste clipboard before selections",
            args: &[]
        function: server::
        },
        Command {
            name: "paste_primary_clipboard_after",
            aliases: &[],
            description: "Paste primary clipboard after selections",
            args: &[]
        function: server::
        },
        Command {
            name: "paste_primary_clipboard_before",
            aliases: &[],
            description: "Paste primary clipboard before selections",
            args: &[]
        function: server::
        },
    Command {
        name: "clipboard-paste-after",
        aliases: &[],
        description: "Paste system clipboard after selections.",
        args: &[],
        function: client::
    },
    Command {
        name: "clipboard-paste-before",
        aliases: &[],
        description: "Paste system clipboard before selections.",
        args: &[],
        function: client::
    },
    Command {
        name: "clipboard-paste-replace",
        aliases: &[],
        description: "Replace selections with content of system clipboard.",
        args: &[],
        function: client::
    },
    Command {
        name: "primary-clipboard-paste-after",
        aliases: &[],
        description: "Paste primary clipboard after selections.",
        args: &[],
        function: client::
    },
    Command {
        name: "primary-clipboard-paste-before",
        aliases: &[],
        description: "Paste primary clipboard before selections.",
        args: &[],
        function: client::
    },
    Command {
        name: "primary-clipboard-paste-replace",
        aliases: &[],
        description: "Replace selections with content of system primary clipboard.",
        args: &[],
        function: client::
    },
        // Open
        Command {
            name: "open_below",
            aliases: &[],
            description: "Open new line below selection",
            args: &[]
        function: server::
        },
        Command {
            name: "open_above",
            aliases: &[],
            description: "Open new line above selection",
            args: &[]
        function: server::
        },
        // Special insert mode keybindings
        Command {
            name: "insert_tab",
            aliases: &[],
            description: "Insert tab char",
            args: &[]
        function: server::
        },
        Command {
            name: "insert_newline",
            aliases: &[],
            description: "Insert newline char",
            args: &[]
        function: server::
        },
        // *crement
        Command {
            name: "increment",
            aliases: &[],
            description: "Increment item under cursor",
            args: &[]
        function: server::
        },
        Command {
            name: "decrement",
            aliases: &[],
            description: "Decrement item under cursor",
            args: &[]
        function: server::
        },
        // Indent
        Command {
            name: "indent",
            aliases: &[],
            description: "Indent selection",
            args: &[]
        function: server::
        },
        Command {
            name: "unindent",
            aliases: &[],
            description: "Unindent selection",
            args: &[]
        function: server::
        },
        // Lineadds
        Command {
            name: "add_newline_above",
            aliases: &[],
            description: "Add newline above",
            args: &[]
        function: server::
        },
        Command {
            name: "add_newline_below",
            aliases: &[],
            description: "Add newline below",
            args: &[]
        function: server::
        },
        // Lang
        Command {
            name: "toggle_comments",
            aliases: &[],
            description: "Comment/uncomment selections",
            args: &[]
        function: server::
        },
                // File
        Command {
            name: "file_picker",
            aliases: &[],
            description: "Open file picker",
            args: &[]
        function: client::
        },
        Command {
            name: "file_picker_in_current_directory",
            aliases: &[],
            description: "Open file picker at current working directory",
            args: &[]
        function: client::
        },
                // Buffer
        Command {
            name: "buffer_picker",
            aliases: &[],
            description: "Open buffer picker",
            args: &[]
        function: client::
        },
                // Symbol
        Command {
            name: "symbol_picker",
            aliases: &[],
            description: "Open symbol picker",
            args: &[]
        function: client::
        },
        Command {
            name: "workspace_symbol_picker",
            aliases: &[],
            description: "Open workspace symbol picker",
            args: &[]
        function: client::
        },
                // Diagnostics
        Command {
            name: "diagnostics_picker",
            aliases: &[],
            description: "Open diagnostic picker",
            args: &[]
        function: client::
        },
        Command {
            name: "workspace_diagnostics_picker",
            aliases: &[],
            description: "Open workspace diagnostic picker",
            args: &[]
        function: client::
        },
        // LSP

        Command {
            name: "set-language",
            aliases: &["lang"],
            description: "Set the language of current buffer.",
            args: &[Required(Languages)],
            function: client::
        },
        Command {
            name: "format_selections",
            aliases: &[],
            description: "Format selection using the LSP server provided formatter.",
            args: &[]
            function: server::
        },
        Command {
            name: "format",
            aliases: &["fmt"],
            description: "Format file(s) with the LSP server provided formatter.",
            args: &[Optional(FilePaths)],
            function: client::
        },
        Command {
        name: "lsp-workspace-command",
        aliases: &[],
        description: "Open workspace command picker",
        args: &[],
        function: client::
    },
    Command {
        name: "lsp-restart",
        aliases: &[],
        description: "Restarts the Language Server that is in use by the current doc",
        args: &[],
        function: client::
    },
        Command {
                name: "select_references_to_symbol_under_cursor",
                aliases: &[],
                description: "Select symbol references",
                args: &[]
        function: server::
        },
        Command {
            name: "code_action",
            aliases: &[],
            description: "Perform code action",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_definition",
            aliases: &[],
            description: "Goto definition",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_implementation",
            aliases: &[],
            description: "Goto implimentation",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_type_definition",
            aliases: &[],
            description: "Goto type definition",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_last_modification",
            aliases: &[],
            description: "Goto last modification",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_reference",
            aliases: &[],
            description: "Goto references",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_first_diag",
            aliases: &[],
            description: "Goto first diagnostic",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_last_diag",
            aliases: &[],
            description: "Goto last diagnostic",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_next_diag",
            aliases: &[],
            description: "Goto next diagnostic",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_prev_diag",
            aliases: &[],
            description: "Goto previous diagnostic",
            args: &[]
        function: server::
        },
        Command {
            name: "signature_help",
            aliases: &[],
            description: "Show signature help",
            args: &[]
        function: server::
        },
        Command {
            name: "completion",
            aliases: &[],
            description: "Invoke completion popup",
            args: &[]
        function: server::
        },
        Command {
            name: "hover",
            aliases: &[],
            description: "Show docs for item under cursor",
            args: &[]
        function: server::
        },
        Command {
            name: "rename_symbol",
            aliases: &[],
            description: "Rename symbol",
            args: &[]
        function: server::
        },
        // DAP
        Command {
            name: "debug-start",
            aliases: &["dbg"],
            description: "Start a debug session from a given template with given parameters.",
            args: &[],
            function: client::
        },
        Command {
            name: "debug-remote",
            aliases: &["dbg-tcp"],
            description: "Connect to a debug adapter by TCP address and start a debugging session from a given template with given parameters.",
            args: &[],
            function: client::
        },
        Command {
            name: "debug-eval",
            aliases: &[],
            description: "Evaluate expression in current debug context.",
            args: &[],
            function: client::
        },
        Command {
            name: "dap_launch",
            aliases: &[],
            description: "Launch debug target",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_toggle_breakpoint",
            aliases: &[],
            description: "Toggle breakpoint",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_continue",
            aliases: &[],
            description: "Continue program execution",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_pause",
            aliases: &[],
            description: "Pause program execution",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_step_in",
            aliases: &[],
            description: "Step in",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_step_out",
            aliases: &[],
            description: "Step out",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_next",
            aliases: &[],
            description: "Step to next",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_variables",
            aliases: &[],
            description: "List variables",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_terminate",
            aliases: &[],
            description: "End debug session",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_edit_condition",
            aliases: &[],
            description: "Edit breakpoint condition on current line",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_edit_log",
            aliases: &[],
            description: "Edit breakpoint log message on current line",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_switch_thread",
            aliases: &[],
            description: "Switch current thread",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_switch_stack_frame",
            aliases: &[],
            description: "Switch stack frame",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_enable_exceptions",
            aliases: &[],
            description: "Enable exception breakpoints",
            args: &[]
        function: server::
        },
        Command {
            name: "dap_disable_exceptions",
            aliases: &[],
            description: "Disable exception breakpoints",
            args: &[]
        function: server::
        },
        // VCS
        Command {
            name: "goto_next_change",
            aliases: &[],
            description: "Goto next change",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_prev_change",
            aliases: &[],
            description: "Goto previous change",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_first_change",
            aliases: &[],
            description: "Goto first change",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_last_change",
            aliases: &[],
            description: "Goto last change",
            args: &[]
        function: server::
        },
        // File open from selection
        Command {
            name: "goto_file",
            aliases: &[],
            description: "Goto files in selection",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_file_hsplit",
            aliases: &[],
            description: "Goto files in selection (hsplit)",
            args: &[]
        function: server::
        },
        Command {
            name: "goto_file_vsplit",
            aliases: &[],
            description: "Goto files in selection (vsplit)",
            args: &[]
        function: server::
        }
];

