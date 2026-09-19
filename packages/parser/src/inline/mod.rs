use crate::{Delimiter, Delimiters, Node, NodeKind};

use self::raw::parse_raw;
use self::table::resolve_table;

mod parser;
mod raw;
mod table;

pub(crate) fn resolve_nodes(nodes: Vec<Node>, ancestors: Delimiters) -> Vec<Node> {
    let mut resolved = Vec::new();

    for node in nodes {
        match node.kind {
            NodeKind::Raw(text) => {
                let parsed = parse_raw(&text, ancestors);

                if resolved.is_empty() {
                    resolved = parsed;
                } else {
                    for node in parsed {
                        push_resolved(&mut resolved, node);
                    }
                }
            }
            _ => push_resolved(&mut resolved, resolve_node(node, ancestors)),
        }
    }

    resolved
}

fn resolve_node(mut node: Node, ancestors: Delimiters) -> Node {
    let children = std::mem::take(&mut node.children);

    node.children = match &node.kind {
        NodeKind::Table => resolve_table(children, ancestors),
        NodeKind::Delimiter(Delimiter::Verbatim) => children,
        NodeKind::Delimiter(delimiter) => resolve_nodes(children, ancestors.with(*delimiter)),
        _ => resolve_nodes(children, ancestors),
    };

    node
}

// TODO: footnotes to document level
// LinkName and Footnote can attach to a preceding sibling; a Link is already
// rewritten to NodeKind::Link by the time it is pushed, so both spellings
// count
fn attaches(previous: &NodeKind, node: &NodeKind) -> bool {
    matches!(
        (previous, node),
        (
            NodeKind::Link(_) | NodeKind::Delimiter(Delimiter::Link),
            NodeKind::Delimiter(Delimiter::LinkName | Delimiter::Footnote)
        ) | (
            NodeKind::Delimiter(Delimiter::LinkName),
            NodeKind::Delimiter(Delimiter::Footnote)
        )
    )
}

// attribute nodes describe the parent, so they are skipped over
fn content<'a>(mut nodes: impl Iterator<Item = &'a mut Node>) -> Option<&'a mut Node> {
    nodes.find(|node| !node.is_attributes())
}

fn push_resolved(resolved: &mut Vec<Node>, node: Node) {
    match content(resolved.iter_mut().rev()) {
        Some(last) if attaches(&last.kind, &node.kind) => last.push_child(node),
        _ => resolved.push(node),
    }
}
