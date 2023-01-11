fn no_op(command: &Command, ui_tree: &UITree) {}

fn move_line_start(command: &Command, ui_tree: &UITree) {
    goto_line_start_impl(Movement::Move)
}

fn extend_to_line_start(command: &Command, ui_tree: &UITree) {
    goto_line_start_impl(Movement::Extend)
}

fn goto_line_start_impl(movement: Movement) {
    let (buffer_view, buffer) = current!(cx.ui_tree);
    let text = buffer.text().slice(..);

    let selection = buffer.selection(buffer_view.id).clone().transform(|range| {
        let line = range.cursor_line(text);

        // adjust to start of the line
        let pos = text.line_to_char(line);
        range.put_cursor(text, pos, movement == Movement::Extend)
    });
    buffer.set_selection(buffer_view.id, selection);
}
fn no_op(command: &Command, ui_tree: &UITree) {}
fn no_op(command: &Command, ui_tree: &UITree) {}
fn no_op(command: &Command, ui_tree: &UITree) {}
fn no_op(command: &Command, ui_tree: &UITree) {}
fn no_op(command: &Command, ui_tree: &UITree) {}
fn no_op(command: &Command, ui_tree: &UITree) {}
