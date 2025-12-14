use std::ops::Range;
use crate::byte_pos::BytePos;
use crate::source_map::SourceId;

pub struct Span {
    pub lo: BytePos,
    pub hi: BytePos,
    pub src_id: SourceId
}

impl Span {
    pub fn new(lo: BytePos, hi: BytePos, src_id: SourceId) -> Span {
        Span { lo, hi, src_id }
    }

    pub fn to_range(&self) -> Range<BytePos> {
        self.lo..self.hi
    }

    pub const fn range(&self) -> Range<usize> {
        self.lo.to_usize()..self.hi.to_usize()
    }
}
