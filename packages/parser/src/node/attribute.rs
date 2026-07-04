use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub enum Attribute {
    String(String),
    Pair(String, String),
}

#[inline]
pub(crate) fn is_reserved(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'|' | b';' | b'"' | b'\'' | b',' | b' ' | b'\t'
    )
}

// parses string for attributes, if found at start of string, returned in vec
// rest of str returned in 2nd position
pub fn parse_attributes(src: &str) -> (Option<Vec<Attribute>>, &str) {
    let mut attributes: Vec<Attribute> = Vec::new();
    let str = parse_attribute(src, &mut attributes);

    if attributes.is_empty() {
        (None, str)
    } else {
        (Some(attributes), str)
    }
}

pub fn parse_attribute<'a>(src: &'a str, attributes: &mut Vec<Attribute>) -> &'a str {
    match src.bytes().next() {
        Some(byte) => {
            if byte.is_ascii_whitespace() {
                src
            } else {
                match src.split_once(':') {
                    Some((left, right)) => {
                        if left.bytes().any(is_reserved) {
                            return src;
                        }
                        attributes.push(Attribute::String(left.to_string()));
                        parse_attribute(right, attributes)
                    }
                    None => src,
                }
            }
        }
        None => src,
    }
}
