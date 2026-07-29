use crate::{
    Delimiter, Marker, MarkerKind, Node, NodeKind,
    node::attribute::{parse_attributes, parse_delimiter_attributes},
    whitespace,
};

pub(super) enum Line {
    Resolved(Node),
    Block(Node),
    Row(Node),
    Text,
}

pub(super) fn classify(line: &str) -> Line {
    let bytes = line.as_bytes();
    let Some(&first) = bytes.first() else {
        return Line::Text;
    };

    // Delimiter blocks, skips inline delimiters
    if let Some(delimiter) = Delimiter::at(bytes, 0) {
        let (attributes, rest) =
            parse_delimiter_attributes(&line[2..], delimiter.attribute_boundary());
        let rest = whitespace::trim(rest);

        if matches!(delimiter, Delimiter::TableHeader | Delimiter::TableRow) {
            return Line::Row(Node {
                kind: NodeKind::Delimiter(delimiter),
                attributes,
                children: (!rest.is_empty()).then(|| vec![Node::raw(rest)]),
            });
        }

        // TODO: matches!(delimiter, Delimiter::TableCell)
        // error, table cell cannot start a row
        // handle inline delimiters in phase 2
        if delimiter.closing().is_none() || !rest.is_empty() {
            return Line::Text;
        }

        return Line::Block(Node {
            kind: NodeKind::Delimiter(delimiter),
            attributes,
            children: None,
        });
    }

    // Structural Markers
    if let Some(kind) = MarkerKind::from_byte(first) {
        let depth = bytes.iter().take_while(|&&b| b == first).count();
        let marker = Marker { depth, kind };
        let (attributes, rest) = parse_attributes(&line[depth..]);
        let rest = whitespace::trim(rest);

        let node = Node {
            kind: NodeKind::Marker(marker),
            attributes,
            children: (!rest.is_empty()).then(|| vec![Node::raw(rest)]),
        };

        return match rest.is_empty() {
            true => Line::Block(node),
            false => Line::Resolved(node),
        };
    }

    Line::Text
}
