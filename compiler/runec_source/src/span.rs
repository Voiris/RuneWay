use std::ops::{Deref, Range};
use crate::byte_pos::BytePos;
use crate::source_map::SourceId;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Span {
    pub lo: BytePos,
    pub hi: BytePos,
    pub src_id: SourceId
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
    pub span: Span
}

impl<T> Spanned<T> {
    pub const fn new(node: T, span: Span) -> Spanned<T> {
        Spanned { node, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned::new(f(self.node), self.span)
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.node
    }
}
