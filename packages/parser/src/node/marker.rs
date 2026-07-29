use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Marker {
    pub depth: usize,
    pub kind: MarkerKind,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub enum MarkerKind {
    Heading,
    UnorderedList,
    OrderedList,
    Blockquote,
    Free,
}

impl MarkerKind {
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'#' => Some(MarkerKind::Heading),
            b'-' => Some(MarkerKind::UnorderedList),
            b'.' => Some(MarkerKind::OrderedList),
            b'>' => Some(MarkerKind::Blockquote),
            b'=' => Some(MarkerKind::Free),
            _ => None,
        }
    }
}
