use std::ops::{Deref, Range};

use crate::byte_pos::BytePos;
use crate::source_map::SourceId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    pub lo: BytePos,
    pub hi: BytePos,
    pub src_id: SourceId,
}

impl Span {
    pub const fn new(lo: BytePos, hi: BytePos, src_id: SourceId) -> Span {
        Span { lo, hi, src_id }
    }

    pub const fn to_range(&self) -> Range<BytePos> {
        self.lo..self.hi
    }

    pub const fn range(&self) -> Range<usize> {
        self.lo.to_usize()..self.hi.to_usize()
    }

    pub const fn len(&self) -> usize {
        self.hi.to_usize() - self.lo.to_usize()
    }

    pub const fn is_empty(&self) -> bool {
        self.lo.to_usize() == self.hi.to_usize()
    }
}

#[macro_export]
macro_rules! span {
    ($source_id:expr => $span_range:expr) => {
        runec_source::span::Span::new($span_range.start, $span_range.end, $source_id)
    };
}

#[derive(Debug, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub const fn new(node: T, span: Span) -> Spanned<T> {
        Spanned { node, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned::new(f(self.node), self.span)
    }

    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned::new(&self.node, self.span)
    }

    pub fn as_mut(&mut self) -> Spanned<&mut T> {
        Spanned::new(&mut self.node, self.span)
    }

    pub fn into_inner(self) -> T {
        self.node
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.node
    }
}

#[cfg(test)]
mod tests {
    use super::{Span, Spanned};
    use crate::byte_pos::BytePos;
    use crate::source_map::SourceId;

    #[test]
    fn span_reports_length_and_empty_state() {
        let source_id = SourceId::from_usize(0);
        let span = Span::new(BytePos::from_usize(3), BytePos::from_usize(8), source_id);
        let empty = Span::new(BytePos::from_usize(5), BytePos::from_usize(5), source_id);

        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn spanned_borrows_preserve_span() {
        let span =
            Span::new(BytePos::from_usize(3), BytePos::from_usize(8), SourceId::from_usize(0));
        let mut value = Spanned::new(String::from("value"), span);

        assert_eq!(value.as_ref(), Spanned::new(&String::from("value"), span));
        value.as_mut().node.push('!');

        assert_eq!(value, Spanned::new(String::from("value!"), span));
    }

    #[test]
    fn spanned_returns_inner_value() {
        let value = Spanned::new(
            String::from("value"),
            Span::new(BytePos::from_usize(0), BytePos::from_usize(5), SourceId::from_usize(0)),
        );

        assert_eq!(value.into_inner(), "value");
    }
}
