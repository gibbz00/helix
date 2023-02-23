use crate::{
    selection::{FindSyntaxNode, SelectionRule},
    Range, RopeSlice, Selection, Syntax,
};
use tree_sitter::Node;

pub fn select_sibling<F>(
    syntax: &Syntax,
    text: RopeSlice,
    selection: Selection,
    sibling_fn: &F,
) -> Selection
where
    F: Fn(Node) -> Option<Node>,
{
    select_node_impl(syntax, text, selection, |descendant, _from, _to| {
        find_sibling_recursive(descendant, sibling_fn)
    })
}

fn find_sibling_recursive<F>(node: Node, sibling_fn: F) -> Option<Node>
where
    F: Fn(Node) -> Option<Node>,
{
    sibling_fn(node).or_else(|| {
        node.parent()
            .and_then(|node| find_sibling_recursive(node, sibling_fn))
    })
}

fn select_node_impl<F>(
    syntax: &Syntax,
    text: RopeSlice,
    selection: Selection,
    select_fn: F,
) -> Selection
where
    F: Fn(Node, usize, usize) -> Option<Node>,
{
    // TODO: send in syntaxt tree to strategy_parameters?
    let tree = syntax.tree();

    selection.transform(|range| {
        let from = text.char_to_byte(range.from());
        let to = text.char_to_byte(range.to());

        let node = match tree
            .root_node()
            .descendant_for_byte_range(from, to)
            .and_then(|node| select_fn(node, from, to))
        {
            Some(node) => node,
            None => return range,
        };

        let from = text.byte_to_char(node.start_byte());
        let to = text.byte_to_char(node.end_byte());

        if range.head < range.anchor {
            Range::new(to, from)
        } else {
            Range::new(from, to)
        }
    })
}

#[allow(non_upper_case_globals)]
pub const parent_node: FindSyntaxNode = |mut node, from, to| {
    while node.start_byte() == from && node.end_byte() == to {
        node = node.parent()?;
    }
    Some(node)
};

#[allow(non_upper_case_globals)]
pub const first_child: FindSyntaxNode = |node, _from, _to| node.child(0).or(Some(node));

#[allow(non_upper_case_globals)]
pub const select_next_sibling: FindSyntaxNode =
    |descendant, _to, _from| find_sibling_recursive(descendant, |node| Node::next_sibling(&node));

#[allow(non_upper_case_globals)]
pub const select_prev_sibling: FindSyntaxNode =
    |descendant, _to, _from| find_sibling_recursive(descendant, |node| Node::prev_sibling(&node));

#[allow(non_upper_case_globals)]
pub const select_from_node: SelectionRule = |range, rule_cx| {
    let text = rule_cx.text;
    let from = text.char_to_byte(range.from());
    let to = text.char_to_byte(range.to());

    let tree = rule_cx
        .syntax
        .expect("syntax existence should be part of the run_condition")
        .tree();
    let find_fn = rule_cx
        .syntax_find_node_fn
        .expect("select_node must be called with a syntax_find_node_fn in the rule_cx");

    let node = match tree
        .root_node()
        .descendant_for_byte_range(from, to)
        .and_then(|node| (find_fn)(node, from, to))
    {
        Some(node) => node,
        None => return range,
    };

    let from = text.byte_to_char(node.start_byte());
    let to = text.byte_to_char(node.end_byte());

    if range.head < range.anchor {
        Range::new(to, from)
    } else {
        Range::new(from, to)
    }
};
