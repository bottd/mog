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

// the set of delimiters enclosing a position, as a bitset: at most ten
// variants, so a Copy mask replaces a slice that would be reallocated at
// every nesting level
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Delimiters(u16);

impl Delimiters {
    pub(crate) const fn with(self, delimiter: Delimiter) -> Self {
        Self(self.0 | delimiter.mask())
    }

    pub(crate) const fn contains(self, delimiter: Delimiter) -> bool {
        self.0 & delimiter.mask() != 0
    }
}

impl Delimiter {
    const fn mask(self) -> u16 {
        1 << self as u16
    }

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

    pub fn at(text: &str) -> Option<Self> {
        match text.as_bytes() {
            [first, second, ..] => Self::from_bytes([*first, *second]),
            _ => None,
        }
    }

    pub fn opens(self, text: &str) -> bool {
        text.as_bytes().starts_with(&self.opening())
    }

    pub fn closes(self, text: &str) -> bool {
        self.closing()
            .is_some_and(|closing| text.as_bytes().starts_with(&closing))
    }

    pub const fn opening(self) -> [u8; 2] {
        match self {
            Self::Strong => *b"**",
            Self::Italic => *b"__",
            Self::Verbatim => *b"``",
            Self::Strikethrough => *b"~~",
            Self::TableHeader => *b"#|",
            Self::TableRow => *b"-|",
            Self::TableCell => *b"||",
            Self::Link => *b"[[",
            Self::LinkName => *b"((",
            Self::Footnote => *b"{{",
        }
    }

    pub const fn closing(self) -> Option<[u8; 2]> {
        match self {
            Self::TableHeader | Self::TableRow | Self::TableCell => None,
            Self::Link => Some(*b"]]"),
            Self::LinkName => Some(*b"))"),
            Self::Footnote => Some(*b"}}"),
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
