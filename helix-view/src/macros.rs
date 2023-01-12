//! These are macros to make getting very nested fields in the `UITree` struct easier
//! These are macros instead of functions because functions will have to take `&mut self`
//! However, rust doesn't know that you only want a partial borrow instead of borrowing the
//! entire struct which `&mut self` says.  This makes it impossible to do other mutable
//! stuff to the struct because it is already borrowed. Because macros are expanded,
//! this circumvents the problem because it is just like indexing fields by hand and then
//! putting a `&mut` in front of it. This way rust can see that we are only borrowing a
//! part of the struct and not the entire thing.

/// Get the currently focused buffer_mirror as mutable.
/// Returns `&mut BufferMirror`
#[macro_export]
macro_rules! current_mut {
    ($ui_tree:expr) => {{
        let buffer_view = $crate::buffer_view_mut!($ui_tree);
        let buffer = $crate::buffer_mut!($ui_tree, &buffer_view.buffer_id);
        (buffer_view, buffer)
    }};
}

/// Get the currently focused buffer_mirror.
/// Returns `&mut BufferMirror`
#[macro_export]
macro_rules! current {
    ($ui_tree:expr) => {{
        &$ui_tree.buffers[&$ui_tree.tree.get($ui_tree.tree.focus).buffer_id];
    }};
}

/// Get the current buffer view mutably.
/// Returns `&mut BufferView`
#[macro_export]
macro_rules! buffer_view_mut {
    ($ui_tree:expr, $id:expr) => {{
        $ui_tree.tree.get_mut($id)
    }};
    ($ui_tree:expr) => {{
        $ui_tree.tree.get_mut($ui_tree.tree.focus)
    }};
}

/// Get the current buffer view immutably
/// Returns `&BufferView`
#[macro_export]
macro_rules! buffer_view {
    ($ui_tree:expr, $id:expr) => {{
        $ui_tree.tree.get($id)
    }};
    ($ui_tree:expr) => {{
        $ui_tree.tree.get($ui_tree.tree.focus)
    }};
}

