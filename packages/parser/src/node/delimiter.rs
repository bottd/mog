use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub enum Delimiter {
    Strong,
    Italic,
    Verbatim,
    Strikethrough,
    TableHeader,
    TableRow,
    TableCell,
    Footnote,
    Link,
    LinkName,
}

impl Delimiter {
    pub const fn from_bytes(pair: [u8; 2]) -> Option<Self> {
        Some(match pair {
            [b'*', b'*'] => Self::Strong,
            [b'_', b'_'] => Self::Italic,
            [b'`', b'`'] => Self::Verbatim,
            [b'~', b'~'] => Self::Strikethrough,
            [b'#', b'|'] => Self::TableHeader,
            [b'-', b'|'] => Self::TableRow,
            [b'|', b'|'] => Self::TableCell,
            [b'{', b'{'] | [b'}', b'}'] => Self::Footnote,
            [b'[', b'['] | [b']', b']'] => Self::Link,
            [b'(', b'('] | [b')', b')'] => Self::LinkName,
            _ => return None,
        })
    }

    pub fn at(bytes: &[u8], position: usize) -> Option<Self> {
        match (bytes.get(position), bytes.get(position + 1)) {
            (Some(&first), Some(&second)) => Delimiter::from_bytes([first, second]),
            _ => None,
        }
    }

    pub const fn opening(self) -> [u8; 2] {
        match self {
            Self::Strong => [b'*', b'*'],
            Self::Italic => [b'_', b'_'],
            Self::Verbatim => [b'`', b'`'],
            Self::Strikethrough => [b'~', b'~'],
            Self::TableHeader => [b'#', b'|'],
            Self::TableRow => [b'-', b'|'],
            Self::TableCell => [b'|', b'|'],
            Self::Link => [b'[', b'['],
            Self::LinkName => [b'(', b'('],
            Self::Footnote => [b'{', b'{'],
        }
    }

    pub const fn closing(self) -> Option<[u8; 2]> {
        match self {
            Self::TableHeader | Self::TableRow | Self::TableCell => None,
            Self::Link => Some([b']', b']']),
            Self::LinkName => Some([b')', b')']),
            Self::Footnote => Some([b'}', b'}']),
            _ => Some(self.opening()),
        }
    }

    pub fn is_symmetric(self) -> bool {
        self.closing() == Some(self.opening())
    }

    pub fn attribute_boundary(self) -> Option<[u8; 2]> {
        match self {
            Self::TableHeader | Self::TableRow => Some(Self::TableCell.opening()),
            _ => self.closing(),
        }
    }
}
