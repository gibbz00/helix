#[macro_use]
pub mod macros;

pub mod clipboard;
pub mod buffer_mirror;
pub mod ui_tree;
pub mod env;
pub mod graphics;
pub mod gutter;
pub mod handlers {
    pub mod dap;
    pub mod lsp;
}
pub mod base64;
pub mod info;
pub mod input;
pub mod keyboard;
pub mod theme;
pub mod tree;
pub mod buffer_view;
pub mod config;
pub mod event_handler;
pub mod mode;
mod keymap;
mod command;
mod lists;
mod jump;

pub use buffer_mirror::BufferMirror;
pub use ui_tree::UITree;
pub use theme::Theme;
pub use buffer_view::BufferView;
use std::num::NonZeroUsize;

slotmap::new_key_type! {
    pub struct BufferViewID;
}

pub enum Align {
    Top,
    Center,
    Bottom,
}

pub fn align_view(buffer: &BufferMirror, buffer_view: &mut BufferView, align: Align) {
    let pos = buffer
        .selection(buffer_view.view_id)
        .primary()
        .cursor(buffer.text().slice(..));
    let line = buffer.text().char_to_line(pos);

    let last_line_height = buffer_view.inner_height().saturating_sub(1);

    let relative = match align {
        Align::Center => last_line_height / 2,
        Align::Top => 0,
        Align::Bottom => last_line_height,
    };

    buffer_view.offset.row = line.saturating_sub(relative);
}

/// Applies a [`helix_core::Transaction`] to the given [`Buffer`]
/// and [`BufferView`].
pub fn apply_transaction(
    transaction: &helix_core::Transaction,
    buffer: &mut BufferMirror,
    buffer_view: &BufferView,
) -> bool {
    // TODO remove this helper function. Just call Buffer::apply everywhere directly.
    buffer.apply(transaction, buffer_view.view_id)
}

