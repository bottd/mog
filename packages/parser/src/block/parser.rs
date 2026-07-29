use crate::{Delimiter, Node, NodeKind, whitespace};

use super::line::{Line, classify};

#[derive(Default)]
pub(super) struct Parser {
    resolved: Vec<Node>,
    stack: Vec<Node>,
    accumulator: Option<Accumulator>,
}

enum Accumulator {
    Paragraph(String),
    Table(Vec<Node>),
    // verbatim carries usize to preserve indentation
    Verbatim(Node, usize),
}

impl Parser {
    pub(super) fn feed(&mut self, line: &str) {
        if let Some(Accumulator::Verbatim(node, indent)) = &mut self.accumulator {
            // TODO: node.attributes.is_some()
            // error, unclosed verbatim or attribute on closing verbatim
            if whitespace::trim(line).as_bytes() == Delimiter::Verbatim.opening() {
                return self.collect_accumulator();
            }

            let content = whitespace::dedent(line, *indent);
            let content = match whitespace::trim_start(line).starts_with(r"\``") {
                true => content.replacen('\\', "", 1),
                false => content.to_owned(),
            };
            node.push_child(Node::raw(content));
            return;
        }

        let trimmed = whitespace::trim(line);

        // Paragraph and Table end on empty line
        if trimmed.is_empty() {
            return self.collect_accumulator();
        }

        match classify(trimmed) {
            Line::Resolved(node) => {
                self.collect_accumulator();
                self.push_resolved_node(node);
            }
            Line::Block(node) if node.kind == NodeKind::Delimiter(Delimiter::Verbatim) => {
                self.collect_accumulator();
                self.accumulator = Some(Accumulator::Verbatim(node, whitespace::indent(line)));
            }
            Line::Block(node) => {
                self.collect_accumulator();
                self.push_stack(node, trimmed);
            }
            Line::Row(node) => match &mut self.accumulator {
                Some(Accumulator::Table(rows)) => rows.push(node),
                _ => {
                    self.collect_accumulator();
                    self.accumulator = Some(Accumulator::Table(vec![node]));
                }
            },
            Line::Text => self.text_line(trimmed),
        }
    }

    pub(super) fn close(mut self) -> Vec<Node> {
        // TODO: !node.children.is_empty()
        // error, unclosed verbatim
        self.collect_accumulator();

        while let Some(open) = self.stack.pop() {
            self.push_resolved_node(open);
        }

        self.resolved
    }

    fn text_line(&mut self, line: &str) {
        match &mut self.accumulator {
            Some(Accumulator::Paragraph(paragraph)) => {
                paragraph.push(' ');
                paragraph.push_str(line);
            }
            _ => {
                self.collect_accumulator();
                self.accumulator = Some(Accumulator::Paragraph(String::from(line)));
            }
        }
    }

    fn collect_accumulator(&mut self) {
        let Some(accumulator) = self.accumulator.take() else {
            return;
        };

        let node = match accumulator {
            Accumulator::Paragraph(text) => Node::raw(text),
            Accumulator::Table(rows) => Node {
                kind: NodeKind::Table,
                attributes: None,
                children: Some(rows),
            },
            Accumulator::Verbatim(node, _) => node,
        };

        self.push_resolved_node(node);
    }

    fn push_resolved_node(&mut self, node: Node) {
        match self.stack.last_mut() {
            Some(open) => open.push_child(node),
            None => self.resolved.push(node),
        }
    }

    fn push_stack(&mut self, node: Node, line: &str) {
        let (is_opener, is_closer) = match &node.kind {
            NodeKind::Delimiter(delimiter) => {
                let closer = delimiter
                    .closing()
                    .is_some_and(|closing| line.as_bytes().starts_with(&closing));
                (!closer || delimiter.is_symmetric(), closer)
            }
            _ => (true, true),
        };

        if is_closer && self.pop_stack(&node) {
            return;
        }

        match is_opener {
            true => self.stack.push(node),
            false => self.text_line(line),
        }
    }

    fn pop_stack(&mut self, node: &Node) -> bool {
        match self.stack.last() {
            Some(open) if node.attributes.is_none() && open.kind == node.kind => {
                // TODO: node.attributes.is_some()
                // error, unclosed block or attribute on closing block

                // TODO: node.children.is_some()
                // error, unclosed block
                let closed = self.stack.pop().expect("matching node");
                self.push_resolved_node(closed);
                true
            }
            _ => false,
        }
    }
}
