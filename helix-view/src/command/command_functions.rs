use helix_core::movement::Movement;

use crate::{
    ui_tree::UITree,
    command::Command
};

fn no_op(command: &Command, ui_tree: &UITree) {}

fn move_line_start(command: &Command, ui_tree: &UITree) {
    goto_line_start_impl(ui_tree, Movement::Move)
}

fn extend_to_line_start(command: &Command, ui_tree: &UITree) {
    goto_line_start_impl(ui_tree, Movement::Extend)
}

fn goto_line_start_impl(ui_tree: &UITree, movement: Movement) {
    let (buffer_view, buffer) = current!(ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.buffer_id).clone().transform(|range| {
        let line = range.cursor_line(text);

        // adjust to start of the line
        let pos = text.line_to_char(line);
        range.put_cursor(text, pos, movement == Movement::Extend)
    });
    buffer.set_selection(buffer_view.buffer_id, selection);
}
