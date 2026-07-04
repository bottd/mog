pub use error::MogError;
pub use node::{Document, Node};

use crate::blocks::parse_blocks;

mod blocks;
mod error;
mod metadata;
mod node;

pub fn parse(src: &str) -> Result<Document, MogError> {
    parse_blocks(src)
}
