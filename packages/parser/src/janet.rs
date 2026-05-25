use crate::attribute::Attribute;

pub enum JanetValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    Keyword(String),
    Tuple(Vec<JanetValue>),
    Struct(Vec<(JanetValue, JanetValue)>),
}

// I can't ahve parse_janet take attributes as an arg
// becuase it needs to be recurisve
//
// otherwise i need parse_janet to call a separate inner fn that can be recursive

// This method only parses janet tuple and structs, for use in mog attributes
pub fn parse_janet<'a>(src: &str, attributes: &mut Vec<Attribute>) -> &'a str {
    let mut value = match src.bytes().next() {
        Some(b'{') => JanetValue::Struct(Vec::new()),
        Some(b'[') => JanetValue::Tuple(Vec::new()),
        _ => JanetValue::Nil,
    };
    todo!()
}
