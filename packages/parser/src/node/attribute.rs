use kdl::{KdlDocument, KdlEntry, KdlError, KdlNode, KdlValue};
use serde::Serialize;

use crate::span::Span;
use crate::whitespace::WHITESPACE;
use crate::{Delimiter, Node, NodeKind};

#[derive(Debug, Serialize, PartialEq)]
pub struct Attribute {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    pub value: Value,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    String(String),
    Node(Attributes),
}

// Attributes follow KDL's model of a node: a chain supplies the node's
// entries (arguments and properties) and ``attr: blocks supply its children.
// The two are separate namespaces, so ``color="red"`` in a chain and
// ``color "red"`` in a block coexist rather than override one another.
#[derive(Debug, Default, Serialize, PartialEq)]
pub struct Attributes {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<Attribute>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Attribute>,
    /// Where the ``attr: blocks that supplied `children` are in the source, in
    /// source order — what a tool replaces to rewrite them. Several blocks
    /// merge into one owner, so this is a list; the document's metadata
    /// routinely collects blocks from all over the file.
    ///
    /// Only block-form blocks appear. One written inside a line has no span to
    /// give, and a span that pointed at the whole line would delete the line.
    /// Empty here means "nothing to splice", never "no block".
    ///
    /// Always empty on the nested `Attributes` inside a [`Value::Node`]: those
    /// come from KDL within a block, not from a block of their own.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<Span>,
}

// the sole attribute marking a verbatim block as an attribute block
const ATTR: &str = "attr";

impl Attribute {
    fn is_attr(&self) -> bool {
        self.name.is_none()
            && self.ty.is_none()
            && matches!(&self.value, Value::String(name) if name == ATTR)
    }
}

impl Attributes {
    /// Whether the chain consists solely of the bare `attr` marker.
    pub fn is_attribute_chain(&self) -> bool {
        matches!(self.entries.as_slice(), [attribute] if attribute.is_attr())
    }

    // KDL resolves a repeated property to its rightmost value. Arguments are
    // unnamed and have no name to collide on, so they accumulate.
    fn push_entry(&mut self, entry: Attribute) {
        let position = entry.name.as_ref().and_then(|name| {
            self.entries
                .iter()
                .position(|existing| existing.name.as_ref() == Some(name))
        });

        match position {
            Some(position) => self.entries[position] = entry,
            None => self.entries.push(entry),
        }
    }
}

// boxed because attributes are a minority of nodes; the shape every owner stores
pub type Owned = Option<Box<Attributes>>;

// an inherent method cannot be added to Option
pub(crate) trait AttributesExt {
    // `span` records where the block was, so a tool can replace it; a block
    // written inside a line has none
    fn push_children(&mut self, children: Vec<Attribute>, span: Option<Span>);

    // ``attr must be the only attribute; a chain such as ``rust:attr: is a
    // verbatim block whose content is preserved rather than consumed
    fn is_attribute_chain(&self) -> bool;
}

impl AttributesExt for Owned {
    fn is_attribute_chain(&self) -> bool {
        self.as_deref().is_some_and(Attributes::is_attribute_chain)
    }

    // KDL allows child nodes to repeat a name, so the children of successive
    // ``attr: blocks append into one document rather than override each other.
    //
    // an empty ``attr: block is still consumed, but leaves its target without
    // attributes rather than with an empty collection
    // an empty block still records its span: a generator that rewrites blocks
    // in place has to find the empty one it wrote last time, or it appends a
    // second one beside it every run
    fn push_children(&mut self, children: Vec<Attribute>, span: Option<Span>) {
        if children.is_empty() && span.is_none() {
            return;
        }

        let attributes = self.get_or_insert_default();
        attributes.children.extend(children);
        attributes.blocks.extend(span);
    }
}

// an attribute node folds into its parent; at the root the owner is the document
pub(crate) fn fold_nodes(nodes: &mut Vec<Node>, owner: &mut Owned) {
    if nodes.iter().any(Node::is_attributes) {
        let mut joins = false;

        for mut node in std::mem::take(nodes) {
            if node.is_attributes() {
                let span = node.attribute_block_span();
                let children = node
                    .attributes
                    .take()
                    .map(|a| a.children)
                    .unwrap_or_default();
                owner.push_children(children, span);
                joins = true;
                continue;
            }

            // text split only by the removed node is one run again
            if std::mem::take(&mut joins)
                && let NodeKind::Text(text) = &node.kind
                && let Some(NodeKind::Text(previous)) = nodes.last_mut().map(|last| &mut last.kind)
            {
                join_text(previous, text);
                continue;
            }

            nodes.push(node);
        }
    }

    for node in nodes {
        fold_nodes(&mut node.children, &mut node.attributes);
    }
}

// the space either side of a removed node is a single separator
fn join_text(previous: &mut String, text: &str) {
    match previous.ends_with(WHITESPACE) && text.starts_with(WHITESPACE) {
        true => previous.push_str(&text[1..]),
        false => previous.push_str(text),
    }
}

