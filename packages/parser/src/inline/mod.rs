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
    for node in nodes {
        match node.kind {
            NodeKind::Raw(text) => resolved.extend(parse_raw(&text, ancestors)),
            _ => resolved.push(resolve_node(node, ancestors)),
        }
    }
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
