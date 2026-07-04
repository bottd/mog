use crate::node::NodeKind;
use crate::node::attribute::parse_attributes;
use crate::node::delimiter::Delimiter;
use crate::node::marker::{Marker, MarkerKind};
use crate::{Document, MogError, Node};

const BOM: char = '\u{FEFF}';
pub fn parse_blocks(src: &str) -> Result<Document, MogError> {
    // BOM is stripped if present
    // String is then split with .lines()
    // which handles both LF and CRLF
    let lines = src.strip_prefix(BOM).unwrap_or(src).lines().peekable();
    let mut document = Document::default();
    let mut open: Vec<Node> = Vec::new();

    for line in lines {
        // TODO: line-level error handling
        // this just skips error lines
        if let Ok(Some(node)) = parse_line(line, &mut open) {
            document.body.push(node);
        }
    }

    // if let Some(first) = document.body.first() && first.
    // TODO: Update meta parsing
    // we still call this on the contents,
    // but it is different now metadata is conforming syntax
    // we can post-parse pluck this off the tree if node 1 is verbatim meta
    //
    // if first line is metadata delimiter, parse metadata
    // if lines.peek() == Some(&METADATA_OPEN) {
    //     lines.next(); // consume opening delimiter
    //     let mut meta_lines = String::new();
    //
    //     while let Some(line) = lines.next_if(|l| l != &METADATA_CLOSE) {
    //         meta_lines.push_str(line);
    //         meta_lines.push('\n');
    //     }
    //     lines.next(); // consume closing delimiter
    //
    //     // TODO: kdl error handling
    //     if let Ok(kdl) = KdlDocument::parse(&meta_lines)
    //         && !kdl.is_empty()
    //     {
    //         document.meta = Some(kdl.try_into()?);
    //     }
    // }

    Ok(document)
}

pub fn parse_line(line: &str, open: &mut Vec<Node>) -> Result<Option<Node>, MogError> {
    let line_trimmed = line.trim();
    let bytes = line_trimmed.as_bytes();
    let mut offset = 0;
    let mut node: Option<Node> = None;

    // If in open verbatim delimiter,
    // pass full line to verbatim content
    //
    // TODO: track indentation on opening delimiter
    //       then strip that much whitespace from
    //       verbatim lines
    if let Some(parent) = open.last_mut()
        && parent.kind == NodeKind::Delimiter(Delimiter::Verbatim)
    {
        parent
            .children
            .get_or_insert(Vec::new())
            .push(Node::leaf(NodeKind::Raw(String::from(line))))
    }

    if let Some(&first) = bytes.first() {
        if let Some(second) = bytes.get(1)
            && let Some(delimiter) = Delimiter::from_bytes(&first, second)
        {
            if let Some(last) = open.last()
                && last.kind == NodeKind::Delimiter(delimiter)
            {
                return Ok(open.pop());
            }
            // consume both delimiter chars
            offset = 2;
            node = Some(Node::leaf(NodeKind::Delimiter(delimiter)));
        } else if let Some(marker_kind) = MarkerKind::from_byte(first) {
            let marker_byte = marker_kind.byte();
            // consume all marker chars
            offset = bytes
                .iter()
                .take_while(|&&byte| byte == marker_byte)
                .count();
            let marker = Marker {
                kind: marker_kind,
                depth: offset,
            };

            if let Some(last) = open.last()
                && last.kind == NodeKind::Marker(marker)
            {
                // TODO: error/warn when closing marker line has attributes or text
                return Ok(open.pop());
            }
            node = Some(Node::leaf(NodeKind::Marker(marker)));
        }
    }

    let Some(mut node) = node else {
        return Ok(None);
    };

    let (attributes, rest) = parse_attributes(&line_trimmed[offset..]);
    node.attributes = attributes;

    if rest.trim().is_empty() {
        open.push(node);
        Ok(None)
    } else {
        node.children = Some(vec![Node::leaf(NodeKind::Raw(String::from(rest)))]);
        Ok(Some(node))
    }
}
