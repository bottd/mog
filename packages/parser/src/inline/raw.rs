use std::borrow::Cow;

use winnow::stream::{Offset, Stream};

use crate::node::attribute::{AttributesExt, parse_attribute_block};
use crate::{
    Delimiter, Delimiters, Node, NodeKind, node::attribute::parse_delimiter_attributes, whitespace,
};

use super::parser::Parser;

// the unconsumed source is the cursor: every step advances it in place, so
// there is no separate index to keep in sync
pub(super) fn parse_raw(text: &str, ancestors: Delimiters) -> Vec<Node> {
    let mut parser = Parser::default();
    let mut input = text;

    while !input.is_empty() {
        // a backslash makes the delimiter or character after it literal; a
        // trailing backslash is literal itself
        if let Some(escaped) = input.strip_prefix('\\')
            && let Some(first) = escaped.chars().next()
        {
            let width = match Delimiter::at(escaped) {
                Some(_) => 2,
                None => first.len_utf8(),
            };

            input.next_slice(1);
            parser.buffer.push_str(input.next_slice(width));
            continue;
        }

        if let Some(delimiter) = Delimiter::at(input) {
            // TODO: position < self.stack.len() - 1
            // error, everything on stack between top and [position] is unclosed

            // TODO: node.attributes.is_some()
            // error, unclosed verbatim or attribute on closing verbatim
            if delimiter.closes(input)
                && parser.stack.last().map(|open| &open.kind)
                    == Some(&NodeKind::Delimiter(delimiter))
            {
                input.next_slice(2);
                parser.close_node();
                continue;
            }

            if let Some(closing) = delimiter.closing()
                && delimiter.opens(input)
                && !ancestors.contains(delimiter)
                && !parser
                    .stack
                    .iter()
                    .any(|open| open.kind == NodeKind::Delimiter(delimiter))
            {
                input.next_slice(2);
                open_delimiter(&mut parser, &mut input, delimiter, closing);
                continue;
            }
        }

        let Some(ch) = input.next_token() else {
            break;
        };

        parser.push_char(ch);
    }

    parser.collect()
}

// the cursor sits just past the opening pair; consumes the attribute chain and
// whatever body the delimiter owns
fn open_delimiter(parser: &mut Parser, input: &mut &str, delimiter: Delimiter, closing: [u8; 2]) {
    let (attributes, rest) = parse_delimiter_attributes(input, delimiter);
    *input = rest;

    let mut node = Node::new(NodeKind::Delimiter(delimiter), attributes);

    match delimiter {
        Delimiter::Verbatim => {
            let content = take_until_closing(input, closing);
            let content = whitespace::trim(&content);

            // an attribute node in place of the block; invalid KDL stays verbatim
            if node.attributes.is_attribute_chain()
                && let Ok(attributes) = parse_attribute_block(content)
            {
                parser.absorb_separator(input);
                parser.flush();
                parser.push_node(Node::attributes(attributes));
                return;
            }

            node.children = Node::raw_children(content);
        }
        Delimiter::Link => {
            node.kind =
                NodeKind::Link(whitespace::trim(&take_until_closing(input, closing)).to_owned());
        }
        _ => {
            parser.flush();
            parser.stack.push(node);
            return;
        }
    }

    parser.flush();
    parser.push_node(node);
}

// the content up to the closing pair, consuming the closer; an escaped closer
// is literal content, and is the only case that cannot be borrowed
pub(super) fn take_until_closing<'a>(input: &mut &'a str, closing: [u8; 2]) -> Cow<'a, str> {
    let start = *input;

    while !input.is_empty() {
        if input.starts_with('\\') {
            return Cow::Owned(unescape_until_closing(input, start, closing));
        }

        if input.as_bytes().starts_with(&closing) {
            let content = &start[..input.offset_from(&start)];
            input.next_slice(2);
            return Cow::Borrowed(content);
        }

        input.next_token();
    }

    // TODO: error, unclosed delimiter
    Cow::Borrowed(start)
}

// resumes the scan with an owned buffer once a backslash is seen
fn unescape_until_closing(input: &mut &str, start: &str, closing: [u8; 2]) -> String {
    let mut content = String::from(&start[..input.offset_from(&start)]);

    while !input.is_empty() {
        if let Some(escaped) = input.strip_prefix('\\')
            && escaped.as_bytes().starts_with(&closing)
        {
            input.next_slice(1);
            content.push_str(input.next_slice(2));
            continue;
        }

        if input.as_bytes().starts_with(&closing) {
            input.next_slice(2);
            return content;
        }

        let Some(ch) = input.next_token() else {
            break;
        };

        content.push(ch);
    }

    // TODO: error, unclosed delimiter
    content
}
