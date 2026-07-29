use crate::{Delimiter, Node, NodeKind, node::attribute::parse_delimiter_attributes, whitespace};

use super::raw::{parse_raw, verbatim_end};

// TODO: track column width to pad rows so table has same cols in all row
// or maybe just do the padding at the end
pub(super) fn resolve_table(rows: Vec<Node>, ancestors: &[Delimiter]) -> Vec<Node> {
    let mut rows: Vec<Node> = rows
        .into_iter()
        .flat_map(|row| resolve_row(row, ancestors))
        .collect();

    pad_table(&mut rows);

    rows
}

fn pad_table(rows: &mut [Node]) {
    let width = rows.iter().map(row_len).max().unwrap_or_default();

    for row in rows {
        if row_len(row) < width {
            row.children.get_or_insert_default().resize_with(width, || {
                Node::empty(NodeKind::Delimiter(Delimiter::TableCell))
            });
        }
    }
}

fn row_len(row: &Node) -> usize {
    row.children.as_ref().map_or(0, Vec::len)
}

fn resolve_row(row: Node, ancestors: &[Delimiter]) -> Vec<Node> {
    let text = row.get_raw_text();
    let bytes = text.as_bytes();

    let mut rows = Vec::new();
    let mut current = Node {
        kind: row.kind,
        attributes: row.attributes,
        children: None,
    };
    let mut cell_start = 0;
    let mut position = 0;

    while position < bytes.len() {
        if bytes[position] == b'\\' && Delimiter::at(bytes, position + 1).is_some() {
            position += 3;
            continue;
        }

        if Delimiter::at(bytes, position) == Some(Delimiter::Verbatim) {
            let (_, next) = verbatim_end(&text, position + 2);
            position = next;
            continue;
        }

        if Delimiter::at(bytes, position) != Some(Delimiter::TableCell) {
            position += 1;
            continue;
        }

        push_cell(&mut current, &text[cell_start..position], ancestors);
        position += 2;
        cell_start = position;

        // a row delimiter where the next cell would start opens a new row
        let rest = whitespace::trim_start(&text[position..]);
        if let Some(delimiter) = Delimiter::at(rest.as_bytes(), 0)
            && matches!(delimiter, Delimiter::TableHeader | Delimiter::TableRow)
        {
            let (attributes, remainder) =
                parse_delimiter_attributes(&rest[2..], delimiter.attribute_boundary());

            rows.push(current);
            current = Node {
                kind: NodeKind::Delimiter(delimiter),
                attributes,
                children: None,
            };

            position = text.len() - remainder.len();
            cell_start = position;
        }
    }

    let trailing = whitespace::trim(&text[cell_start..]);
    if !trailing.is_empty() {
        push_cell(&mut current, trailing, ancestors);
    }

    rows.push(current);
    rows
}

fn push_cell(row: &mut Node, content: &str, ancestors: &[Delimiter]) {
    let content = whitespace::trim(content);
    row.push_child(Node {
        kind: NodeKind::Delimiter(Delimiter::TableCell),
        attributes: None,
        children: (!content.is_empty()).then(|| parse_raw(content, ancestors)),
    });
}
