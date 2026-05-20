use crate::attribute::parse_attributes;
use crate::marker::{Marker, StructuralMarker, StructuralMarkerScope};
use crate::node::task::parse_task;
use crate::{Document, MogError, Node};
use kdl::KdlDocument;

const BOM: char = '\u{FEFF}';
const METADATA_DELIMITER: &str = "--";

pub fn lexer(src: &str) -> Result<Document, MogError> {
    // BOM is stripped if present
    // String is then split with .lines()
    // which handles both LF and CRLF
    let mut lines = src.strip_prefix(BOM).unwrap_or(src).lines().peekable();
    let mut document = Document::default();

    // if first line is metadata delimiter, parse metadata
    if lines.peek() == Some(&METADATA_DELIMITER) {
        lines.next(); // consume opening delimiter
        let mut meta_lines = String::new();

        while let Some(line) = lines.next_if(|l| l != &METADATA_DELIMITER) {
            meta_lines.push_str(line);
            meta_lines.push('\n');
        }
        lines.next(); // consume closing delimiter

        // TODO: kdl error handling
        if let Ok(kdl) = KdlDocument::parse(&meta_lines)
            && !kdl.is_empty()
        {
            document.meta = Some(kdl.try_into()?);
        }
    }

    for line in lines {
        document.body.extend(parse_line(line))
    }

    Ok(document)
}

pub fn parse_line(line: &str) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    let line = line.trim();

    if let Some(first) = line.bytes().next() {
        if let Some(kind) = StructuralMarker::from_byte(first) {
            let mut children: Vec<Node> = Vec::new();
            let marker_byte = kind.byte();
            let depth = line.bytes().take_while(|byte| *byte == marker_byte).count();

            let (attributes, mut rest) = parse_attributes(&line[depth..]);
            if let Some((task, _rest)) = parse_task(rest) {
                children.push(task);
                rest = _rest;
            }

            let scope = match rest.trim().is_empty() {
                true => StructuralMarkerScope::Block,
                false => StructuralMarkerScope::Inline,
            };

            if scope == StructuralMarkerScope::Inline {
                children.push(Node::Text {
                    value: rest.trim().to_string(),
                });
            }

            nodes.push(Node::Marker(Marker {
                attributes,
                kind,
                depth,
                scope,
                children,
            }))
        }
    } else {
        return nodes;
    }

    nodes
}
