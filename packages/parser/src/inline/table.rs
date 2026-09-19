use crate::{
    Delimiter, Delimiters, Node, NodeKind, node::attribute::parse_delimiter_attributes, whitespace,
};

use winnow::stream::{Offset, Stream};

use super::raw::{parse_raw, take_until_closing};

// TODO: track column width to pad rows so table has same cols in all row
// or maybe just do the padding at the end
pub(super) fn resolve_table(rows: Vec<Node>, ancestors: Delimiters) -> Vec<Node> {
    let mut rows: Vec<Node> = rows
        .into_iter()
        .flat_map(|row| resolve_row(row, ancestors))
        .collect();

    pad_table(&mut rows);

    rows
}

fn pad_table(rows: &mut [Node]) {
    let width = rows
        .iter()
        .map(|row| row.children.len())
        .max()
        .unwrap_or_default();

    // width is the maximum, so this only ever grows a row
    for row in rows.iter_mut().filter(|row| !row.is_attributes()) {
        row.children.resize_with(width, || {
            Node::empty(NodeKind::Delimiter(Delimiter::TableCell))
        });
    }
}

fn resolve_row(row: Node, ancestors: Delimiters) -> Vec<Node> {
    if row.is_attributes() {
        return vec![row];
    }

    // the row's only child is its raw text, taken here rather than copied
    let mut current = row;
    let text = match current.children.pop().map(|child| child.kind) {
        Some(NodeKind::Raw(text)) => text,
        _ => String::new(),
    };

    let mut rows = Vec::new();
    let mut input = text.as_str();
    // where the cell being scanned began; the span back to it is its content
    let mut cell = input;

    while !input.is_empty() {
        // an escaped delimiter is cell content
        if input
            .strip_prefix('\\')
            .is_some_and(|escaped| Delimiter::at(escaped).is_some())
        {
            input.next_slice(3);
            continue;
        }

        match Delimiter::at(input) {
            // a verbatim span is opaque, so a || inside it does not end the cell
            Some(Delimiter::Verbatim) => {
                input.next_slice(2);
                // Verbatim is symmetric, so its opening pair is also its closer
                take_until_closing(&mut input, Delimiter::Verbatim.opening());
                continue;
            }
            Some(Delimiter::TableCell) => {}
            _ => {
                input.next_token();
                continue;
            }
        }

        push_cell(&mut current, &cell[..input.offset_from(&cell)], ancestors);
        input.next_slice(2);
        cell = input;

        // a row delimiter where the next cell would start opens a new row
        let rest = whitespace::trim_start(input);
        if let Some(delimiter) = Delimiter::at(rest)
            && matches!(delimiter, Delimiter::TableHeader | Delimiter::TableRow)
        {
            let (attributes, remainder) = parse_delimiter_attributes(&rest[2..], delimiter);

            rows.push(current);
            current = Node::new(NodeKind::Delimiter(delimiter), attributes);

            input = remainder;
            cell = input;
        }
    }

    let trailing = whitespace::trim(cell);
    if !trailing.is_empty() {
        push_cell(&mut current, trailing, ancestors);
    }

    rows.push(current);
    rows
}

fn push_cell(row: &mut Node, content: &str, ancestors: Delimiters) {
    let content = whitespace::trim(content);
    let mut cell = Node::empty(NodeKind::Delimiter(Delimiter::TableCell));
    cell.children = parse_raw(content, ancestors);

    row.push_child(cell);
}
