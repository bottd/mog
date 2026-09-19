use serde::Serialize;

use self::attribute::AttributesExt;
use self::delimiter::Delimiter;
use self::marker::Marker;
use crate::span::Span;

pub mod attribute;
pub mod delimiter;
pub mod marker;

#[derive(Debug, Serialize, PartialEq)]
pub enum NodeKind {
    Marker(Marker),
    Delimiter(Delimiter),
    Raw(String),
    Link(String),
    Attributes,
    Paragraph,
    Table,
    Text(String),
}

/// Spans take part in equality: two nodes with the same shape in different
/// places are not equal. Snapshot tests downstream depend on knowing which it is.
#[derive(Debug, Serialize, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: attribute::Owned,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
    /// Where the node is in the source, from its opening syntax through the
    /// end of its last line. Every block-level node carries one. Inline nodes
    /// do not: the inline pass reads text that has already been dedented,
    /// unescaped and rejoined, so any offset it reported would be a guess.
    ///
    /// Positions live on `Node` rather than inside [`NodeKind`] on purpose —
    /// the block parser closes a block by comparing its kind to the closing
    /// line's, and a position in that comparison would stop every block from
    /// ever closing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// A marker's opening fence line alone, where that differs from `span`.
    /// "Insert directly after the fence" is the one position an editor needs
    /// that is not a node boundary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fence: Option<Span>,
}

impl Node {
    pub(crate) fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }

    pub(crate) fn raw(text: impl Into<String>) -> Node {
        Node::empty(NodeKind::Raw(text.into()))
    }

    // the parsed body of an ``attr: block
    pub(crate) fn attributes(children: Vec<attribute::Attribute>) -> Node {
        let mut attributes = None;
        attributes.push_children(children, None);

        Node::new(NodeKind::Attributes, attributes)
    }

    pub(crate) fn is_attributes(&self) -> bool {
        self.kind == NodeKind::Attributes
    }

    pub(crate) fn empty(kind: NodeKind) -> Node {
        Node::new(kind, None)
    }

    pub(crate) fn new(kind: NodeKind, attributes: attribute::Owned) -> Node {
        Node {
            kind,
            attributes,
            children: Vec::new(),
            span: None,
            fence: None,
        }
    }

    /// The block pass calls this on every node it builds from a source line.
    pub(crate) fn at(mut self, span: Span) -> Node {
        self.span = Some(span);
        self
    }

    /// Grows this node's span to end where `closing` does. A node without a
    /// span stays without one.
    pub(crate) fn extend_to(&mut self, closing: Option<Span>) {
        if let (Some(span), Some(closing)) = (self.span, closing) {
            self.span = Some(span.extend(closing));
        }
    }

    /// Where this `attr` block is, if it is one a tool can rewrite.
    ///
    /// Only the block pass records spans, so a span on an attribute node means
    /// the block was written on its own lines. One written inside a line has
    /// none, and splicing a whole line in its place would delete the line.
    /// Stated once here because folding and diagnostics both rely on it.
    pub(crate) fn attribute_block_span(&self) -> Option<Span> {
        self.is_attributes().then_some(self.span).flatten()
    }

    // a single raw child, or none when the text is empty
    pub(crate) fn raw_children(text: &str) -> Vec<Node> {
        Vec::from_iter((!text.is_empty()).then(|| Node::raw(text)))
    }

    /// Literal content of a verbatim node, joined with source line separators.
    pub fn raw_text(&self) -> String {
        self.children
            .iter()
            .filter_map(|child| match &child.kind {
                NodeKind::Raw(text) => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
