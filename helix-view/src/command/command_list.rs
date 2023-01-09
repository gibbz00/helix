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
    // ### CLIENT only commands
    // Config
    Command {
        name: "config_reload",
        aliases: &[],
        description: "Refresh user config.",
        args: &[],
        function: config_reload
    },
    Command {
        name: "theme",
        aliases: &[],
        description: "Change the editor theme (show current theme if no name specified).",
        args: &[&CommandArgument::Optional(Theme)],
        function: theme
    },
    Command {
        name: "set_option",
        aliases: &["set"],
        description: "Set a config option at runtime.\n To disable smart case search for example; `:set search.smart_case false`.",
        args: &[&CommandArgument::Required(ConfigOptions)],
        function: set_option
    },
    Command {
        name: "get_option",
        aliases: &["get"],
        description: "Get the current value of a config option.",
        args: &[&CommandArgument::Required(ConfigOptions)],
        function: get_option
    },
    // ## MISC
    Command {
        name: "show_clipboard_provider",
        aliases: &[],
        description: "Show clipboard provider name in status bar.",
        args: &[],
        function: show_clipboard_provider
    },
    Command {
        name: "no_op",
        aliases: &[],
        description: "Do nothing.",
        args: &[],
        function: no_op
    },
    // Selection movement
    Command {
        name: "repeat_last_motion",
        aliases: &[],
        description: "Repeat last motion.",
        args: &[],
        function: repeat_last_motion
    },
    Command {
        name: "goto",
        aliases: &["g"],
        description: "Goto line number.",
        args: &[],
        function: goto
    },
    Command {
        name: "goto_line_start",
        aliases: &[],
        description: "Goto line start.",
        args: &[],
        function: goto_line_start
    },
    Command {
        name: "extend_to_line_start",
        aliases: &[],
        description: "Extend to line start.",
        args: &[],
        function: extend_to_line_start
    },
    Command {
        name: "goto_line_end",
        aliases: &[],
        description: "Goto line end.",
        args: &[],
        function: goto_line_end
    },
    Command {
        name: "extend_to_line_end",
        aliases: &[],
        description: "Extend to line end.",
        args: &[],
        function: extend_to_line_end
    },
    Command {
        name: "goto_line_end_newline",
        aliases: &[],
        description: "Goto newline at line end.",
        args: &[],
        function: goto_line_end_newline
    },
    Command {
        name: "extend_to_line_end_newline",
        aliases: &[],
        description: "Extend to line end.",
        args: &[],
        function: extend_to_line_end_newline
    },
    Command {
        name: "extend_line",
        aliases: &[],
        description: "Select current line, if already selected, extend to another line based on the anchor.",
        args: &[],
        function: extend_line
    },
    Command {
        name: "extend_line_below",
        aliases: &[],
        description: "Select current line, if already selected, extend to next line.",
        args: &[],
        function: extend_line_below
    },
    Command {
        name: "extend_line_above",
        aliases: &[],
        description: "Select current line, if already selected, extend to previous line.",
        args: &[],
        function: extend_line_above
    },
    Command {
        name: "extend_to_line_bounds",
        aliases: &[],
        description: "Extend selection to line bounds.",
        args: &[],
        function: extend_to_line_bounds
    },
    Command {
        name: "move_char_left",
        aliases: &[],
        description: "Move left.",
        args: &[],
        function: move_char_left
    },
    Command {
        name: "extend_char_left",
        aliases: &[],
        description: "Extend left.",
        args: &[],
        function: extend_char_left
    },
    Command {
        name: "move_char_right",
        aliases: &[],
        description: "Move right.",
        args: &[],
        function: move_char_right
    },
    Command {
        name: "extend_char_right",
        aliases: &[],
        description: "Extend right.",
        args: &[],
        function: extend_char_right
    },
    Command {
        name: "move_line_up",
        aliases: &[],
        description: "Move up.",
        args: &[],
        function: move_line_up
    },
    Command {
        name: "extend_line_up",
        aliases: &[],
        description: "Extend up.",
        args: &[],
        function: extend_line_up
    },
    Command {
        name: "move_line_down",
        aliases: &[],
        description: "Move down.",
        args: &[],
        function: move_line_down
    },
    Command {
        name: "extend_line_down",
        aliases: &[],
        description: "Extend down.",
        args: &[],
        function: extend_line_down
    },
    Command {
        name: "move_next_word_start",
        aliases: &[],
        description: "Move to start of next word.",
        args: &[],
        function: move_next_word_start
    },
    Command {
        name: "extend_next_word_start",
        aliases: &[],
        description: "Extend to start of next word.",
        args: &[],
        function: extend_next_word_start
    },
    Command {
        name: "move_prev_word_start",
        aliases: &[],
        description: "Move to start of previous word.",
        args: &[],
        function: move_prev_word_start
    },
    Command {
        name: "extend_prev_word_start",
        aliases: &[],
        description: "Extend to start of previous word.",
        args: &[],
        function: extend_prev_word_start
    },
    Command {
        name: "move_next_word_end",
        aliases: &[],
        description: "Move to end of next word.",
        args: &[],
        function: move_next_word_end
    },
    Command {
        name: "extend_next_word_end",
        aliases: &[],
        description: "Extend to end of next word.",
        args: &[],
        function: extend_next_word_end
    },
    Command {
        name: "move_prev_word_end",
        aliases: &[],
        description: "Move to end of previous word.",
        args: &[],
        function: move_prev_word_end
    },
    Command {
        name: "extend_prev_word_end",
        aliases: &[],
        description: "Extend to end of previous word.",
        args: &[],
        function: extend_prev_word_end
    },
    Command {
        name: "move_next_long_word_start",
        aliases: &[],
        description: "Move to start of next long word.",
        args: &[],
        function: move_next_long_word_start
    },
    Command {
        name: "extend_next_long_word_start",
        aliases: &[],
        description: "Extend to start of next long word.",
        args: &[],
        function: extend_next_long_word_start
    },
    Command {
        name: "move_prev_long_word_start",
        aliases: &[],
        description: "Move to start of previous long word.",
        args: &[],
        function: move_prev_long_word_start
    },
    Command {
        name: "extend_prev_long_word_start",
        aliases: &[],
        description: "Extend to start of previous long word.",
        args: &[],
        function: extend_prev_long_word_start
    },
    Command {
        name: "move_next_long_word_end",
        aliases: &[],
        description: "Move to end of next long word.",
        args: &[],
        function: move_next_long_word_end
    },
    Command {
        name: "extend_next_long_word_end",
        aliases: &[],
        description: "Extend to end of next long word.",
        args: &[],
        function: extend_next_long_word_end
    },
    Command {
        name: "goto_first_nonwhitespace",
        aliases: &[],
        description: "Goto first non_blank in line.",
        args: &[],
        function: goto_first_nonwhitespace
    },
    Command {
        name: "find_till_char",
        aliases: &[],
        description: "Move till next occurrence of char.",
        args: &[],
        function: find_till_char
    },
    Command {
        name: "extend_till_char",
        aliases: &[],
        description: "Extend till next occurrence of char.",
        args: &[],
        function: extend_till_char
    },      
    Command {
        name: "find_next_char",
        aliases: &[],
        description: "Move to next occurrence of char.",
        args: &[],
        function: find_next_char
    },
    Command {
        name: "extend_next_char",
        aliases: &[],
        description: "Extend to next occurrence of char.",
        args: &[],
        function: extend_next_char
    },      
    Command {
        name: "till_prev_char",
        aliases: &[],
        description: "Move till previous occurrence of char.",
        args: &[],
        function: till_prev_char
    },
    Command {
        name: "extend_till_prev_char",
        aliases: &[],
        description: "Extend till previous occurrence of char.",
        args: &[],
        function: extend_till_prev_char
    },      
    Command {
        name: "find_prev_char",
        aliases: &[],
        description: "Move to previous occurrence of char.",
        args: &[],
        function: find_prev_char
    },
    Command {
        name: "extend_prev_char",
        aliases: &[],
        description: "Extend to previous occurrence of char.",
        args: &[],
        function: extend_prev_char
    },      
    Command {
        name: "copy_selection_on_next_line",
        aliases: &[],
        description: "Copy selection on next line.",
        args: &[],
        function: copy_selection_on_next_line
    },
    Command {
        name: "copy_selection_on_prev_line",
        aliases: &[],
        description: "Copy selection on previous line.",
        args: &[],
        function: copy_selection_on_prev_line
    },
    Command {
        name: "select_all",
        aliases: &[],
        description: "Select whole document.",
        args: &[],
        function: select_all
    },
    Command {
        name: "select_regex",
        aliases: &[],
        description: "Select all regex matches inside selections.",
        args: &[],
        function: select_regex
    },
    Command {
        name: "keep_selections",
        aliases: &[],
        description: "Keep selections matching regex.",
        args: &[],
        function: keep_selections
    },
    Command {
        name: "remove_selections",
        aliases: &[],
        description: "Remove selections matching regex.",
        args: &[],
        function: remove_selections
    },
    Command {
        name: "split_selection",
        aliases: &[],
        description: "Split selections on regex matches.",
        args: &[],
        function: split_selection
    },
    Command {
        name: "split_selection_on_newline",
        aliases: &[],
        description: "Split selection on newlines.",
        args: &[],
        function: split_selection_on_newline
    },
    Command {
        name: "shrink_to_line_bounds",
        aliases: &[],
        description: "Shrink selection to line bounds.",
        args: &[],
        function: shrink_to_line_bounds
    },
    Command {
        name: "collapse_selection",
        aliases: &[],
        description: "Collapse selection into single cursor.",
        args: &[],
        function: collapse_selection
    },
    Command {
        name: "flip_selections",
        aliases: &[],
        description: "Flip selection cursor and anchor.",
        args: &[],
        function: flip_selections
    },
    Command {
        name: "ensure_selections_forward",
        aliases: &[],
        description: "Ensure all selections face forward.",
        args: &[],
        function: ensure_selections_forward
    },
    Command {
        name: "keep_primary_selection",
        aliases: &[],
        description: "Keep primary selection.",
        args: &[],
        function: keep_primary_selection
    },
    Command {
        name: "remove_primary_selection",
        aliases: &[],
        description: "Remove primary selection.",
        args: &[],
        function: remove_primary_selection
    },
    Command {
        name: "rotate_selections_forward",
        aliases: &[],
        description: "Rotate selections forward.",
        args: &[],
        function: rotate_selections_forward
    },
    Command {
        name: "rotate_selections_backward",
        aliases: &[],
        description: "Rotate selections backward.",
        args: &[],
        function: rotate_selections_backward
    },
    // Remove duplicates
    Command {
        name: "yank",
        aliases: &[],
        description: "Yank selection.",
        args: &[],
        function: yank
    },
    Command {
        name: "clipboard_yank",
        aliases: &["yank_main_selection_to_clipboard"],
        description: "Yank main selection to system clipboard.",
        args: &[],
        function: clipboard_yank
    },
    Command {
        name: "clipboard_yank_join",
        aliases: &["yank_joined_to_clipboard"],
        // FIXME: current UI can't display long doc.
        description: "Yank joined selections into system clipboard. A separator can be provided as an argument. Default value is newline.",
        args: &[&CommandArgument::Optional(Char)],
        function: clipboard_yank_join
    },
    Command {
        name: "primary_clipboard_yank",
        aliases: &[],
        description: "Yank main selection into system primary clipboard.",
        args: &[],
        function: primary_clipboard_yank
    },
    Command {
        name: "primary_clipboard_yank_join",
        aliases: &[],
        // FIXME: current UI can't display long doc.
        description: "Yank joined selections into system primary clipboard. A separator can be provided as first argument. Default value is newline.",
        args: &[&CommandArgument::Optional(Char)],
        function: primary_clipboard_yank_join
    },
    // Treesitter objects
    Command {
        name: "tree_sitter_subtree",
        aliases: &["ts_subtree"],
        description: "Display tree sitter subtree under cursor, primarily for debugging queries.",
        args: &[],
        function: tree_sitter_subtree
    },
    Command {
        name: "tree_sitter_scopes",
        aliases: &[],
        description: "Display tree sitter scopes, primarily for theming and development.",
        args: &[],
        function: tree_sitter_scopes
   },
    Command {
        name: "match_brackets",
        aliases: &[],
        description: "Goto matching bracket.",
        args: &[],
        function: match_brackets
    },
    Command {
        name: "select_textobject_around",
        aliases: &[],
        description: "Select around object.",
        args: &[],
        function: select_textobject_around
    },
    Command {
        name: "select_textobject_inner",
        aliases: &[],
        description: "Select inside object.",
        args: &[],
        function: select_textobject_inner
    },
    Command {
        name: "expand_selection",
        aliases: &[],
        description: "Expand selection to parent syntax node.",
        args: &[],
        function: expand_selection
    },
    Command {
        name: "shrink_selection",
        aliases: &[],
        description: "Shrink selection to previously expanded syntax node.",
        args: &[],
        function: shrink_selection
    },
    Command {
        name: "select_next_sibling",
        aliases: &[],
        description: "Select next sibling in syntax tree.",
        args: &[],
        function: select_next_sibling
    },
    Command {
        name: "select_prev_sibling",
        aliases: &[],
        description: "Select previous sibling in syntax tree.",
        args: &[],
        function: select_prev_sibling
    },
    Command {
        name: "goto_next_function",
        aliases: &[],
        description: "Goto next function.",
        args: &[],
        function: goto_next_function
    },
    Command {
        name: "goto_prev_function",
        aliases: &[],
        description: "Goto previous function.",
        args: &[],
        function: goto_prev_function
    },
    Command {
        name: "goto_next_class",
        aliases: &[],
        description: "Goto next type definition.",
        args: &[],
        function: goto_next_class
    },
    Command {
        name: "goto_prev_class",
        aliases: &[],
        description: "Goto previous type definition.",
        args: &[],
        function: goto_prev_class
    },
    Command {
        name: "goto_next_parameter",
        aliases: &[],
        description: "Goto next parameter.",
        args: &[],
        function: goto_next_parameter
    },
    Command {
        name: "goto_prev_parameter",
        aliases: &[],
        description: "Goto previous parameter.",
        args: &[],
        function: goto_prev_parameter
    },
    Command {
        name: "goto_next_comment",
        aliases: &[],
        description: "Goto next comment.",
        args: &[],
        function: goto_next_comment
    },
    Command {
        name: "goto_prev_comment",
        aliases: &[],
        description: "Goto previous comment.",
        args: &[],
        function: goto_prev_comment
    },
    Command {
        name: "goto_next_test",
        aliases: &[],
        description: "Goto next test.",
        args: &[],
        function: goto_next_test
    },
    Command {
        name: "goto_prev_test",
        aliases: &[],
        description: "Goto previous test.",
        args: &[],
        function: goto_prev_test
    },
    Command {
        name: "goto_next_paragraph",
        aliases: &[],
        description: "Goto next paragraph.",
        args: &[],
        function: goto_next_paragraph
    },
    Command {
        name: "goto_prev_paragraph",
        aliases: &[],
        description: "Goto previous paragraph.",
        args: &[],
        function: goto_prev_paragraph
    },
    // Search
    Command {
        name: "search",
        aliases: &[],
        description: "Search for regex pattern.",
        args: &[],
        function: search
    },
    Command {
        name: "rsearch",
        aliases: &[],
        description: "Reverse search for regex pattern.",
        args: &[],
        function: rsearch
    },
    Command {
        name: "search_next",
        aliases: &[],
        description: "Select next search match.",
        args: &[],
        function: search_next
    },
    Command {
        name: "search_prev",
        aliases: &[],
        description: "Select previous search match.",
        args: &[],
        function: search_prev
    },
    Command {
        name: "search_selection",
        aliases: &[],
        description: "Use current selection as search pattern.",
        args: &[],
        function: search_selection
    },
    Command {
        name: "make_search_word_bounded",
        aliases: &[],
        description: "Modify current search to make it word bounded.",
        args: &[],
        function: make_search_word_bounded
    },
    Command {
        name: "global_search",
        aliases: &[],
        description: "Global search in workspace folder.",
        args: &[],
        function: global_search
    },
    Command {
        name: "extend_search_next",
        aliases: &[],
        description: "Add next search match to selection.",
        args: &[],
        function: extend_search_next
    },
    Command {
        name: "extend_search_prev",
        aliases: &[],
        description: "Add previous search match to selection.",
        args: &[],
        function: extend_search_prev
    },
    // Document movement/panning
    Command {
        name: "align_view_middle",
        aliases: &[],
        description: "Align view middle.",
        args: &[],
        function: align_view_middle
    },
    Command {
        name: "align_view_top",
        aliases: &[],
        description: "Align view top.",
        args: &[],
        function: align_view_top
    },
    Command {
        name: "align_view_center",
        aliases: &[],
        description: "Align view center.",
        args: &[],
        function: align_view_center
    },
    Command {
        name: "align_view_bottom",
        aliases: &[],
        description: "Align view bottom.",
        args: &[],
        function: align_view_bottom
    },
    Command {
        name: "scroll_up",
        aliases: &[],
        description: "Scroll view up.",
        args: &[],
        function: scroll_up
    },
    Command {
        name: "scroll_down",
        aliases: &[],
        description: "Scroll view down.",
        args: &[],
        function: scroll_down
    },
    Command {
        name: "page_up",
        aliases: &[],
        description: "Move page up.",
        args: &[],
        function: page_up
    },
    Command {
        name: "page_down",
        aliases: &[],
        description: "Move page down.",
        args: &[],
        function: page_down
    },
    Command {
        name: "half_page_up",
        aliases: &[],
        description: "Move half page up.",
        args: &[],
        function: half_page_up
    },
    Command {
        name: "half_page_down",
        aliases: &[],
        description: "Move half page down.",
        args: &[],
        function: half_page_down
    },
    Command {
        name: "goto_file_start",
        aliases: &[],
        description: "Goto line number <n> else file start.",
        args: &[],
        function: goto_file_start
    },
    Command {
        name: "goto_file_end",
        aliases: &[],
        description: "Goto file end.",
        args: &[],
        function: goto_file_end
    },
    Command {
        name: "goto_window_top",
        aliases: &[],
        description: "Goto window top.",
        args: &[],
        function: goto_window_top
    },
    Command {
        name: "goto_window_center",
        aliases: &[],
        description: "Goto window center.",
        args: &[],
        function: goto_window_center
    },
    Command {
        name: "goto_window_bottom",
        aliases: &[],
        description: "Goto window bottom.",
        args: &[],
        function: goto_window_bottom
    },
    Command {
        name: "goto_line",
        aliases: &[],
        description: "Goto line.",
        args: &[],
        function: goto_line
    },
    Command {
        name: "goto_last_line",
        aliases: &[],
        description: "Goto last line.",
        args: &[],
        function: goto_last_line
    },
    // Buffer and bufferview movements
    Command {
        name: "buffer_next",
        aliases: &["bn", "bnext", "goto_next_buffer"],
        description: "Goto next buffer.",
        args: &[],
        function: buffer_next
    },
    Command {
        name: "buffer_previous",
        aliases: &["bp", "bprev", "goto_previous_buffer"],
        description: "Goto previous buffer.",
        args: &[],
        function: buffer_previous
    },
    Command {
        name: "goto_prev_buffer_access",
        aliases: &["goto_last_accessed_file"],
        description: "Goto previously accessed buffer.",
        args: &[],
        function: goto_prev_buffer_access
    },
    Command {
        name: "goto_prev_modified_buffer",
        aliases: &["goto_last_modified_file"],
        description: "Goto previoulsy modified buffer.",
        args: &[],
        function: goto_prev_modified_buffer
    },
    Command {
        name: "goto_last_modification",
        aliases: &[],
        description: "Goto last modification in current buffer.",
        args: &[],
        function: goto_last_modification
    },
    Command {
        name: "jump_view_left",
        aliases: &[],
        description: "Jump to left buffer view.",
        args: &[],
        function: jump_view_left
    },
    Command {
        name: "jump_view_right",
        aliases: &[],
        description: "Jump to right split.",
        args: &[],
        function: jump_view_right
    },
    Command {
        name: "jump_view_up",
        aliases: &[],
        description: "Jump to split above.",
        args: &[],
        function: jump_view_up
    },
    Command {
        name: "jump_view_down",
        aliases: &[],
        description: "Jump to split below.",
        args: &[],
        function: jump_view_down
    },
    Command {
        name: "swap_view_left",
        aliases: &[],
        description: "Swap with left split.",
        args: &[],
        function: swap_view_left
    },
    Command {
        name: "swap_view_right",
        aliases: &[],
        description: "Swap with right split.",
        args: &[],
        function: swap_view_right
    },
    Command {
        name: "swap_view_up",
        aliases: &[],
        description: "Swap with split above.",
        args: &[],
        function: swap_view_up
    },
    Command {
        name: "swap_view_down",
        aliases: &[],
        description: "Swap with split below.",
        args: &[],
        function: swap_view_down
    },
    Command {
        name: "transpose_view",
        aliases: &[],
        description: "Transpose splits.",
        args: &[],
        function: transpose_view
    },
    Command {
        name: "rotate_view",
        aliases: &[],
        description: "Goto next window.",
        args: &[],
        function: rotate_view
    },
    // Open / Close
    Command {
        name: "quit",
        aliases: &["q, wclose"],
        description: "Close buffer view. Exit helix if none remain.",
        args: &[],
            function: window_close
    },
    Command {
        name: "quit!",
        aliases: &["q!"],
        description: "Force close buffer view, ignoring unsaved changes. Exit helix if none remain.",
        args: &[],
            function: quit_force
    },
    Command {
        name: "cquit",
        aliases: &["cq"],
        description: "Quit helix with exit code (default 1). Accepts an optional integer exit code (:cq 2).",
        args: &[&CommandArgument::Optional(Integer)],
        function: cquit
    },
    Command {
        name: "cquit!",
        aliases: &["cq!"],
        description: "Force quit helix with exit code (default 1) ignoring unsaved changes. Accepts an optional integer exit code (:cq! 2).",
        args: &[&CommandArgument::Optional(Integer)],
        function: cquit_force
    },
    Command {
        name: "quit_all",
        aliases: &["qa"],
        description: "Close all buffer views and exit helix.",
        args: &[],
        function: quit_all
    },
    Command {
        name: "quit_all!",
        aliases: &["qa!"],
        description: "Force close all buffer views and exit helix, ignoring unsaved changes.",
        args: &[],
        function: quit_all_force
    },
    Command {
        name: "quit_other",
        aliases: &["wonly"],
        description: "Close all buffer views except the current one.",
        args: &[],
        function: quit_other
    },
    // MODE change
    Command {
        name: "command_mode",
        aliases: &[],
        description: "Enter command mode.",
        args: &[],
        function: command_mode
    },
    Command {
        name: "normal_mode",
        aliases: &[],
        description: "Enter normal mode.",
        args: &[],
        function: normal_mode
    },
    Command {
        name: "select_mode",
        aliases: &[],
        description: "Enter selection extend mode.",
        args: &[],
        function: select_mode
    },
    Command {
        name: "exit_select_mode",
        aliases: &[],
        description: "Exit selection mode.",
        args: &[],
        function: exit_select_mode
    },
    Command {
        name: "append_mode",
        aliases: &[],
        description: "Append after selection.",
        args: &[],
        function: append_mode
    },
    Command {
        name: "append_at_line_end",
        aliases: &[],
        description: "Insert at end of line.",
        args: &[],
        function: append_at_line_end
    },
    Command {
        name: "insert_mode",
        aliases: &[],
        description: "Insert before selection.",
        args: &[],
        function: insert_mode
    },
    Command {
        name: "insert_at_line_start",
        aliases: &[],
        description: "Insert at start of line.",
        args: &[],
        function: insert_at_line_start
    },
    // Register
    Command {
        name: "select_register",
        aliases: &[],
        description: "Select register.",
        args: &[],
        function: select_register
    },
    Command {
        name: "insert_register",
        aliases: &[],
        description: "Insert register.",
        args: &[],
        function: insert_register
    },
    Command {
        name: "record_macro",
        aliases: &[],
        description: "Record macro.",
        args: &[],
        function: record_macro
    },
    Command {
        name: "replay_macro",
        aliases: &[],
        description: "Replay macro.",
        args: &[],
        function: replay_macro
    },
    // Commandrow/shell
    Command {
        name: "pipe",
        aliases: &[],
        description: "Pipe each selection to the shell command.",
        args: &[&CommandArgument::Required(ShellCommand)],
        function: pipe
    },
    Command {
        name: "pipe_to",
        aliases: &["pipe_silent"],
        description: "Pipe each selection to the shell command, ignoring output.",
        args: &[&CommandArgument::Required(ShellCommand)],
        function: pipe_silent
    },
    Command {
        name: "run_shell_command",
        aliases: &["sh"],
        description: "Run a shell command.",
        args: &[&CommandArgument::Required(ShellCommand)],
        function: run_shell_command
    },
    Command {
        name: "shell_pipe",
        aliases: &[],
        description: "Pipe selections through shell command.",
        args: &[],
        function: shell_pipe
    },
    Command {
        name: "shell_pipe_to",
        aliases: &[],
        description: "Pipe selections into shell command ignoring output.",
        args: &[],
        function: shell_pipe_to
    },
    Command {
        name: "shell_keep_pipe",
        aliases: &[],
        description: "Filter selections with shell predicate.",
        args: &[],
        function: shell_keep_pipe
    },
    Command {
        name: "suspend",
        aliases: &[],
        description: "Suspend and return to shell.",
        args: &[],
        function: suspend
    },
    // Pickers
    Command {
        name: "last_picker",
        aliases: &[],
        description: "Open last picker.",
        args: &[],
        function: last_picker
    },
    // Jumplist
    Command {
        name: "jumplist_picker",
        aliases: &[],
        description: "Open jumplist picker.",
        args: &[],
        function: jumplist_picker
    },
    Command {
        name: "jump_forward",
        aliases: &[],
        description: "Jump forward on jumplist.",
        args: &[],
        function: jump_forward
    },
    Command {
        name: "jump_backward",
        aliases: &[],
        description: "Jump backward on jumplist.",
        args: &[],
        function: jump_backward
    },
    Command {
        name: "save_selection",
        aliases: &[],
        description: "Save current selection to jumplist.",
        args: &[],
        function: save_selection
    },
    // Command palette
    Command {
        name: "command_palette",
        aliases: &[],
        description: "Open command palette.",
        args: &[],
        function: command_palette
    },
    // Might/will be server forwarded
    // Server env
    Command {
        name: "change_current_directory",
        aliases: &["cd"],
        description: "Change the current working directory.",
        args: &[&CommandArgument::Required(DirectoryPath)],
        function: change_current_directory
    },
    Command {
        name: "show_directory",
        aliases: &["pwd"],
        description: "Show the current working directory.",
        args: &[],
        function: show_directory
    },
    // File open
    Command {
        name: "config_open",
        aliases: &[],
        description: "Open the user config.toml file.",
        args: &[],
        function: config_open
    },
    Command {
        name: "log_open",
        aliases: &[],
        description: "Open the helix log file.",
        args: &[],
        function: log_open
    },
    Command {
        name: "tutor",
        aliases: &[],
        description: "Open the tutorial.",
        args: &[],
        function: tutor
    },
    Command {
        name: "new",
        aliases: &["n"],
        description: "Create a new scratch buffer.",
        args: &[],
        function: new
    },
    Command {
        name: "open",
        aliases: &["o"],
        description: "Open file(s).",
        args: &[&CommandArgument::Required(FilePaths)],
        function: open
    },
    Command {
        name: "file_picker",
        aliases: &[],
        description: "Open file picker.",
        args: &[],
        function: file_picker
    },
    Command {
        name: "file_picker_in_current_directory",
        aliases: &[],
        description: "Open file picker at current working directory.",
        args: &[],
        function: file_picker_in_current_directory
    },
    Command {
        name: "buffer_picker",
        aliases: &[],
        description: "Open buffer picker.",
        args: &[],
        function: buffer_picker
    },
    Command {
        name: "vsplit",
        aliases: &["vs"],
        description: "Open file(s) in vertical splits. Opens current buffer view if no argument is supplied.",
        args: &[&CommandArgument::Required(FilePaths)],
        function: vsplit
    },
    Command {
        name: "vsplit_new",
        aliases: &["vnew"],
        description: "Open a scratch buffer in a vertical split.",
        args: &[],
        function: vsplit_new
    },
    Command {
        name: "hsplit",
        aliases: &["hs", "sp"],
        description: "Open file(s) in horizontal splits. Opens current buffer view if no argument is supplied.",
        args: &[&CommandArgument::Required(FilePaths)],
        function: hsplit
    },
    Command {
        name: "hsplit_new",
        aliases: &["hnew"],
        description: "Open a scratch buffer in a horizontal split.",
        args: &[],
        function: hsplit_new
    },        
    Command {
        name: "goto_file",
        aliases: &[],
        description: "Goto files in selection.",
        args: &[],
        function: goto_file
    },
    Command {
        name: "goto_file_hsplit",
        aliases: &[],
        description: "Goto files in selection (hsplit).",
        args: &[],
        function: goto_file_hsplit
    },
    Command {
        name: "goto_file_vsplit",
        aliases: &[],
        description: "Goto files in selection (vsplit).",
        args: &[],
        function: goto_file_vsplit
    },
    // Bufferwide write/update
    Command {
        name: "reload",
        aliases: &[],
        description: "Discard changes and reload from the source file.",
        args: &[],
        function: reload
    },
    Command {
        name: "reload_all",
        aliases: &[],
        description: "Discard changes and reload all documents from the source files.",
        args: &[],
        function: reload_all
    },
    Command {
        name: "update",
        aliases: &[],
        description: "Write changes only if the file has been modified.",
        args: &[],
        function: update
    },
    Command {
        name: "write",
        aliases: &["w"],
        description: "Write changes to disk. Accepts an optional path (:write some/path.txt).",
        args: &[&CommandArgument::Optional(FilePath)],
        function: write
    },
    Command {
        name: "write!",
        aliases: &["w!"],
        description: "Forcefully write changes to disk by creating necessary parent directories. Accepts an optional path (:write some/path.txt).",
        args: &[&CommandArgument::Optional(FilePath)],
        function: write_force
    },
    Command {
        name: "write_quit",
        aliases: &["wq", "x"],
        description: "Write changes to disk and close the current buffer view. Accepts an optional path (:wq some/path.txt).",
        args: &[&CommandArgument::Optional(FilePath)],
        function: write_quit
    },
    Command {
        name: "write_quit!",
        aliases: &["wq!", "x!"],
        description: "Write changes to disk and close the current buffer view forcefully. Accepts an optional path (:wq! some/path.txt).",
        args: &[&CommandArgument::Optional(FilePath)],
        function: write_quit_force
    },
    Command {
        name: "write_all",
        aliases: &["wa"],
        description: "Write changes from all buffers to disk.",
        args: &[],
        function: write_all
    },
    Command {
        name: "write_quit_all",
        aliases: &["wqa", "xa"],
        description: "Write changes from all buffers to disk and close all views.",
        args: &[],
        function: write_quit_all
    },
    Command {
        name: "write_quit_all!",
        aliases: &["wqa!", "xa!"],
        description: "Write changes from all buffers to disk and close all views forcefully (ignoring unsaved changes).",
        args: &[],
        function: write_quit_all_force
    },
    Command {
        name: "buffer_close",
        aliases: &["bc", "bclose"],
        description: "Close buffer(s).",
        args: &[&CommandArgument::Optional(Buffers)],
        function: buffer_close
    },
    Command {
        name: "buffer_close!",
        aliases: &["bc!", "bclose!"],
        description: "Close buffer(s) forcefully, ignoring unsaved changes.",
        args: &[&CommandArgument::Optional(Buffers)],
        function: buffer_close_force
    },
    Command {
        name: "buffer_close_others",
        aliases: &["bco", "bcloseother"],
        description: "Close all buffers exept the one in focus.",
        args: &[],
        function: buffer_close_others
    },
    Command {
        name: "buffer_close_others!",
        aliases: &["bco!", "bcloseother!"],
        description: "Forcefully close all buffers exept the one in focus.",
        args: &[],
        function: buffer_close_others_force
    },
    Command {
        name: "buffer_close_all",
        aliases: &["bca", "bcloseall"],
        description: "Close all buffers.",
        args: &[],
        function: buffer_close_all
    },
    Command {
        name: "buffer_close_all!",
        aliases: &["bca!", "bcloseall!"],
        description: "Forcefully close all buffers, ignoring unsaved.",
        args: &[],
        function: buffer_close_all_force
    },
    // Buffer settings
    Command {
        name: "encoding",
        aliases: &[],
        description: "Set encoding. Based on `https://encoding.spec.whatwg.org`.",
        args: &[],
        function: encoding
    },
     Command {
        name: "indent_style",
        aliases: &[],
        description: "Set the indentation style. Syntax: [s,t] number. t for tabs, s for spaces. If neither s or t is supplied, number is assumed to be in spaces.",
        args: &[&CommandArgument::Required(IndentStyle)],
        function: indent_style
    },
    Command {
        name: "line_ending",
        aliases: &[],
        #[cfg(not(feature = "unicode_lines"))]
        description: "Set the document's default line ending. Options: crlf, lf.",
        #[cfg(feature = "unicode_lines")]
        description: "Set the document's default line ending. Options: crlf, lf, cr, ff, nel.",
        args: &[&CommandArgument::Required(LineEnding)],
        // TODO: use cfg attribute in function definition
        function: line_ending
    },
    // Shell to buffer
    Command {
        name: "shell_insert_output",
        aliases: &["insert_output", "shi"],
        description: "Insert shell command output before each selections.",
        args: &[&CommandArgument::Required(ShellCommand)],
        function: shell_insert_output
    },
    Command {
        name: "shell_append_output",
        aliases: &["append_output", "sha"],
        description: "Append shell command output after each selections.",
        args: &[&CommandArgument::Required(ShellCommand)],
        function: shell_append_output
    },
    // Selection manipulation
    Command {
        name: "reflow",
        aliases: &[],
        description: "Hard_wrap the current selection of lines to a given width.",
        args: &[],
        function: reflow
    },
    Command {
        name: "sort",
        aliases: &[],
        description: "Sort ranges in selection.",
        args: &[],
        function: sort
    },
    Command {
        name: "rsort",
        aliases: &[],
        description: "Sort ranges in selection in reverse order.",
        args: &[],
        function: rsort
    },
    Command {
        name: "rotate_selection_contents_forward",
        aliases: &[],
        description: "Rotate selection contents forward.",
        args: &[],
        function: rotate_selection_contents_forward
    },
    Command {
        name: "rotate_selection_contents_backward",
        aliases: &[],
        description: "Rotate selections contents backward.",
        args: &[],
        function: rotate_selection_contents_backward
    },
    Command {
        name: "align_selections",
        aliases: &[],
        description: "Align selections in column.",
        args: &[],
        function: align_selections
    },
    Command {
        name: "trim_selections",
        aliases: &[],
        description: "Trim whitespace from selections.",
        args: &[],
        function: trim_selections
    },
    Command {
        name: "join_selections",
        aliases: &[],
        description: "Join lines inside selection.",
        args: &[],
        function: join_selections
    },
    Command {
        name: "join_selections_space",
        aliases: &[],
        description: "Join lines inside selection and select spaces.",
        args: &[],
        function: join_selections_space
    },
    // Caseing
    Command {
        name: "switch_case",
        aliases: &[],
        description: "Switch (toggle) case.",
        args: &[],
        function: switch_case
    },
    Command {
        name: "switch_to_uppercase",
        aliases: &[],
        description: "Switch to uppercase.",
        args: &[],
        function: switch_to_uppercase
    },
    Command {
        name: "switch_to_lowercase",
        aliases: &[],
        description: "Switch to lowercase.",
        args: &[],
        function: switch_to_lowercase
    },
    // Surround
    Command {
        name: "surround_add",
        aliases: &[],
        description: "Surround add.",
        args: &[],
        function: surround_add
    },
    Command {
        name: "surround_replace",
        aliases: &[],
        description: "Surround replace.",
        args: &[],
        function: surround_replace
    },
    Command {
        name: "surround_delete",
        aliases: &[],
        description: "Surround delete.",
        args: &[],
        function: surround_delete
    },
    // Change
    Command {
        name: "change_selection",
        aliases: &[],
        description: "Change selection.",
        args: &[],
        function: change_selection
    },
    Command {
        name: "change_selection_noyank",
        aliases: &[],
        description: "Change selection without yanking.",
        args: &[],
        function: change_selection_noyank
    },
    // Delete
    Command {
        name: "delete_selection",
        aliases: &[],
        description: "Delete selection.",
        args: &[],
        function: delete_selection
    },
    Command {
        name: "delete_selection_noyank",
        aliases: &[],
        description: "Delete selection without yanking.",
        args: &[],
        function: delete_selection_noyank
    },
    Command {
        name: "delete_char_backward",
        aliases: &[],
        description: "Delete previous char.",
        args: &[],
        function: delete_char_backward
    },
    Command {
        name: "delete_char_forward",
        aliases: &[],
        description: "Delete next char.",
        args: &[],
        function: delete_char_forward
    },
    Command {
        name: "delete_word_backward",
        aliases: &[],
        description: "Delete previous word.",
        args: &[],
        function: delete_word_backward
    },
    Command {
        name: "delete_word_forward",
        aliases: &[],
        description: "Delete next word.",
        args: &[],
        function: delete_word_forward
    },
    Command {
        name: "kill_to_line_start",
        aliases: &[],
        description: "Delete till start of line.",
        args: &[],
        function: kill_to_line_start
    },
    Command {
        name: "kill_to_line_end",
        aliases: &[],
        description: "Delete till end of line.",
        args: &[],
        function: kill_to_line_end
    },
    // Replace
    Command {
        name: "replace",
        aliases: &[],
        description: "Replace with new char.",
        args: &[],
        function: replace
    },
    Command {
        name: "replace_with_yanked",
        aliases: &[],
        description: "Replace with yanked text.",
        args: &[],
        function: replace_with_yanked
    },
    Command {
        name: "replace_with_clipboard",
        aliases: &["clipboard_paste_replace", "replace_selections_with_clipboard"],
        description: "Replace selections with clipboard content.",
        args: &[],
        function: clipboard_paste_replace
    },
    Command {
        name: "replace_with_primary_clipboard",
        aliases: &["primary_clipboard_paste_replace", "replace_selections_with_primary_clipboard"],
        description: "Replace selections with content of system primary clipboard.",
        args: &[],
        function: primary_clipboard_paste_replace
    },
    // Paste
    Command {
        name: "paste_after",
        aliases: &[],
        description: "Paste after selection.",
        args: &[],
        function: paste_after
    },
    Command {
        name: "clipboard_paste_after",
        aliases: &["paste_clipboard_after"],
        description: "Paste from clipboard after selections.",
        args: &[],
        function: clipboard_paste_after
    },
    Command {
        name: "primary_clipboard_paste_after",
        aliases: &["paste_primary_clipboard_after"],
        description: "Paste from primary clipboard after selections.",
        args: &[],
        function: primary_clipboard_paste_after
    },
    Command {
        name: "paste_before",
        aliases: &[],
        description: "Paste before selection.",
        args: &[],
        function: paste_before
    },
    Command {
        name: "clipboard_paste_before",
        aliases: &["paste_clipboard_before"],
        description: "Paste from clipboard before selections.",
        args: &[],
        function: clipboard_paste_before
    },
    Command {
        name: "primary_clipboard_paste_before",
        aliases: &["paste_primary_clipboard_before"],
        description: "Paste primary clipboard before selections.",
        args: &[],
        function: primary_clipboard_paste_before
    },
    Command {
        name: "increment",
        aliases: &[],
        description: "Increment item under cursor.",
        args: &[],
        function: increment
    },
    Command {
        name: "decrement",
        aliases: &[],
        description: "Decrement item under cursor.",
        args: &[],
        function: decrement
    },
    Command {
        name: "indent",
        aliases: &[],
        description: "Indent selection.",
        args: &[],
        function: indent
    },
    Command {
        name: "unindent",
        aliases: &[],
        description: "Unindent selection.",
        args: &[],
        function: unindent
    },
    Command {
        name: "add_newline_below",
        aliases: &[],
        description: "Add newline below.",
        args: &[],
        function: add_newline_below
    },
    Command {
        name: "open_below",
        aliases: &[],
        description: "Open new line below selection.",
        args: &[],
        function: open_below
    },
    Command {
        name: "add_newline_above",
        aliases: &[],
        description: "Add newline above.",
        args: &[],
        function: add_newline_above
    },
    Command {
        name: "open_above",
        aliases: &[],
        description: "Open new line above selection.",
        args: &[],
        function: open_above
    },
    // Special insert mode keybindings
    Command {
        name: "insert_tab",
        aliases: &[],
        description: "Insert tab char.",
        args: &[],
        function: insert_tab
    },
    Command {
        name: "insert_newline",
        aliases: &[],
        description: "Insert newline char.",
        args: &[],
        function: insert_newline
    },
    // Undohistory
    Command {
        name: "commit_undo_checkpoint",
        aliases: &[],
        description: "Commit changes to undohistory.",
        args: &[],
        function: commit_undo_checkpoint
    },
    Command {
        name: "undo",
        aliases: &[],
        description: "Undo change.",
        args: &[],
        function: undo
    },
    Command {
        name: "earlier",
        aliases: &["ear"],
        description: "Jump back to an earlier point in edit history. Optionaly accepts a number of steps or a time duration.",
        args: &[&CommandArgument::Optional(UndoKind)],
        function: earlier
    },
    Command {
        name: "redo",
        aliases: &[],
        description: "Redo change.",
        args: &[],
        function: redo
    },
    Command {
        name: "later",
        aliases: &["lat"],
        description: "Jump to a later point in edit history. Optionaly accepts a number of steps or a time span.",
        args: &[&CommandArgument::Optional(UndoKind)],
        function: later
    },
    // LSP forwards
    Command {
        name: "set_language",
        aliases: &["lang"],
        description: "Set the language of current buffer.",
        args: &[&CommandArgument::Required(Languages)],
        function: set_language
    },
    Command {
        name: "lsp_restart",
        aliases: &[],
        description: "Restarts the Language Server that is in use by the current doc.",
        args: &[],
        function: lsp_restart
    },
    Command {
        name: "code_action",
        aliases: &[],
        description: "Perform code action.",
        args: &[],
        function: code_action
    },
    Command {
        name: "lsp_workspace_command",
        aliases: &[],
        description: "Open workspace command picker.",
        args: &[],
        function: lsp_workspace_command
    },
    Command {
        name: "format_selections",
        aliases: &[],
        description: "Format selection using the LSP server provided formatter.",
        args: &[],
        function: format_selections
    },
    Command {
        name: "format",
        aliases: &["fmt"],
        description: "Format file(s) with the LSP server provided formatter.",
        args: &[&CommandArgument::Optional(FilePaths)],
        function: format
    },
    Command {
        name: "toggle_comments",
        aliases: &[],
        description: "Comment/uncomment selections.",
        args: &[],
        function: toggle_comments
    },
    Command {
        name: "select_references_to_symbol_under_cursor",
        aliases: &[],
        description: "Select symbol references.",
        args: &[],
        function: select_references_to_symbol_under_cursor
    },
    Command {
        name: "symbol_picker",
        aliases: &[],
        description: "Open symbol picker.",
        args: &[],
        function: symbol_picker
    },
    Command {
        name: "workspace_symbol_picker",
        aliases: &[],
        description: "Open workspace symbol picker.",
        args: &[],
        function: workspace_symbol_picker
    },
    Command {
        name: "diagnostics_picker",
        aliases: &[],
        description: "Open diagnostic picker.",
        args: &[],
        function: diagnostics_picker
    },
    Command {
        name: "workspace_diagnostics_picker",
        aliases: &[],
        description: "Open workspace diagnostic picker.",
        args: &[],
        function: workspace_diagnostics_picker
    },
    Command {
        name: "goto_definition",
        aliases: &[],
        description: "Goto definition.",
        args: &[],
        function: goto_definition
    },
    Command {
        name: "goto_implementation",
        aliases: &[],
        description: "Goto implimentation.",
        args: &[],
        function: goto_implementation
    },
    Command {
        name: "goto_type_definition",
        aliases: &[],
        description: "Goto type definition.",
        args: &[],
        function: goto_type_definition
    },
    Command {
        name: "goto_reference",
        aliases: &[],
        description: "Goto references.",
        args: &[],
        function: goto_reference
    },
    Command {
        name: "goto_first_diag",
        aliases: &[],
        description: "Goto first diagnostic.",
        args: &[],
        function: goto_first_diag
    },
    Command {
        name: "goto_last_diag",
        aliases: &[],
        description: "Goto last diagnostic.",
        args: &[],
        function: goto_last_diag
    },
    Command {
        name: "goto_next_diag",
        aliases: &[],
        description: "Goto next diagnostic.",
        args: &[],
        function: goto_next_diag
    },
    Command {
        name: "goto_prev_diag",
        aliases: &[],
        description: "Goto previous diagnostic.",
        args: &[],
        function: goto_prev_diag
    },
    Command {
        name: "signature_help",
        aliases: &[],
        description: "Show signature help.",
        args: &[],
        function: signature_help
    },
    Command {
        name: "completion",
        aliases: &[],
        description: "Invoke completion popup.",
        args: &[],
        function: completion
    },
    Command {
        name: "hover",
        aliases: &[],
        description: "Show docs for item under cursor.",
        args: &[],
        function: hover
    },
    Command {
        name: "rename_symbol",
        aliases: &[],
        description: "Rename symbol.",
        args: &[],
        function: rename_symbol
    },
    // DAP
    Command {
        name: "debug_start",
        aliases: &["dbg"],
        description: "Start a debug session from a given template with given parameters.",
        args: &[],
        function: debug_start
    },
    Command {
        name: "debug_remote",
        aliases: &["dbg_tcp"],
        description: "Connect to a debug adapter by TCP address and start a debugging session from a given template with given parameters.",
        args: &[],
        function: debug_remote
    },
    Command {
        name: "debug_eval",
        aliases: &[],
        description: "Evaluate expression in current debug context.",
        args: &[],
        function: debug_eval
    },
    Command {
        name: "dap_launch",
        aliases: &[],
        description: "Launch debug target.",
        args: &[],
        function: dap_launch
    },
    Command {
        name: "dap_toggle_breakpoint",
        aliases: &[],
        description: "Toggle breakpoint.",
        args: &[],
        function: dap_toggle_breakpoint
    },
    Command {
        name: "dap_continue",
        aliases: &[],
        description: "Continue program execution.",
        args: &[],
        function: dap_continue
    },
    Command {
        name: "dap_pause",
        aliases: &[],
        description: "Pause program execution.",
        args: &[],
        function: dap_pause
    },
    Command {
        name: "dap_step_in",
        aliases: &[],
        description: "Step in.",
        args: &[],
        function: dap_step_in
    },
    Command {
        name: "dap_step_out",
        aliases: &[],
        description: "Step out.",
        args: &[],
        function: dap_step_out
    },
    Command {
        name: "dap_next",
        aliases: &[],
        description: "Step to next.",
        args: &[],
        function: dap_next
    },
    Command {
        name: "dap_variables",
        aliases: &[],
        description: "List variables.",
        args: &[],
        function: dap_variables
    },
    Command {
        name: "dap_terminate",
        aliases: &[],
        description: "End debug session.",
        args: &[],
        function: dap_terminate
    },
    Command {
        name: "dap_edit_condition",
        aliases: &[],
        description: "Edit breakpoint condition on current line.",
        args: &[],
        function: dap_edit_condition
    },
    Command {
        name: "dap_edit_log",
        aliases: &[],
        description: "Edit breakpoint log message on current line.",
        args: &[],
        function: dap_edit_log
    },
    Command {
        name: "dap_switch_thread",
        aliases: &[],
        description: "Switch current thread.",
        args: &[],
        function: dap_switch_thread
    },
    Command {
        name: "dap_switch_stack_frame",
        aliases: &[],
        description: "Switch stack frame.",
        args: &[],
        function: dap_switch_stack_frame
    },
    Command {
        name: "dap_enable_exceptions",
        aliases: &[],
        description: "Enable exception breakpoints.",
        args: &[],
        function: dap_enable_exceptions
    },
    Command {
        name: "dap_disable_exceptions",
        aliases: &[],
        description: "Disable exception breakpoints.",
        args: &[],
        function: dap_disable_exceptions
    },
    // VCS
    Command {
        name: "goto_next_change",
        aliases: &[],
        description: "Goto next change.",
        args: &[],
        function: goto_next_change
    },
    Command {
        name: "goto_prev_change",
        aliases: &[],
        description: "Goto previous change.",
        args: &[],
        function: goto_prev_change
    },
    Command {
        name: "goto_first_change",
        aliases: &[],
        description: "Goto first change.",
        args: &[],
        function: goto_first_change
    },
    Command {
        name: "goto_last_change",
        aliases: &[],
        description: "Goto last change.",
        args: &[],
        function: goto_last_change
    },
];

