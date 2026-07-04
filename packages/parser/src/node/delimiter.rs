use serde::Serialize;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, Serialize, EnumIter, PartialEq)]
pub enum Delimiter {
    Strong,
    Italic,
    Verbatim,
    Strikethrough,
    TableHeader,
    TableRow,
    TableCell,
    Link,
    LinkName,
    LinkFootnote,
}

impl Delimiter {
    pub fn from_bytes(first: &u8, second: &u8) -> Option<Self> {
        Delimiter::iter().find(|delimiter| {
            delimiter.opening_delimiter() == (first, second)
                || delimiter.closing_delimiter() == Some((first, second))
        })
        // match first {
        //     b'*' => {
        //         if second == b'*' {
        //             Some(Self::Strong)
        //         } else {
        //             None
        //         }
        //     }
        //     b'_' => {
        //         if second == b'_' {
        //             Some(Self::Italic)
        //         } else {
        //             None
        //         }
        //     }
        //     b'`' => {
        //         if second == b'`' {
        //             Some(Self::Verbatim)
        //         } else {
        //             None
        //         }
        //     }
        //     b'~' => {
        //         if second == b'~' {
        //             Some(Self::Strikethrough)
        //         } else {
        //             None
        //         }
        //     }
        //     b'[' => {
        //         if second == b'[' {
        //             Some(Self::Link)
        //         } else {
        //             None
        //         }
        //     }
        //     b']' => {
        //         if second == b']' {
        //             Some(Self::Link)
        //         } else {
        //             None
        //         }
        //     }
        //     b'(' => {
        //         if second == b'(' {
        //             Some(Self::LinkName)
        //         } else {
        //             None
        //         }
        //     }
        //     b')' => {
        //         if second == b')' {
        //             Some(Self::LinkName)
        //         } else {
        //             None
        //         }
        //     }
        //     b'{' => {
        //         if second == b'{' {
        //             Some(Self::LinkFootnote)
        //         } else {
        //             None
        //         }
        //     }
        //     b'}' => {
        //         if second == b'}' {
        //             Some(Self::LinkFootnote)
        //         } else {
        //             None
        //         }
        //     }
        //     b'|' => {
        //         if second == b'|' {
        //             Some(Self::TableCell)
        //         } else {
        //             None
        //         }
        //     }
        //     b'#' => {
        //         if second == b'|' {
        //             Some(Self::TableHeader)
        //         } else {
        //             None
        //         }
        //     }
        //     b'-' => {
        //         if second == b'|' {
        //             Some(Self::TableRow)
        //         } else {
        //             None
        //         }
        //     }
        //     _ => None,
        // }
    }
    pub const fn opening_delimiter<'a>(self) -> (&'a u8, &'a u8) {
        match self {
            Self::Strong => (&b'*', &b'*'),
            Self::Italic => (&b'_', &b'_'),
            Self::Verbatim => (&b'`', &b'`'),
            Self::Strikethrough => (&b'~', &b'~'),
            Self::TableHeader => (&b'#', &b'|'),
            Self::TableRow => (&b'-', &b'|'),
            Self::TableCell => (&b'|', &b'|'),
            Self::Link => (&b'[', &b'['),
            Self::LinkName => (&b'(', &b'('),
            Self::LinkFootnote => (&b'{', &b'{'),
        }
    }

    pub const fn closing_delimiter<'a>(self) -> Option<(&'a u8, &'a u8)> {
        match self {
            Self::Strong => Some((&b'*', &b'*')),
            Self::Italic => Some((&b'_', &b'_')),
            Self::Verbatim => Some((&b'`', &b'`')),
            Self::Strikethrough => Some((&b'~', &b'~')),
            Self::TableHeader => None,
            Self::TableRow => None,
            Self::TableCell => None,
            Self::Link => Some((&b']', &b']')),
            Self::LinkName => Some((&b')', &b')')),
            Self::LinkFootnote => Some((&b'}', &b'}')),
        }
    }
}
