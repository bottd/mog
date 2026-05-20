use serde::Serialize;
use strum::EnumIter;

use crate::Node;
use crate::attribute::Attribute;

#[derive(Debug, Serialize, PartialEq)]
pub struct Marker {
    pub kind: StructuralMarker,
    pub depth: usize,
    pub scope: StructuralMarkerScope,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, Serialize, EnumIter, PartialEq)]
pub enum StructuralMarker {
    Heading,
    UnorderedList,
    OrderedList,
    Blockquote,
    ThematicBreak,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub enum StructuralMarkerScope {
    Inline,
    Block,
}

impl StructuralMarker {
    pub const fn byte(self) -> u8 {
        match self {
            Self::Heading => b'#',
            Self::UnorderedList => b'-',
            Self::OrderedList => b'.',
            Self::Blockquote => b'>',
            Self::ThematicBreak => b'=',
        }
    }

    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'#' => Some(Self::Heading),
            b'-' => Some(Self::UnorderedList),
            b'.' => Some(Self::OrderedList),
            b'>' => Some(Self::Blockquote),
            b'=' => Some(Self::ThematicBreak),
            _ => None,
        }
    }
}
