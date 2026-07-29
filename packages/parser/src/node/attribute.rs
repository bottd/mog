use crate::node::data::{Data, entry_into_data};
use kdl::KdlEntry;

// src: substring beginning after a marker or delimiter
//
// parses attribute entries into vec until attribute chain
// broken or str end
pub fn parse_attributes(src: &str) -> (Option<Vec<Data>>, &str) {
    let mut attributes = Vec::new();
    let mut rest = src;

    while let Some(end) = attribute_end(rest)
        && let Ok(entry) = KdlEntry::parse(&rest[..end])
    {
        attributes.push(entry_into_data(&entry));
        rest = &rest[end + 1..];
    }

    ((!attributes.is_empty()).then_some(attributes), rest)
}

fn attribute_end(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let byte = *bytes.get(index)?;

        if byte == b':' {
            return Some(index);
        }

        if matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
            return None;
        }

        index = match byte {
            // ? propagates quote_attribute_end's None result
            // Some() => {} None => return None
            b'"' => quoted_attribute_end(bytes, index + 1)?,
            _ => index + 1,
        };
    }
    None
}

pub fn parse_delimiter_attributes(
    src: &str,
    boundary: Option<[u8; 2]>,
) -> (Option<Vec<Data>>, &str) {
    let (attributes, rest) = parse_attributes(src);

    // check for closing delimiter within parsed attribute source
    if let Some(boundary) = boundary
        && let Some(position) = src.as_bytes()[..src.len() - rest.len()]
            .windows(2)
            .position(|pair| pair == boundary)
    {
        let (attributes, inner_rest) = parse_attributes(&src[..position]);
        return (attributes, &src[position - inner_rest.len()..]);
    }

    (attributes, rest)
}

// Different rules for end of KDL entries once entering a quote
fn quoted_attribute_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Some(index + 1),
            b'\n' | b'\r' => return None,
            b'\\' => index += 2,
            _ => index += 1,
        }
    }
    None
}
