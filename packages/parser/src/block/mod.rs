use crate::Node;

use parser::Parser;

mod line;
mod parser;

const BOM: char = '\u{FEFF}';

pub fn parse_blocks(src: &str) -> Vec<Node> {
    let mut parser = Parser::default();
    // BOM is stripped if present; .lines() handles both LF and CRLF
    for line in src.strip_prefix(BOM).unwrap_or(src).lines() {
        parser.feed(line);
    }
    parser.close()
}