// src: substring beginning after a marker or delimiter
//
// parses attribute entries into vec until attribute chain
// broken or str end
pub fn parse_attributes(src: &str) -> (Owned, &str) {
    // a chain cannot begin with a delimiter: in **``attr: x`` bold** the
    // verbatim opener is content, not an entry named ``attr
    if Delimiter::at(src).is_some() {
        return (None, src);
    }

    let mut attributes = Attributes::default();
    let mut rest = src;

    while let Some(end) = attribute_end(rest)
        && let Some(attribute) = parse_attribute(&rest[..end])
    {
        attributes.push_entry(attribute);
        rest = &rest[end + 1..];
    }

    (
        (!attributes.entries.is_empty()).then(|| Box::new(attributes)),
        rest,
    )
}

fn parse_attribute(src: &str) -> Option<Attribute> {
    match KdlEntry::parse(src) {
        Ok(entry) => Some(Attribute::from(&entry)),
        // KDL reserves ``#`` for keywords and raw strings
        // currently plan to use this for as a link attribute
        // so we need to bypass KDL error on "#" attribute
        //
        // [[#: Link Target Header]]
        Err(_) => (src == "#").then(|| Attribute {
            name: None,
            ty: None,
            value: Value::String(String::from(src)),
        }),
    }
}

fn attribute_end(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];

        if byte == b':' {
            // preserve ``://`` in urls, ex: https://www.google.com
            return (!bytes[index..].starts_with(b"://")).then_some(index);
        }

        if matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
            return None;
        }

        index = match byte {
            // ? propagates quote_attribute_end's None result
            // Some() => {} None => return None
            b'"' => quoted_attribute_end(bytes, index + 1)?,
            _ => index + 1,
        };
    }
    None
}

pub fn parse_delimiter_attributes(src: &str, delimiter: Delimiter) -> (Owned, &str) {
    let (attributes, rest) = parse_attributes(src);

    // check for closing delimiter within parsed attribute source
    if let Some(boundary) = delimiter.attribute_boundary()
        && let Some(position) = src.as_bytes()[..src.len() - rest.len()]
            .windows(2)
            .position(|pair| pair == boundary)
    {
        let (attributes, inner_rest) = parse_attributes(&src[..position]);
        return (attributes, &src[position - inner_rest.len()..]);
    }

    (attributes, rest)
}

// Different rules for end of KDL entries once entering a quote
fn quoted_attribute_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Some(index + 1),
            b'\n' | b'\r' => return None,
            b'\\' => index += 2,
            _ => index += 1,
        }
    }
    None
}

/// The body of an `` ``attr: `` block as attributes, or the KDL error saying
/// why it is not one.
///
/// A block that fails to parse stays an ordinary verbatim block, which renders
/// its body as a code sample. That is a quiet way to lose data, so the error is
/// returned rather than dropped: a host can say what was wrong and where.
pub fn parse_attribute_block(text: &str) -> Result<Vec<Attribute>, String> {
    KdlDocument::parse(text)
        .map(|document| document_into_attributes(&document))
        .map_err(|error| describe(text, &error))
}

/// `KdlError`'s own `Display` is only ever "Failed to parse KDL document" —
/// what a reader needs is in its diagnostics. The line is counted within the
/// block, because that is the only frame this function knows about; the caller
/// knows where the block starts.
fn describe(text: &str, error: &KdlError) -> String {
    let Some(diagnostic) = error.diagnostics.first() else {
        return error.to_string();
    };

    // Diagnostic offsets are UTF-8 bytes, including any multibyte text before
    // the error. Counting characters here would overshoot its source position.
    let line = text
        .bytes()
        .take(diagnostic.span.offset())
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;

    let message = diagnostic
        .message
        .clone()
        .unwrap_or_else(|| error.to_string());

    match &diagnostic.help {
        Some(help) => format!("{message}, on line {line} of the block ({help})"),
        None => format!("{message}, on line {line} of the block"),
    }
}

// Vec is foreign, so this cannot be a From impl the way the two below are
fn document_into_attributes(document: &KdlDocument) -> Vec<Attribute> {
    document.nodes().iter().map(Attribute::from).collect()
}

impl From<&KdlNode> for Attribute {
    fn from(node: &KdlNode) -> Self {
        Attribute {
            name: Some(node.name().value().to_string()),
            ty: node.ty().map(|ty| ty.value().to_string()),
            value: Value::Node(Attributes {
                entries: node.entries().iter().map(Attribute::from).collect(),
                children: node
                    .children()
                    .map(document_into_attributes)
                    .unwrap_or_default(),
                // KDL within a block, not a block of its own
                blocks: Vec::new(),
            }),
        }
    }
}

impl From<&KdlEntry> for Attribute {
    fn from(entry: &KdlEntry) -> Self {
        Attribute {
            name: entry.name().map(|name| name.value().to_string()),
            ty: entry.ty().map(|ty| ty.value().to_string()),
            value: match entry.value() {
                KdlValue::Bool(bool) => Value::Bool(*bool),
                KdlValue::Float(float) => Value::Float(*float),
                KdlValue::Integer(int) => Value::Int(*int),
                KdlValue::Null => Value::Null,
                // value() lends a &KdlValue, so the String cannot be moved
                // out; KdlIdentifier has no owning accessor either
                KdlValue::String(string) => Value::String(string.clone()),
            },
        }
    }
}
