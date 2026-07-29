use crate::{Delimiter, Node, NodeKind, node::attribute::parse_delimiter_attributes, whitespace};

use super::parser::Parser;

pub(super) fn parse_raw(text: &str, ancestors: &[Delimiter]) -> Vec<Node> {
    let mut parser = Parser::default();
    let bytes = text.as_bytes();
    let mut position = 0;

    while position < bytes.len() {
        // escaping
        if bytes[position] == b'\\' && position + 1 < bytes.len() {
            let escaped = &text[position + 1..];
            let width = match Delimiter::at(bytes, position + 1) {
                Some(_) => 2,
                None => escaped.chars().next().expect("char").len_utf8(),
            };
            parser.buffer.push_str(&escaped[..width]);
            position += 1 + width;
            continue;
        }

        if let Some(delimiter) = Delimiter::at(bytes, position) {
            let pair = [bytes[position], bytes[position + 1]];
            let closing = delimiter.closing();

            // TODO: position < self.stack.len() - 1
            // error, everything on stack between top and [position] is unclosed

            // TODO: node.attributes.is_some()
            // error, unclosed verbatim or attribute on closing verbatim
            if closing == Some(pair)
                && parser.stack.last().map(|open| &open.kind)
                    == Some(&NodeKind::Delimiter(delimiter))
            {
                position += 2;
                parser.close_node();
                continue;
            }

            if delimiter.opening() == pair
                && closing.is_some()
                && !ancestors.contains(&delimiter)
                && !parser
                    .stack
                    .iter()
                    .any(|open| open.kind == NodeKind::Delimiter(delimiter))
            {
                let (attributes, rest) = parse_delimiter_attributes(
                    &text[position + 2..],
                    delimiter.attribute_boundary(),
                );
                position = text.len() - rest.len();

                let mut node = Node {
                    kind: NodeKind::Delimiter(delimiter),
                    attributes,
                    children: None,
                };

                parser.flush();
                match delimiter {
                    Delimiter::Verbatim => {
                        let (content, next) = verbatim_end(text, position);
                        let content = whitespace::trim(&content);
                        node.children = (!content.is_empty()).then(|| vec![Node::raw(content)]);
                        parser.push_node(node);
                        position = next;
                    }
                    _ => parser.stack.push(node),
                }
                continue;
            }
        }

        let ch = text[position..].chars().next().expect("char");
        parser.push_char(ch);
        position += ch.len_utf8();
    }

    parser.collect()
}

pub(super) fn verbatim_end(text: &str, start: usize) -> (String, usize) {
    let bytes = text.as_bytes();
    let mut content = String::new();
    let mut position = start;

    while position < bytes.len() {
        if bytes[position] == b'\\'
            && Delimiter::at(bytes, position + 1) == Some(Delimiter::Verbatim)
        {
            content.push_str(&text[position + 1..position + 3]);
            position += 3;
            continue;
        }
        if Delimiter::at(bytes, position) == Some(Delimiter::Verbatim) {
            return (content, position + 2);
        }
        let ch = text[position..].chars().next().expect("char");
        content.push(ch);
        position += ch.len_utf8();
    }

    // TODO: position >= bytes.len()
    // error, unclosed verbatim
    (content, bytes.len())
}
