pub use node::attribute::{Attribute, Attributes, Value, parse_attribute_block};

use node::attribute::fold_nodes;
pub use node::delimiter::Delimiter;

use node::delimiter::Delimiters;
pub use node::marker::{Marker, MarkerKind};
pub use node::{Node, NodeKind};
pub use span::Span;

use serde::Serialize;

use crate::block::parse_blocks;
use crate::inline::resolve_nodes;

mod block;
mod inline;
mod node;
mod span;
mod whitespace;

#[derive(Debug, Serialize, PartialEq)]
pub struct Document {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: node::attribute::Owned,
    pub body: Vec<Node>,
}

pub fn parse(src: &str) -> Document {
    parse_unfolded(src).fold_attributes()
}

// for tooling: ``attr: blocks stay in the tree as NodeKind::Attributes nodes
pub fn parse_unfolded(src: &str) -> Document {
    let body = parse_blocks(src);
    let body = resolve_nodes(body, Delimiters::default());

    Document {
        attributes: None,
        body,
    }
}

impl Document {
    pub fn fold_attributes(mut self) -> Document {
        fold_nodes(&mut self.body, &mut self.attributes);
        self
    }
}
