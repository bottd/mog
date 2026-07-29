use crate::{Delimiter, Node, NodeKind};

use self::raw::parse_raw;
use self::table::resolve_table;

mod parser;
mod raw;
mod table;

pub fn parse_inlines(nodes: Vec<Node>) -> Vec<Node> {
    resolve_nodes(nodes, &[])
}

fn resolve_nodes(nodes: Vec<Node>, ancestors: &[Delimiter]) -> Vec<Node> {
    let mut resolved = Vec::with_capacity(nodes.len());
    nodes
        .into_iter()
        .flat_map(|node| match node.kind {
            NodeKind::Raw(text) => parse_raw(&text, ancestors),
            _ => vec![resolve_node(node, ancestors)],
        })
        .for_each(|node| push_resolved(&mut resolved, node));

    resolved
}

fn resolve_node(node: Node, ancestors: &[Delimiter]) -> Node {
    if let Some(data) = node.get_verbatim_data(crate::node::DataArgument::Data) {
        return Node {
            kind: NodeKind::Data(data),
            children: None,
            ..node
        };
    }

    let Node {
        kind,
        attributes,
        children,
    } = node;

    let children = match &kind {
        NodeKind::Table => children.map(|rows| resolve_table(rows, ancestors)),
        NodeKind::Delimiter(Delimiter::Verbatim) => children,
        NodeKind::Delimiter(delimiter) => {
            let ancestors = [ancestors, &[*delimiter]].concat();
            children.map(|nodes| resolve_nodes(nodes, &ancestors))
        }
        _ => children.map(|nodes| resolve_nodes(nodes, ancestors)),
    };

    Node {
        kind,
        attributes,
        children,
    }
}

fn push_resolved(resolved: &mut Vec<Node>, node: Node) {
    // TODO: footnotes to document level
    // LinkName and Footnote can attach to a preceeding resolved node
    let attaches = matches!(
        (resolved.last().map(|last| &last.kind), &node.kind),
        (
            Some(NodeKind::Delimiter(Delimiter::Link)),
            NodeKind::Delimiter(Delimiter::LinkName | Delimiter::Footnote)
        ) | (
            Some(NodeKind::Delimiter(Delimiter::LinkName)),
            NodeKind::Delimiter(Delimiter::Footnote)
        )
    );

    if attaches {
        resolved.last_mut().expect("checked above").push_child(node);
    } else {
        resolved.push(node);
    }
}
