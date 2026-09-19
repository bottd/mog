use crate::{
    Delimiter, Marker, MarkerKind, Node, NodeKind,
    node::attribute::{parse_attributes, parse_delimiter_attributes},
    span::Span,
    whitespace,
};

pub(super) enum Line {
    Resolved(Node),
    Block(Node),
    // ``…: — the body is raw content rather than markup
    Verbatim(Node),
    Row(Node),
    Blank,
    Text,
}

// a verbatim block closes on a line that is nothing but the delimiter
pub(super) fn is_verbatim_close(line: &str) -> bool {
    whitespace::trim(line).as_bytes() == Delimiter::Verbatim.opening()
}

pub(super) fn classify(line: &str, span: Span) -> Line {
    let bytes = line.as_bytes();
    let Some(&first) = bytes.first() else {
        return Line::Blank;
    };

    // Delimiter blocks, skips inline delimiters
    if let Some(delimiter) = Delimiter::at(line) {
        let (attributes, rest) = parse_delimiter_attributes(&line[2..], delimiter);
        let rest = whitespace::trim(rest);

        if matches!(delimiter, Delimiter::TableHeader | Delimiter::TableRow) {
            return Line::Row(Node {
                kind: NodeKind::Delimiter(delimiter),
                attributes,
                children: Node::raw_children(rest),
                span: Some(span),
                fence: None,
            });
        }

        // TODO: matches!(delimiter, Delimiter::TableCell)
        // error, table cell cannot start a row
        // handle inline delimiters in phase 2
        if delimiter.closing().is_none() || !rest.is_empty() {
            return Line::Text;
        }

        let node = Node::new(NodeKind::Delimiter(delimiter), attributes).at(span);

        return match delimiter {
            Delimiter::Verbatim => Line::Verbatim(node),
            _ => Line::Block(node),
        };
    }

    // Structural Markers
    if let Some(kind) = MarkerKind::from_byte(first) {
        let depth = bytes.iter().take_while(|&&b| b == first).count();
        let marker = Marker { depth, kind };
        let (attributes, rest) = parse_attributes(&line[depth..]);
        let rest = whitespace::trim(rest);

        let mut node = Node {
            kind: NodeKind::Marker(marker),
            attributes,
            children: Node::raw_children(rest),
            span: Some(span),
            fence: None,
        };

        return match rest.is_empty() {
            // A block marker's span grows to its closing fence, so it keeps the
            // opening line separately — that is where an editor inserts. A
            // single-line marker starts without one; attaching an attribute
            // block preserves its opening line before growing its span later.
            true => {
                node.fence = Some(span);
                Line::Block(node)
            }
            false => Line::Resolved(node),
        };
    }

    Line::Text
}
