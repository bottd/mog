use crate::node::attribute::{AttributesExt, parse_attribute_block};
use crate::span::Span;
use crate::{Node, NodeKind, whitespace};

use super::line::{Line, classify, is_verbatim_close};

#[derive(Default)]
pub(super) struct Parser {
    resolved: Vec<Node>,
    stack: Vec<Node>,
    accumulator: Option<Accumulator>,
    open: Option<Open>,
}

enum Accumulator {
    // carries attribute nodes so an ``attr: block can interrupt a paragraph
    // the same way it interrupts a table
    Paragraph {
        text: String,
        attributes: Vec<Node>,
        span: Span,
    },
    Table(Node),
    // a single-line marker, such as a heading or list item, held for a line
    // so an ``attr: block directly below it can attach
    Resolved(Node),
}

struct Open {
    node: Node,
    indent: usize,
}

impl Parser {
    pub(super) fn feed(&mut self, line: &str, span: Span) {
        if let Some(Open { node, indent }) = &mut self.open {
            // TODO: node.attributes.is_some()
            // error, unclosed verbatim or attribute on closing verbatim
            if is_verbatim_close(line) {
                return self.close_open(span);
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
        // A span begins at the node's own syntax, so indentation is not part
        // of it. The end stays where the line does, trailing spaces included.
        let span = Span {
            start: span.start + whitespace::indent(line),
            ..span
        };

        match classify(trimmed, span) {
            // Paragraph and Table end on empty line
            Line::Blank => self.collect_accumulator(),
            Line::Resolved(node) => {
                self.collect_accumulator();
                self.accumulator = Some(Accumulator::Resolved(node));
            }
            Line::Verbatim(node) => self.open_block(node, line),
            Line::Block(node) => {
                self.collect_accumulator();
                self.push_stack(node, trimmed, span);
            }
            Line::Row(node) => self.row_line(node),
            Line::Text => self.text_line(trimmed, span),
        }
    }

    /// `last` is the final line of the document — where anything still open ends.
    pub(super) fn close(mut self, last: Option<Span>) -> Vec<Node> {
        self.collect_accumulator();

        // TODO: error, unclosed verbatim
        // an unclosed ``attr: block stays verbatim: prose lines are valid KDL
        // nodes, so consuming it would swallow the rest of the document
        if let Some(Open { mut node, .. }) = self.open.take() {
            node.extend_to(last);
            self.push_resolved_node(node);
        }

        while let Some(mut open) = self.stack.pop() {
            open.extend_to(last);
            self.push_resolved_node(open);
        }

        self.resolved
    }

    fn text_line(&mut self, line: &str, span: Span) {
        match &mut self.accumulator {
            Some(Accumulator::Paragraph {
                text,
                span: paragraph,
                ..
            }) => {
                text.push(' ');
                text.push_str(line);
                *paragraph = paragraph.extend(span);
            }
            _ => {
                self.collect_accumulator();
                self.accumulator = Some(Accumulator::Paragraph {
                    text: String::from(line),
                    attributes: Vec::new(),
                    span,
                });
            }
        }
    }

    fn row_line(&mut self, node: Node) {
        match &mut self.accumulator {
            Some(Accumulator::Table(table)) => {
                table.extend_to(node.span);
                table.push_child(node);
            }
            _ => {
                self.collect_accumulator();
                // the table starts where its first row does
                let mut table = Node::empty(NodeKind::Table);
                table.span = node.span;
                table.push_child(node);
                self.accumulator = Some(Accumulator::Table(table));
            }
        }
    }

    fn open_block(&mut self, node: Node, line: &str) {
        // the accumulator parents an ``attr: block that interrupts it, so it
        // stays open across the block
        if !node.attributes.is_attribute_chain() {
            self.collect_accumulator();
        }

        self.open = Some(Open {
            node,
            indent: whitespace::indent(line),
        });
    }

    // an ``attr: block resolves to an attribute node rather than a verbatim
    // block; a body that is not valid KDL degrades to verbatim
    fn close_open(&mut self, closing: Span) {
        let Some(Open { mut node, .. }) = self.open.take() else {
            return;
        };

        // opening fence line through closing fence line, inclusive
        node.extend_to(Some(closing));

        // the text is only collected once the chain marks the block
        let attributes = node
            .attributes
            .is_attribute_chain()
            .then(|| parse_attribute_block(&node.raw_text()).ok())
            .flatten();

        let Some(attributes) = attributes else {
            // an accumulator the block interrupted is flushed ahead of it
            self.collect_accumulator();
            return self.push_resolved_node(node);
        };

        let mut block = Node::attributes(attributes);
        // A span is what marks this as a block-form ``attr: block. The inline
        // pass mints attribute nodes too, and those stay span-less, which is
        // how folding tells a spliceable block from one written inside a line.
        block.span = node.span;
        self.push_attribute_node(block);
    }

    // the construct directly above an ``attr: block owns it: the accumulator,
    // else the innermost open block; with neither the document owns it
    fn push_attribute_node(&mut self, node: Node) {
        // The owner grows over the block: a block in the middle of a paragraph
        // is covered anyway by the text after it, so covering a trailing one
        // too is what makes the rule the same either way. `fence` still gives
        // the owner's own first line.
        match &mut self.accumulator {
            Some(Accumulator::Paragraph {
                attributes, span, ..
            }) => {
                if let Some(block) = node.span {
                    *span = span.extend(block);
                }
                attributes.push(node);
            }
            Some(Accumulator::Table(parent) | Accumulator::Resolved(parent)) => {
                // The marker was written on one line and is about to stop being
                // one line, so it needs its own line recorded — that is what
                // `fence` is for, and it is the position an editor inserts at.
                if parent.fence.is_none() && matches!(parent.kind, NodeKind::Marker(_)) {
                    parent.fence = parent.span;
                }
                parent.extend_to(node.span);
                parent.push_child(node);
            }
            None => self.push_resolved_node(node),
        }
    }

    fn collect_accumulator(&mut self) {
        let node = match self.accumulator.take() {
            None => return,
            Some(Accumulator::Table(node) | Accumulator::Resolved(node)) => node,
            Some(Accumulator::Paragraph {
                text,
                attributes,
                span,
            }) => {
                let mut node = Node::empty(NodeKind::Paragraph).at(span);
                node.push_child(Node::raw(text));
                node.children.extend(attributes);
                node
            }
        };

        self.push_resolved_node(node);
    }

    fn push_resolved_node(&mut self, node: Node) {
        match self.stack.last_mut() {
            Some(open) => open.push_child(node),
            None => self.resolved.push(node),
        }
    }

    fn push_stack(&mut self, node: Node, line: &str, span: Span) {
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
            false => self.text_line(line, span),
        }
    }

    fn pop_stack(&mut self, node: &Node) -> bool {
        // TODO: node.attributes.is_some()
        // error, unclosed block or attribute on closing block

        // TODO: !node.children.is_empty()
        // error, unclosed block
        let matching = |open: &mut Node| node.attributes.is_none() && open.kind == node.kind;

        let Some(mut closed) = self.stack.pop_if(matching) else {
            return false;
        };

        // the closing fence line ends the block; `fence` keeps the opening one
        closed.extend_to(node.span);
        self.push_resolved_node(closed);
        true
    }
}
