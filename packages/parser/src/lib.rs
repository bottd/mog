pub use error::MogError;
pub use node::{Document, Node};

use crate::lexer::lexer;

mod attribute;
mod error;
mod lexer;
mod marker;
mod metadata;
mod node;

pub fn parse(src: &str) -> Result<Document, MogError> {
    lexer(src)
}
