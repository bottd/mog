pub use node::data::{Data, Value};
pub use node::delimiter::Delimiter;
pub use node::marker::{Marker, MarkerKind};
pub use node::{Node, NodeKind};

use serde::Serialize;

use crate::block::parse_blocks;
use crate::inline::parse_inlines;
use crate::node::DataArgument;

mod block;
mod inline;
mod node;
mod whitespace;

#[derive(Debug, Serialize, PartialEq)]
pub struct Document {
    pub meta: Option<Vec<Data>>,
    pub body: Vec<Node>,
}

pub fn parse(src: &str) -> Document {
    let mut nodes = parse_blocks(src);

    let meta = nodes
        .first()
        .and_then(|node| node.get_verbatim_data(DataArgument::Meta))
        .inspect(|_| {
            nodes.remove(0);
        });

    Document {
        meta,
        body: parse_inlines(nodes),
    }
}
