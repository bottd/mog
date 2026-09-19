use serde::Serialize;

/// A node's position in the source text handed to [`parse`](crate::parse).
///
/// `start` and `end` are a half-open range of **UTF-8 byte offsets**. A host
/// slicing a JavaScript string indexes UTF-16 code units instead, so it splices
/// on the line numbers or on a byte buffer, never on these offsets directly.
///
/// A line terminator belongs to no span: a span ends at the last byte of its
/// last line. So text inserted "directly after" a span begins at line
/// `end_line + 1`, whatever the file's line endings are.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    /// 0-based line holding `start`.
    pub start_line: usize,
    /// 0-based line holding `end - 1`.
    pub end_line: usize,
}

impl Span {
    pub(crate) fn line(start: usize, end: usize, line: usize) -> Self {
        Span {
            start,
            end,
            start_line: line,
            end_line: line,
        }
    }

    /// An opening line joined to the closing line that ends the same node.
    pub(crate) fn extend(self, closing: Span) -> Self {
        Span {
            end: closing.end,
            end_line: closing.end_line,
            ..self
        }
    }
}
