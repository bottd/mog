use crate::{Delimiter, whitespace};
use crate::{Node, NodeKind};

#[derive(Default)]
pub(super) struct Parser {
    resolved: Vec<Node>,
    pub(super) stack: Vec<Node>,
    pub(super) buffer: String,
}

impl Parser {
    pub fn collect(mut self) -> Vec<Node> {
        while !self.stack.is_empty() {
            self.close_node();
        }
        self.flush();
        self.resolved
    }

    pub fn push_node(&mut self, node: Node) {
        let siblings = match self.stack.last_mut().or_else(|| self.resolved.last_mut()) {
            Some(open) => open.children.get_or_insert_default(),
            None => &mut self.resolved,
        };

        let attaches = matches!(
            (siblings.last().map(|last| &last.kind), &node.kind),
            (
                Some(NodeKind::Link(_)),
                NodeKind::Delimiter(Delimiter::LinkName | Delimiter::Footnote)
            ) | (
                Some(NodeKind::Delimiter(Delimiter::LinkName)),
                NodeKind::Delimiter(Delimiter::Footnote)
            )
        );

        // TODO: footnotes to document level
        // LinkName and Footnote can attach to a preceding sibling
        if attaches {
            siblings.last_mut().expect("checked above").push_child(node);
        } else {
            siblings.push(node);
        }
    }

    pub fn push_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let text = std::mem::take(&mut self.buffer);
            self.push_node(Node::empty(NodeKind::Text(text)));
        }
    }

    pub(super) fn close_node(&mut self) {
        self.flush();
        let mut node = self.stack.pop().expect("open node");
        trim_node(&mut node);
        self.push_node(node);
    }
}

fn trim_node(node: &mut Node) {
    let Some(children) = node.children.as_mut() else {
        return;
    };

    if let Some(NodeKind::Text(text)) = children.first_mut().map(|child| &mut child.kind) {
        text.drain(..whitespace::indent(text).min(1));
    }

    if let Some(NodeKind::Text(text)) = children.last_mut().map(|child| &mut child.kind) {
        let trailing = text.len() - whitespace::trim_end(text).len();
        text.truncate(text.len() - trailing.min(1));
    }
}
