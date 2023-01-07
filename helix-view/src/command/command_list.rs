use super::{Command, CommandArguments};

pub const COMMAND_LIST: &'static[Command] = &[
    Command {
        name: "quit",
        aliases: &["q"],
        description: "Close the current view.",
        args: &[],
    },
    Command {
        name: "quit!",
        aliases: &["q!"],
        description: "Force close the current view, ignoring unsaved changes.",
        args: &[],
    },
    Command {
        name: "open",
        aliases: &["o"],
        description: "Open file(s)",
        args: &[FilePaths],
    },
    Command {
        name: "buffer-close",
        aliases: &["bc", "bclose"],
        description: "Close buffer(s).",
        args: &[OptionalBuffers],
    },
    Command {
        name: "buffer-close!",
        aliases: &["bc!", "bclose!"],
        description: "Close buffer(s) forcefully, ignoring unsaved changes.",
        args: &[OptionalBuffers],
    },
    Command {
        name: "buffer-close-others",
        aliases: &["bco", "bcloseother"],
        description: "Close all buffers exept the one in focus.",
        args: &[],
    },
    Command {
        name: "buffer-close-others!",
        aliases: &["bco!", "bcloseother!"],
        description: "Forcefully close all buffers exept the one in focus.",
        args: &[],
    },
    Command {
        name: "buffer-close-all",
        aliases: &["bca", "bcloseall"],
        description: "Close all buffers.",
        args: &[],
    },
    Command {
        name: "buffer-close-all!",
        aliases: &["bca!", "bcloseall!"],
        description: "Forcefully close all buffers, ignoring unsaved.",
        args: &[],
    },
    Command {
        name: "buffer-next",
        aliases: &["bn", "bnext"],
        description: "Goto next buffer.",
        args: &[],
    },
    Command {
        name: "buffer-previous",
        aliases: &["bp", "bprev"],
        description: "Goto previous buffer.",
        args: &[],
    },
    Command {
        name: "write",
        aliases: &["w"],
        description: "Write changes to disk. Accepts an optional path (:write some/path.txt)",
        args: &[OptionalFilePath],
    },
    Command {
        name: "write!",
        aliases: &["w!"],
        description: "Forcefully write changes to disk by creating necessary subdirectories. Accepts an optional path (:write some/path.txt)",
        args: &[OptionalFilePath]
    },
    Command {
        name: "new",
        aliases: &["n"],
        description: "Create a new scratch buffer.",
        args: &[],
    },
    Command {
        name: "format",
        aliases: &["fmt"],
        description: "Format file(s) with the LSP server provided formatter.",
        args: &[OptionalFilePaths],
    },
    Command {
        name: "indent-style",
        aliases: &[],
        description: "Set the indentation style. Syntax: [s,t] number. t for tabs, s for spaces. If neither s or t is supplied, number is assumed to be in spaces.",
        args: &[IndentStyle],
    },
    Command {
        name: "line-ending",
        aliases: &[],
        #[cfg(not(feature = "unicode-lines"))]
        description: "Set the document's default line ending. Options: crlf, lf.",
        #[cfg(feature = "unicode-lines")]
        description: "Set the document's default line ending. Options: crlf, lf, cr, ff, nel.",
        args: &[LineEnding],
    },
    Command {
        name: "earlier",
        aliases: &["ear"],
        description: "Jump back to an earlier point in edit history. Optionally accepts a number of steps or a time duration.",
        args: &[OptionalUndoKind],
    },
    Command {
        name: "later",
        aliases: &["lat"],
        description: "Jump to a later point in edit history. Accepts a number of steps or a time span.",
        args: &[OptionalUndoKind],
    },
    Command {
        name: "write-quit",
        aliases: &["wq", "x"],
        description: "Write changes to disk and close the current view. Accepts an optional path (:wq some/path.txt)",
        args: &[OptionalFilePath],
    },
    Command {
        name: "write-quit!",
        aliases: &["wq!", "x!"],
        description: "Write changes to disk and close the current view forcefully. Accepts an optional path (:wq! some/path.txt)",
        args: &[OptionalFilePath],
    },
    Command {
        name: "write-all",
        aliases: &["wa"],
        description: "Write changes from all buffers to disk.",
        args: &[],
    },
    Command {
        name: "write-quit-all",
        aliases: &["wqa", "xa"],
        description: "Write changes from all buffers to disk and close all views.",
        args: &[],
    },
    Command {
        name: "write-quit-all!",
        aliases: &["wqa!", "xa!"],
        description: "Write changes from all buffers to disk and close all views forcefully (ignoring unsaved changes).",
        args: &[],
    },
    Command {
        name: "quit-all",
        aliases: &["qa"],
        description: "Close all views.",
        args: &[],
    },
    Command {
        name: "quit-all!",
        aliases: &["qa!"],
        description: "Force close all views ignoring unsaved changes.",
        args: &[],
    },
    Command {
        name: "cquit",
        aliases: &["cq"],
        description: "Quit with exit code (default 1). Accepts an optional integer exit code (:cq 2).",
        args: &[],
    },
    Command {
        name: "cquit!",
        aliases: &["cq!"],
        description: "Force quit with exit code (default 1) ignoring unsaved changes. Accepts an optional integer exit code (:cq! 2).",
        args: &[],
    },
    Command {
        name: "theme",
        aliases: &[],
        description: "Change the editor theme (show current theme if no name specified).",
        args: &[OptionalTheme],
    },
    Command {
        name: "clipboard-yank",
        aliases: &[],
        description: "Yank main selection into system clipboard.",
        args: &[],
    },
    Command {
        name: "clipboard-yank-join",
        aliases: &[],
        description: "Yank joined selections into system clipboard. A separator can be provided as first argument. Default value is newline.", // FIXME: current UI can't display long doc.
        args: &[],
    },
    Command {
        name: "primary-clipboard-yank",
        aliases: &[],
        description: "Yank main selection into system primary clipboard.",
        args: &[],
    },
    Command {
        name: "primary-clipboard-yank-join",
        aliases: &[],
        description: "Yank joined selections into system primary clipboard. A separator can be provided as first argument. Default value is newline.", // FIXME: current UI can't display long doc.
        args: &[],
    },
    Command {
        name: "clipboard-paste-after",
        aliases: &[],
        description: "Paste system clipboard after selections.",
        args: &[],
    },
    Command {
        name: "clipboard-paste-before",
        aliases: &[],
        description: "Paste system clipboard before selections.",
        args: &[],
    },
    Command {
        name: "clipboard-paste-replace",
        aliases: &[],
        description: "Replace selections with content of system clipboard.",
        args: &[],
    },
    Command {
        name: "primary-clipboard-paste-after",
        aliases: &[],
        description: "Paste primary clipboard after selections.",
        args: &[],
    },
    Command {
        name: "primary-clipboard-paste-before",
        aliases: &[],
        description: "Paste primary clipboard before selections.",
        args: &[],
    },
    Command {
        name: "primary-clipboard-paste-replace",
        aliases: &[],
        description: "Replace selections with content of system primary clipboard.",
        args: &[],
    },
    Command {
        name: "show-clipboard-provider",
        aliases: &[],
        description: "Show clipboard provider name in status bar.",
        args: &[],
    },
    Command {
        name: "change-current-directory",
        aliases: &["cd"],
        description: "Change the current working directory.",
        args: &[DirectoryPath],
    },
    Command {
        name: "show-directory",
        aliases: &["pwd"],
        description: "Show the current working directory.",
        args: &[],
    },
    Command {
        name: "encoding",
        aliases: &[],
        description: "Set encoding. Based on `https://encoding.spec.whatwg.org`.",
        args: &[],
    },
    Command {
        name: "reload",
        aliases: &[],
        description: "Discard changes and reload from the source file.",
        args: &[],
    },
    Command {
        name: "reload-all",
        aliases: &[],
        description: "Discard changes and reload all documents from the source files.",
        args: &[],
    },
    Command {
        name: "update",
        aliases: &[],
        description: "Write changes only if the file has been modified.",
        args: &[],
    },
    Command {
        name: "lsp-workspace-command",
        aliases: &[],
        description: "Open workspace command picker",
        args: &[],
    },
    Command {
        name: "lsp-restart",
        aliases: &[],
        description: "Restarts the Language Server that is in use by the current doc",
        args: &[],
    },
    Command {
        name: "tree-sitter-scopes",
        aliases: &[],
        description: "Display tree sitter scopes, primarily for theming and development.",
        args: &[],
   },
    Command {
        name: "debug-start",
        aliases: &["dbg"],
        description: "Start a debug session from a given template with given parameters.",
        args: &[],
    },
    Command {
        name: "debug-remote",
        aliases: &["dbg-tcp"],
        description: "Connect to a debug adapter by TCP address and start a debugging session from a given template with given parameters.",
        args: &[],
    },
    Command {
        name: "debug-eval",
        aliases: &[],
        description: "Evaluate expression in current debug context.",
        args: &[],
    },
    Command {
        name: "vsplit",
        aliases: &["vs"],
        description: "Open file(s) in vertical splits.",
        args: &[FilePaths],
    },
    Command {
        name: "vsplit-new",
        aliases: &["vnew"],
        description: "Open a scratch buffer in a vertical split.",
        args: &[],
    },
    Command {
        name: "hsplit",
        aliases: &["hs", "sp"],
        description: "Open file(s) in horizontal splits.",
        args: &[FilePaths],
    },
    Command {
        name: "hsplit-new",
        aliases: &["hnew"],
        description: "Open a scratch buffer in a horizontal split.",
        args: &[],
    },
    Command {
        name: "tutor",
        aliases: &[],
        description: "Open the tutorial.",
        args: &[],
    },
    Command {
        name: "goto",
        aliases: &["g"],
        description: "Goto line number.",
        args: &[],
    },
    Command {
        name: "set-language",
        aliases: &["lang"],
        description: "Set the language of current buffer.",
        args: &[Languages],
    },
    Command {
        name: "set-option",
        aliases: &["set"],
        description: "Set a config option at runtime.\n To disable smart case search for example; `:set search.smart-case false`.",
        args: &[ConfigOptions],
    },
    Command {
        name: "get-option",
        aliases: &["get"],
        description: "Get the current value of a config option.",
        args: &[ConfigOptions],
    },
    Command {
        name: "sort",
        aliases: &[],
        description: "Sort ranges in selection.",
        args: &[],
    },
    Command {
        name: "rsort",
        aliases: &[],
        description: "Sort ranges in selection in reverse order.",
        args: &[],
    },
    Command {
        name: "reflow",
        aliases: &[],
        description: "Hard-wrap the current selection of lines to a given width.",
        args: &[],
    },
    Command {
        name: "tree-sitter-subtree",
        aliases: &["ts-subtree"],
        description: "Display tree sitter subtree under cursor, primarily for debugging queries.",
        args: &[],
    },
    Command {
        name: "config-reload",
        aliases: &[],
        description: "Refresh user config.",
        args: &[],
    },
    Command {
        name: "config-open",
        aliases: &[],
        description: "Open the user config.toml file.",
        args: &[],
    },
    Command {
        name: "log-open",
        aliases: &[],
        description: "Open the helix log file.",
        args: &[],
    },
    Command {
        name: "insert-output",
        aliases: &[],
        description: "Run shell command, inserting output before each selection.",
        args: &[ShellCommand],
    },
    Command {
        name: "append-output",
        aliases: &[],
        description: "Run shell command, appending output after each selection.",
        args: &[ShellCommand],
    },
    Command {
        name: "pipe",
        aliases: &[],
        description: "Pipe each selection to the shell command.",
        args: &[ShellCommand],
    },
    Command {
        name: "pipe-to",
        aliases: &[],
        description: "Pipe each selection to the shell command, ignoring output.",
        args: &[ShellCommand],
    },
    Command {
        name: "run-shell-command",
        aliases: &["sh"],
        description: "Run a shell command",
        args: &[ShellCommand],
    },
    // ## CLIENT
    Command {
            name: "no_op",
            aliases: &[],
            description: "Do nothing",
            args: &[]
    },
    Command {
            name: "repeat_last_motion",
            aliases: &[],
            description: "Repeat last motion",
            args: &[]
    },
    // Selection movement
        Command {
            name: "goto_line_start",
            aliases: &[],
            description: "Goto line start",
            args: &[]
    },
        Command {
            name: "extend_to_line_start",
            aliases: &[],
            description: "Extend to line start",
            args: &[]
        },
        Command {
            name: "goto_line_end",
            aliases: &[],
            description: "Goto line end",
            args: &[]
        },
        Command {
            name: "extend_to_line_end",
            aliases: &[],
            description: "Extend to line end",
            args: &[]
        },
        Command {
            name: "goto_line_end_newline",
            aliases: &[],
            description: "Goto newline at line end",
            args: &[]
        },
        Command {
            name: "extend_to_line_end_newline",
            aliases: &[],
            description: "Extend to line end",
            args: &[]
        },
        Command {
            name: "extend_line",
            aliases: &[],
            description: "Select current line, if already selected, extend to another line based on the anchor",
            args: &[]
        },
        Command {
            name: "extend_line_below",
            aliases: &[],
            description: "Select current line, if already selected, extend to next line",
            args: &[]
        },
        Command {
            name: "extend_line_above",
            aliases: &[],
            description: "Select current line, if already selected, extend to previous line",
            args: &[]
        },
        Command {
            name: "extend_to_line_bounds",
            aliases: &[],
            description: "Extend selection to line bounds",
            args: &[]
        },
        Command {
            name: "move_char_left",
            aliases: &[],
            description: "Move left",
            args: &[]
        },
        Command {
            name: "extend_char_left",
            aliases: &[],
            description: "Extend left",
            args: &[]
        },
        Command {
            name: "move_char_right",
            aliases: &[],
            description: "Move right",
            args: &[]
        },
        Command {
            name: "extend_char_right",
            aliases: &[],
            description: "Extend right",
            args: &[]
        },
        Command {
            name: "move_line_up",
            aliases: &[],
            description: "Move up",
            args: &[]
        },
        Command {
            name: "extend_line_up",
            aliases: &[],
            description: "Extend up",
            args: &[]
        },
        Command {
            name: "move_line_down",
            aliases: &[],
            description: "Move down",
            args: &[]
        },
        Command {
            name: "extend_line_down",
            aliases: &[],
            description: "Extend down",
            args: &[]
        },
        Command {
            name: "move_next_word_start",
            aliases: &[],
            description: "Move to start of next word",
            args: &[]
        },
        Command {
            name: "extend_next_word_start",
            aliases: &[],
            description: "Extend to start of next word",
            args: &[]
        },
        Command {
            name: "move_prev_word_start",
            aliases: &[],
            description: "Move to start of previous word",
            args: &[]
        },
        Command {
            name: "extend_prev_word_start",
            aliases: &[],
            description: "Extend to start of previous word",
            args: &[]
        },
        Command {
            name: "move_next_word_end",
            aliases: &[],
            description: "Move to end of next word",
            args: &[]
        },
        Command {
            name: "extend_next_word_end",
            aliases: &[],
            description: "Extend to end of next word",
            args: &[]
        },
        Command {
            name: "move_prev_word_end",
            aliases: &[],
            description: "Move to end of previous word",
            args: &[]
        },
        Command {
            name: "extend_prev_word_end",
            aliases: &[],
            description: "Extend to end of previous word",
            args: &[]
        },
        Command {
            name: "move_next_long_word_start",
            aliases: &[],
            description: "Move to start of next long word",
            args: &[]
        },
        Command {
            name: "extend_next_long_word_start",
            aliases: &[],
            description: "Extend to start of next long word",
            args: &[]
        },
        Command {
            name: "move_prev_long_word_start",
            aliases: &[],
            description: "Move to start of previous long word",
            args: &[]
        },
        Command {
            name: "extend_prev_long_word_start",
            aliases: &[],
            description: "Extend to start of previous long word",
            args: &[]
        },
        Command {
            name: "move_next_long_word_end",
            aliases: &[],
            description: "Move to end of next long word",
            args: &[]
        },
        Command {
            name: "extend_next_long_word_end",
            aliases: &[],
            description: "Extend to end of next long word",
            args: &[]
        },
        Command {
            name: "goto_first_nonwhitespace",
            aliases: &[],
            description: "Goto first non-blank in line",
            args: &[]
        },
        Command {
            name: "find_till_char",
            aliases: &[],
            description: "Move till next occurrence of char",
            args: &[]
        },
        Command {
            name: "extend_till_char",
            aliases: &[],
            description: "Extend till next occurrence of char",
            args: &[]
        },      
        Command {
            name: "find_next_char",
            aliases: &[],
            description: "Move to next occurrence of char",
            args: &[]
        },
        Command {
            name: "extend_next_char",
            aliases: &[],
            description: "Extend to next occurrence of char",
            args: &[]
        },      
        Command {
            name: "till_prev_char",
            aliases: &[],
            description: "Move till previous occurrence of char",
            args: &[]
        },
        Command {
            name: "extend_till_prev_char",
            aliases: &[],
            description: "Extend till previous occurrence of char",
            args: &[]
        },      
        Command {
            name: "find_prev_char",
            aliases: &[],
            description: "Move to previous occurrence of char",
            args: &[]
        },
        Command {
            name: "extend_prev_char",
            aliases: &[],
            description: "Extend to previous occurrence of char",
            args: &[]
        },      
        Command {
            name: "copy_selection_on_next_line",
            aliases: &[],
            description: "Copy selection on next line",
            args: &[]
        },
        Command {
            name: "copy_selection_on_prev_line",
            aliases: &[],
            description: "Copy selection on previous line",
            args: &[]
        },
        Command {
            name: "select_all",
            aliases: &[],
            description: "Select whole document",
            args: &[]
        },
        Command {
            name: "select_regex",
            aliases: &[],
            description: "Select all regex matches inside selections",
            args: &[]
        },
        Command {
            name: "keep_selections",
            aliases: &[],
            description: "Keep selections matching regex",
            args: &[]
        },
        Command {
            name: "remove_selections",
            aliases: &[],
            description: "Remove selections matching regex",
            args: &[]
        },
        Command {
            name: "split_selection",
            aliases: &[],
            description: "Split selections on regex matches",
            args: &[]
        },
        Command {
            name: "split_selection_on_newline",
            aliases: &[],
            description: "Split selection on newlines",
            args: &[]
        },
        Command {
            name: "shrink_to_line_bounds",
            aliases: &[],
            description: "Shrink selection to line bounds",
            args: &[]
        },
        Command {
            name: "collapse_selection",
            aliases: &[],
            description: "Collapse selection into single cursor",
            args: &[]
        },
        Command {
            name: "flip_selections",
            aliases: &[],
            description: "Flip selection cursor and anchor",
            args: &[]
        },
        Command {
            name: "ensure_selections_forward",
            aliases: &[],
            description: "Ensure all selections face forward",
            args: &[]
        },
        Command {
            name: "keep_primary_selection",
            aliases: &[],
            description: "Keep primary selection",
            args: &[]
        },
        Command {
            name: "remove_primary_selection",
            aliases: &[],
            description: "Remove primary selection",
            args: &[]
        },
        Command {
            name: "rotate_selections_forward",
            aliases: &[],
            description: "Rotate selections forward",
            args: &[]
        },
        Command {
            name: "rotate_selections_backward",
            aliases: &[],
            description: "Rotate selections backward",
            args: &[]
        },
        Command {
            name: "yank",
            aliases: &[],
            description: "Yank selection",
            args: &[]
        },
        Command {
            name: "yank_joined_to_clipboard",
            aliases: &[],
            description: "Join and yank selections to clipboard",
            args: &[]
        },
        Command {
            name: "yank_main_selection_to_clipboard",
            aliases: &[],
            description: "Yank main selection to clipboard",
            args: &[]
        },
        Command {
            name: "yank_joined_to_primary_clipboard",
            aliases: &[],
            description: "Join and yank selections to primary clipboard",
            args: &[]
        },
        Command {
            name: "yank_main_selection_to_primary_clipboard",
            aliases: &[],
            description: "Yank main selection to primary clipboard",
            args: &[]
        },
        // LSP or treesitter?
        Command {
                name: "select_references_to_symbol_under_cursor",
                aliases: &[],
                description: "Select symbol references",
                args: &[]
        },
        // Treesitter objects
        Command {
            name: "match_brackets",
            aliases: &[],
            description: "Goto matching bracket",
            args: &[]
        },
        Command {
            name: "select_textobject_around",
            aliases: &[],
            description: "Select around object",
            args: &[]
        },
        Command {
            name: "select_textobject_inner",
            aliases: &[],
            description: "Select inside object",
            args: &[]
        },
        Command {
            name: "expand_selection",
            aliases: &[],
            description: "Expand selection to parent syntax node",
            args: &[]
        },
        Command {
            name: "shrink_selection",
            aliases: &[],
            description: "Shrink selection to previously expanded syntax node",
            args: &[]
        },
        Command {
            name: "select_next_sibling",
            aliases: &[],
            description: "Select next sibling in syntax tree",
            args: &[]
        },
        Command {
            name: "select_prev_sibling",
            aliases: &[],
            description: "Select previous sibling in syntax tree",
            args: &[]
        },
        Command {
            name: "goto_next_function",
            aliases: &[],
            description: "Goto next function",
            args: &[]
        },
        Command {
            name: "goto_prev_function",
            aliases: &[],
            description: "Goto previous function",
            args: &[]
        },
        Command {
            name: "goto_next_class",
            aliases: &[],
            description: "Goto next type definition",
            args: &[]
        },
        Command {
            name: "goto_prev_class",
            aliases: &[],
            description: "Goto previous type definition",
            args: &[]
        },
        Command {
            name: "goto_next_parameter",
            aliases: &[],
            description: "Goto next parameter",
            args: &[]
        },
        Command {
            name: "goto_prev_parameter",
            aliases: &[],
            description: "Goto previous parameter",
            args: &[]
        },
        Command {
            name: "goto_next_comment",
            aliases: &[],
            description: "Goto next comment",
            args: &[]
        },
        Command {
            name: "goto_prev_comment",
            aliases: &[],
            description: "Goto previous comment",
            args: &[]
        },
        Command {
            name: "goto_next_test",
            aliases: &[],
            description: "Goto next test",
            args: &[]
        },
        Command {
            name: "goto_prev_test",
            aliases: &[],
            description: "Goto previous test",
            args: &[]
        },
        Command {
            name: "goto_next_paragraph",
            aliases: &[],
            description: "Goto next paragraph",
            args: &[]
        },
        Command {
            name: "goto_prev_paragraph",
            aliases: &[],
            description: "Goto previous paragraph",
            args: &[]
        },

        // Search
        Command {
            name: "search",
            aliases: &[],
            description: "Search for regex pattern",
            args: &[]
        },
        Command {
            name: "rsearch",
            aliases: &[],
            description: "Reverse search for regex pattern",
            args: &[]
        },
        Command {
            name: "search_next",
            aliases: &[],
            description: "Select next search match",
            args: &[]
        },
        Command {
            name: "search_prev",
            aliases: &[],
            description: "Select previous search match",
            args: &[]
        },
        Command {
            name: "search_selection",
            aliases: &[],
            description: "Use current selection as search pattern",
            args: &[]
        },
        Command {
            name: "make_search_word_bounded",
            aliases: &[],
            description: "Modify current search to make it word bounded",
            args: &[]
        },
        Command {
            name: "global_search",
            aliases: &[],
            description: "Global search in workspace folder",
            args: &[]
        },
        Command {
            name: "extend_search_next",
            aliases: &[],
            description: "Add next search match to selection",
            args: &[]
        },
        Command {
            name: "extend_search_prev",
            aliases: &[],
            description: "Add previous search match to selection",
            args: &[]
        },

        // Document movement/panning
        Command {
            name: "align_view_middle",
            aliases: &[],
            description: "Align view middle",
            args: &[]
        },
        Command {
            name: "align_view_top",
            aliases: &[],
            description: "Align view top",
            args: &[]
        },
        Command {
            name: "align_view_center",
            aliases: &[],
            description: "Align view center",
            args: &[]
        },
        Command {
            name: "align_view_bottom",
            aliases: &[],
            description: "Align view bottom",
            args: &[]
        },
        Command {
            name: "scroll_up",
            aliases: &[],
            description: "Scroll view up",
            args: &[]
        },
        Command {
            name: "scroll_down",
            aliases: &[],
            description: "Scroll view down",
            args: &[]
        },
        Command {
            name: "page_up",
            aliases: &[],
            description: "Move page up",
            args: &[]
        },
        Command {
            name: "page_down",
            aliases: &[],
            description: "Move page down",
            args: &[]
        },
        Command {
            name: "half_page_up",
            aliases: &[],
            description: "Move half page up",
            args: &[]
        },
        Command {
            name: "half_page_down",
            aliases: &[],
            description: "Move half page down",
            args: &[]
        },
        Command {
            name: "goto_file_start",
            aliases: &[],
            description: "Goto line number <n> else file start",
            args: &[]
        },
        Command {
            name: "goto_file_end",
            aliases: &[],
            description: "Goto file end",
            args: &[]
        },
        Command {
            name: "goto_window_top",
            aliases: &[],
            description: "Goto window top",
            args: &[]
        },
        Command {
            name: "goto_window_center",
            aliases: &[],
            description: "Goto window center",
            args: &[]
        },
        Command {
            name: "goto_window_bottom",
            aliases: &[],
            description: "Goto window bottom",
            args: &[]
        },
        Command {
            name: "goto_line",
            aliases: &[],
            description: "Goto line",
            args: &[]
        },
        Command {
            name: "goto_last_line",
            aliases: &[],
            description: "Goto last line",
            args: &[]
        },
        // Window
            // movement
        Command {
            name: "jump_view_right",
            aliases: &[],
            description: "Jump to right split",
            args: &[]
        },
        Command {
            name: "jump_view_left",
            aliases: &[],
            description: "Jump to left split",
            args: &[]
        },
        Command {
            name: "jump_view_up",
            aliases: &[],
            description: "Jump to split above",
            args: &[]
        },
        Command {
            name: "jump_view_down",
            aliases: &[],
            description: "Jump to split below",
            args: &[]
        },
        Command {
            name: "swap_view_right",
            aliases: &[],
            description: "Swap with right split",
            args: &[]
        },
        Command {
            name: "swap_view_left",
            aliases: &[],
            description: "Swap with left split",
            args: &[]
        },
        Command {
            name: "swap_view_up",
            aliases: &[],
            description: "Swap with split above",
            args: &[]
        },
        Command {
            name: "swap_view_down",
            aliases: &[],
            description: "Swap with split below",
            args: &[]
        },
        Command {
            name: "transpose_view",
            aliases: &[],
            description: "Transpose splits",
            args: &[]
        },
        Command {
            name: "rotate_view",
            aliases: &[],
            description: "Goto next window",
            args: &[]
        },
            // Open / Close
        Command {
            name: "hsplit",
            aliases: &[],
            description: "Horizontal bottom split",
            args: &[]
        },
        Command {
            name: "hsplit_new",
            aliases: &[],
            description: "Horizontal bottom split scratch buffer",
            args: &[]
        },
        Command {
            name: "vsplit",
            aliases: &[],
            description: "Vertical right split",
            args: &[]
        },
        Command {
            name: "vsplit_new",
            aliases: &[],
            description: "Vertical right split scratch buffer",
            args: &[]
        },
        Command {
            name: "wclose",
            aliases: &[],
            description: "Close window",
            args: &[]
        },
        Command {
            name: "wonly",
            aliases: &[],
            description: "Close windows except current",
            args: &[]
        },
        // MODE change
        Command {
            name: "command_mode",
            aliases: &[],
            description: "Enter command mode",
            args: &[]
        },
        Command {
            name: "normal_mode",
            aliases: &[],
            description: "Enter normal mode",
            args: &[]
        },
        Command {
            name: "select_mode",
            aliases: &[],
            description: "Enter selection extend mode",
            args: &[]
        },
        Command {
            name: "exit_select_mode",
            aliases: &[],
            description: "Exit selection mode",
            args: &[]
        },

        Command {
            name: "append_mode",
            aliases: &[],
            description: "Append after selection",
            args: &[]
        },
        Command {
            name: "append_at_line_end",
            aliases: &[],
            description: "Insert at end of line",
            args: &[]
        },

        Command {
            name: "insert_mode",
            aliases: &[],
            description: "Insert before selection",
            args: &[]
        },
        Command {
            name: "insert_at_line_start",
            aliases: &[],
            description: "Insert at start of line",
            args: &[]
        },
        // Register
        Command {
            name: "select_register",
            aliases: &[],
            description: "Select register",
            args: &[]
        },
        Command {
            name: "insert_register",
            aliases: &[],
            description: "Insert register",
            args: &[]
        },
        Command {
            name: "record_macro",
            aliases: &[],
            description: "Record macro",
            args: &[]
        },
        Command {
            name: "replay_macro",
            aliases: &[],
            description: "Replay macro",
            args: &[]
        },
    // Commandrow/shell
        Command {
            name: "shell_pipe",
            aliases: &[],
            description: "Pipe selections through shell command",
            args: &[]
        },
        Command {
            name: "shell_pipe_to",
            aliases: &[],
            description: "Pipe selections into shell command ignoring output",
            args: &[]
        },
        Command {
            name: "shell_keep_pipe",
            aliases: &[],
            description: "Filter selections with shell predicate",
            args: &[]
        },
        Command {
            name: "suspend",
            aliases: &[],
            description: "Suspend and return to shell",
            args: &[]
        },
            // Forwards result to server
        Command {
            name: "shell_insert_output",
            aliases: &[],
            description: "Insert shell command output before selections",
            args: &[]
        },
        Command {
            name: "shell_append_output",
            aliases: &[],
            description: "Append shell command output after selections",
            args: &[]
        },
        // File/Buffer manipulation
        Command {
            name: "goto_next_buffer",
            aliases: &[],
            description: "Goto next buffer",
            args: &[]
        },
        Command {
            name: "goto_previous_buffer",
            aliases: &[],
            description: "Goto previous buffer",
            args: &[]
        },
        Command {
            name: "goto_last_accessed_file",
            aliases: &[],
            description: "Goto last accessed file",
            args: &[]
        },
        Command {
            name: "goto_last_modified_file",
            aliases: &[],
            description: "Goto last modified file",
            args: &[]
        },
        Command {
            name: "goto_last_modification",
            aliases: &[],
            description: "Goto last modification",
            args: &[]
        },
        // UI modes
            // Pickers
        Command {
            name: "last_picker",
            aliases: &[],
            description: "Open last picker",
            args: &[]
        },
                // Jumplist
        Command {
            name: "jumplist_picker",
            aliases: &[],
            description: "Open jumplist picker",
            args: &[]
        },
                    // jumplist commands
        Command {
            name: "jump_forward",
            aliases: &[],
            description: "Jump forward on jumplist",
            args: &[]
        },
        Command {
            name: "jump_backward",
            aliases: &[],
            description: "Jump backward on jumplist",
            args: &[]
        },
        Command {
            name: "save_selection",
            aliases: &[],
            description: "Save current selection to jumplist",
            args: &[]
        },
                // File
        Command {
            name: "file_picker",
            aliases: &[],
            description: "Open file picker",
            args: &[]
        },
        Command {
            name: "file_picker_in_current_directory",
            aliases: &[],
            description: "Open file picker at current working directory",
            args: &[]
        },
                // Buffer
        Command {
            name: "buffer_picker",
            aliases: &[],
            description: "Open buffer picker",
            args: &[]
        },
                // Symbol
        Command {
            name: "symbol_picker",
            aliases: &[],
            description: "Open symbol picker",
            args: &[]
        },
        Command {
            name: "workspace_symbol_picker",
            aliases: &[],
            description: "Open workspace symbol picker",
            args: &[]
        },
                // Diagnostics
        Command {
            name: "diagnostics_picker",
            aliases: &[],
            description: "Open diagnostic picker",
            args: &[]
        },
        Command {
            name: "workspace_diagnostics_picker",
            aliases: &[],
            description: "Open workspace diagnostic picker",
            args: &[]
        },
    // Command palette
        Command {
            name: "command_palette",
            aliases: &[],
            description: "Open command palette",
            args: &[]
        },
    // Server
    // EDITS
        // Selections
        Command {
            name: "rotate_selection_contents_forward",
            aliases: &[],
            description: "Rotate selection contents forward",
            args: &[]
        },
        Command {
            name: "rotate_selection_contents_backward",
            aliases: &[],
            description: "Rotate selections contents backward",
            args: &[]
        },
        Command {
            name: "align_selections",
            aliases: &[],
            description: "Align selections in column",
            args: &[]
        },
        Command {
            name: "trim_selections",
            aliases: &[],
            description: "Trim whitespace from selections",
            args: &[]
        },
        Command {
            name: "format_selections",
            aliases: &[],
            description: "Format selection",
            args: &[]
        },
        Command {
            name: "join_selections",
            aliases: &[],
            description: "Join lines inside selection",
            args: &[]
        },
        Command {
            name: "join_selections_space",
            aliases: &[],
            description: "Join lines inside selection and select spaces",
            args: &[]
        },
        // Caseing
        Command {
            name: "switch_case",
            aliases: &[],
            description: "Switch (toggle) case",
            args: &[]
        },
        Command {
            name: "switch_to_uppercase",
            aliases: &[],
            description: "Switch to uppercase",
            args: &[]
        },
        Command {
            name: "switch_to_lowercase",
            aliases: &[],
            description: "Switch to lowercase",
            args: &[]
        },
        // Surround
        Command {
            name: "surround_add",
            aliases: &[],
            description: "Surround add",
            args: &[]
        },
        Command {
            name: "surround_replace",
            aliases: &[],
            description: "Surround replace",
            args: &[]
        },
        Command {
            name: "surround_delete",
            aliases: &[],
            description: "Surround delete",
            args: &[]
        },
        // Transaction history
        Command {
            name: "commit_undo_checkpoint",
            aliases: &[],
            description: "Commit changes to new checkpoint",
            args: &[]
        },
        Command {
            name: "undo",
            aliases: &[],
            description: "Undo change",
            args: &[]
        },
        Command {
            name: "redo",
            aliases: &[],
            description: "Redo change",
            args: &[]
        },
        Command {
            name: "earlier",
            aliases: &[],
            description: "Move backward in history",
            args: &[]
        },
        Command {
            name: "later",
            aliases: &[],
            description: "Move forward in history",
            args: &[]
        },
         // Change
        Command {
            name: "change_selection",
            aliases: &[],
            description: "Change selection",
            args: &[]
        },
        Command {
            name: "change_selection_noyank",
            aliases: &[],
            description: "Change selection without yanking",
            args: &[]
        },
        // Delete
        Command {
            name: "delete_selection",
            aliases: &[],
            description: "Delete selection",
            args: &[]
        },
        Command {
            name: "delete_selection_noyank",
            aliases: &[],
            description: "Delete selection without yanking",
            args: &[]
        },
        Command {
            name: "delete_char_backward",
            aliases: &[],
            description: "Delete previous char",
            args: &[]
        },
        Command {
            name: "delete_char_forward",
            aliases: &[],
            description: "Delete next char",
            args: &[]
        },
        Command {
            name: "delete_word_backward",
            aliases: &[],
            description: "Delete previous word",
            args: &[]
        },
        Command {
            name: "delete_word_forward",
            aliases: &[],
            description: "Delete next word",
            args: &[]
        },
        Command {
            name: "kill_to_line_start",
            aliases: &[],
            description: "Delete till start of line",
            args: &[]
        },
        Command {
            name: "kill_to_line_end",
            aliases: &[],
            description: "Delete till end of line",
            args: &[]
        },
        // Replace
        Command {
            name: "replace",
            aliases: &[],
            description: "Replace with new char",
            args: &[]
        },
        Command {
            name: "replace_with_yanked",
            aliases: &[],
            description: "Replace with yanked text",
            args: &[]
        },
        Command {
            name: "replace_selections_with_clipboard",
            aliases: &[],
            description: "Replace selections by clipboard content",
            args: &[]
        },
        Command {
            name: "replace_selections_with_primary_clipboard",
            aliases: &[],
            description: "Replace selections by primary clipboard",
            args: &[]
        },
        // Paste
        Command {
            name: "paste_after",
            aliases: &[],
            description: "Paste after selection",
            args: &[]
        },
        Command {
            name: "paste_before",
            aliases: &[],
            description: "Paste before selection",
            args: &[]
        },
        Command {
            name: "paste_clipboard_after",
            aliases: &[],
            description: "Paste clipboard after selections",
            args: &[]
        },
        Command {
            name: "paste_clipboard_before",
            aliases: &[],
            description: "Paste clipboard before selections",
            args: &[]
        },
        Command {
            name: "paste_primary_clipboard_after",
            aliases: &[],
            description: "Paste primary clipboard after selections",
            args: &[]
        },
        Command {
            name: "paste_primary_clipboard_before",
            aliases: &[],
            description: "Paste primary clipboard before selections",
            args: &[]
        },
        // Open
        Command {
            name: "open_below",
            aliases: &[],
            description: "Open new line below selection",
            args: &[]
        },
        Command {
            name: "open_above",
            aliases: &[],
            description: "Open new line above selection",
            args: &[]
        },
        // Special insert mode keybindings
        Command {
            name: "insert_tab",
            aliases: &[],
            description: "Insert tab char",
            args: &[]
        },
        Command {
            name: "insert_newline",
            aliases: &[],
            description: "Insert newline char",
            args: &[]
        },
        // *crement
        Command {
            name: "increment",
            aliases: &[],
            description: "Increment item under cursor",
            args: &[]
        },
        Command {
            name: "decrement",
            aliases: &[],
            description: "Decrement item under cursor",
            args: &[]
        },
        // Indent
        Command {
            name: "indent",
            aliases: &[],
            description: "Indent selection",
            args: &[]
        },
        Command {
            name: "unindent",
            aliases: &[],
            description: "Unindent selection",
            args: &[]
        },
        // Lineadds
        Command {
            name: "add_newline_above",
            aliases: &[],
            description: "Add newline above",
            args: &[]
        },
        Command {
            name: "add_newline_below",
            aliases: &[],
            description: "Add newline below",
            args: &[]
        },
        // Lang
        Command {
            name: "toggle_comments",
            aliases: &[],
            description: "Comment/uncomment selections",
            args: &[]
        },
        // LSP
        Command {
            name: "code_action",
            aliases: &[],
            description: "Perform code action",
            args: &[]
        },
        Command {
            name: "goto_definition",
            aliases: &[],
            description: "Goto definition",
            args: &[]
        },
        Command {
            name: "goto_implementation",
            aliases: &[],
            description: "Goto implimentation",
            args: &[]
        },
        Command {
            name: "goto_type_definition",
            aliases: &[],
            description: "Goto type definition",
            args: &[]
        },
        Command {
            name: "goto_last_modification",
            aliases: &[],
            description: "Goto last modification",
            args: &[]
        },
        Command {
            name: "goto_reference",
            aliases: &[],
            description: "Goto references",
            args: &[]
        },
        Command {
            name: "goto_first_diag",
            aliases: &[],
            description: "Goto first diagnostic",
            args: &[]
        },
        Command {
            name: "goto_last_diag",
            aliases: &[],
            description: "Goto last diagnostic",
            args: &[]
        },
        Command {
            name: "goto_next_diag",
            aliases: &[],
            description: "Goto next diagnostic",
            args: &[]
        },
        Command {
            name: "goto_prev_diag",
            aliases: &[],
            description: "Goto previous diagnostic",
            args: &[]
        },
        Command {
            name: "signature_help",
            aliases: &[],
            description: "Show signature help",
            args: &[]
        },
        Command {
            name: "completion",
            aliases: &[],
            description: "Invoke completion popup",
            args: &[]
        },
        Command {
            name: "hover",
            aliases: &[],
            description: "Show docs for item under cursor",
            args: &[]
        },
        Command {
            name: "rename_symbol",
            aliases: &[],
            description: "Rename symbol",
            args: &[]
        },
        // DAP
        Command {
            name: "dap_launch",
            aliases: &[],
            description: "Launch debug target",
            args: &[]
        },
        Command {
            name: "dap_toggle_breakpoint",
            aliases: &[],
            description: "Toggle breakpoint",
            args: &[]
        },
        Command {
            name: "dap_continue",
            aliases: &[],
            description: "Continue program execution",
            args: &[]
        },
        Command {
            name: "dap_pause",
            aliases: &[],
            description: "Pause program execution",
            args: &[]
        },
        Command {
            name: "dap_step_in",
            aliases: &[],
            description: "Step in",
            args: &[]
        },
        Command {
            name: "dap_step_out",
            aliases: &[],
            description: "Step out",
            args: &[]
        },
        Command {
            name: "dap_next",
            aliases: &[],
            description: "Step to next",
            args: &[]
        },
        Command {
            name: "dap_variables",
            aliases: &[],
            description: "List variables",
            args: &[]
        },
        Command {
            name: "dap_terminate",
            aliases: &[],
            description: "End debug session",
            args: &[]
        },
        Command {
            name: "dap_edit_condition",
            aliases: &[],
            description: "Edit breakpoint condition on current line",
            args: &[]
        },
        Command {
            name: "dap_edit_log",
            aliases: &[],
            description: "Edit breakpoint log message on current line",
            args: &[]
        },
        Command {
            name: "dap_switch_thread",
            aliases: &[],
            description: "Switch current thread",
            args: &[]
        },
        Command {
            name: "dap_switch_stack_frame",
            aliases: &[],
            description: "Switch stack frame",
            args: &[]
        },
        Command {
            name: "dap_enable_exceptions",
            aliases: &[],
            description: "Enable exception breakpoints",
            args: &[]
        },
        Command {
            name: "dap_disable_exceptions",
            aliases: &[],
            description: "Disable exception breakpoints",
            args: &[]
        },
        // VCS
        Command {
            name: "goto_next_change",
            aliases: &[],
            description: "Goto next change",
            args: &[]
        },
        Command {
            name: "goto_prev_change",
            aliases: &[],
            description: "Goto previous change",
            args: &[]
        },
        Command {
            name: "goto_first_change",
            aliases: &[],
            description: "Goto first change",
            args: &[]
        },
        Command {
            name: "goto_last_change",
            aliases: &[],
            description: "Goto last change",
            args: &[]
        },
        // File open from selection
        Command {
            name: "goto_file",
            aliases: &[],
            description: "Goto files in selection",
            args: &[]
        },
        Command {
            name: "goto_file_hsplit",
            aliases: &[],
            description: "Goto files in selection (hsplit)",
            args: &[]
        },
        Command {
            name: "goto_file_vsplit",
            aliases: &[],
            description: "Goto files in selection (vsplit)",
            args: &[]
        }
];

