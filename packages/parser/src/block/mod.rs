use crate::Node;
use crate::span::Span;

use parser::Parser;

mod line;
mod parser;

const BOM: char = '\u{FEFF}';

pub fn parse_blocks(src: &str) -> Vec<Node> {
    let mut parser = Parser::default();
    // A BOM is skipped, but the offset starts past it so spans stay relative
    // to the string the caller passed rather than to the stripped one.
    let bom = if src.starts_with(BOM) {
        BOM.len_utf8()
    } else {
        0
    };
    let mut offset = bom;

    // split_inclusive rather than .lines(): the terminator's own width is what
    // turns a line into a byte offset, and .lines() throws it away. Stripping
    // it here is also the one place CRLF has to be handled — a span covers a
    // line's content, never its ending.
    let mut last = None;

    for (index, raw) in src[bom..].split_inclusive('\n').enumerate() {
        // Only a \r that precedes the \n is part of the ending — a bare one at
        // the end of the file is content, as it is to `str::lines`.
        let content = match raw.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => raw,
        };

        let span = Span::line(offset, offset + content.len(), index);
        // Anything still open at the end of the document ends at the last line
        // that had something on it. A blank line's content is empty, so ending
        // there would put the span's end before that line's own newline — and a
        // span never includes a line ending.
        if !content.is_empty() {
            last = Some(span);
        }
        parser.feed(content, span);
        offset += raw.len();
    }

    parser.close(last)
}
