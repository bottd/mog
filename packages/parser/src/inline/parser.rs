use crate::{whitespace, whitespace::WHITESPACE};

use super::{content, push_resolved};
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

    // an attribute node takes the whitespace separating it from the content,
    // but only at the edges of its parent: inside, the space is prose
    pub fn absorb_separator(&mut self, input: &mut &str) {
        let open = self.stack.last();
        let siblings = open.map_or(&self.resolved, |open| &open.children);
        let rest = whitespace::trim_start(input);

        let kept = whitespace::trim_end(&self.buffer).len();

        let leading = kept == 0 && siblings.iter().all(Node::is_attributes);
        let trailing = rest.is_empty()
            || open.is_some_and(|open| {
                matches!(open.kind, NodeKind::Delimiter(delimiter) if delimiter.closes(rest))
            });

        if leading || trailing {
            self.buffer.truncate(kept);
            *input = rest;
        }
    }

    pub fn push_node(&mut self, node: Node) {
        let siblings = match self.stack.last_mut() {
            Some(open) => &mut open.children,
            None => &mut self.resolved,
        };

        push_resolved(siblings, node);
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
        let Some(mut node) = self.stack.pop() else {
            return;
        };

        trim_node(&mut node);
        self.push_node(node);
    }
}

fn trim_node(node: &mut Node) {
    // at most one space, so the delimiters can be written apart from the text
    if let Some(NodeKind::Text(text)) = content(node.children.iter_mut()).map(|edge| &mut edge.kind)
        && text.starts_with(WHITESPACE)
    {
        text.drain(..1);
    }

    if let Some(NodeKind::Text(text)) =
        content(node.children.iter_mut().rev()).map(|edge| &mut edge.kind)
        && text.ends_with(WHITESPACE)
    {
        text.pop();
    }
}
