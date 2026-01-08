use std::iter::Peekable;
use std::str::CharIndices;
use runec_source::byte_pos::BytePos;

#[derive(Clone, Debug)]
pub(super) struct Cursor<'src> {
    source: &'src str,
    iter: Peekable<CharIndices<'src>>,
    pos: usize,
}

impl<'src> Cursor<'src> {
    /// Creates a new cursor for the given string slice.
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            iter: source.char_indices().peekable(),
            pos: 0,
        }
    }

    /// Returns the current character first byte position in the source string.
    ///
    /// Note: It accounts for Unicode character lengths.
    pub fn pos(&self) -> BytePos {
        BytePos::from_usize(self.pos)
    }

    /// Returns the next character and its first byte index, advancing the cursor.
    pub fn next(&mut self) -> Option<(usize, char)> {
        self.iter.next().map(|(idx, char)| {
            self.pos = idx + char.len_utf8();
            (idx, char)
        })
    }

    /// Returns a reference to the next character and its first byte index without advancing the cursor.
    pub fn peek(&mut self) -> Option<&(usize, char)> {
        self.iter.peek()
    }

    /// Returns the `n`-th character ahead and its first byte index, advancing the cursor.
    pub fn nth(&mut self, n: usize) -> Option<(usize, char)> {
        for _ in 0..n {
            // Updating position
            self.next();
        }
        self.next()
    }

    /// Returns the `n`-th character ahead, advancing the cursor.
    pub fn nth_char(&mut self, n: usize) -> Option<char> {
        self.nth(n).map(|(_, c)| c)
    }

    /// Returns the next character, advancing the cursor.
    pub fn next_char(&mut self) -> Option<char> {
        self.next().map(|(_, c)| c)
    }

    /// Returns a reference to the next character without advancing the cursor.
    pub fn peek_char(&mut self) -> Option<&char> {
        self.iter.peek().map(|(_, c)| c)
    }

    /// Returns a slice of the next len characters and advances the cursor. Returns None if there aren’t enough characters.
    pub fn try_next_slice(&mut self, len: usize) -> Option<&'src str> {
        let start = self.pos;

        if len == 0 {
            return Some(&self.source[start..start])
        }

        for _ in 0..len {
            self.next()?;
        }

        Some(&self.source[start..self.pos])
    }

    /// Advances the cursor until the given character is found or the end is reached.
    pub fn skip_until_char(&mut self, c: char) {
        while let Some(&char) = self.peek_char() {
            if char == c {
                break;
            } else {
                self.next_char();
            }
        }
    }

    /// Advances the cursor until the given character is found or the end is reached or the max characters count.
    pub fn skip_until_char_counted(&mut self, c: char, max: usize) {
        let mut i = 0;
        while let Some(&char) = self.peek_char() {
            if i == max || char == c {
                break;
            } else {
                i += 1;
                self.next_char();
            }
        }
    }

    /// Returns the `n`-th character ahead and its first byte index without advancing the cursor.
    pub fn lookahead(&mut self, n: usize) -> Option<(usize, char)> {
        self.clone().nth(n)
    }

    /// Returns the `n`-th character ahead without advancing the cursor.
    pub fn lookahead_char(&mut self, n: usize) -> Option<char> {
        self.clone().nth_char(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_peek_cursor_test() {
        let mut cursor = Cursor::new("abc");
        assert_eq!(cursor.pos().to_usize(), 0);
        assert_eq!(cursor.peek(), Some(&(0, 'a')));
        assert_eq!(cursor.pos().to_usize(), 0);
        assert_eq!(cursor.next(), Some((0, 'a')));
        assert_eq!(cursor.pos().to_usize(), 1);
        assert_eq!(cursor.peek(), Some(&(1, 'b')));
        assert_eq!(cursor.pos().to_usize(), 1);
        assert_eq!(cursor.next(), Some((1, 'b')));
        assert_eq!(cursor.pos().to_usize(), 2);
        assert_eq!(cursor.peek(), Some(&(2, 'c')));
        assert_eq!(cursor.pos().to_usize(), 2);
        assert_eq!(cursor.next(), Some((2, 'c')));
        assert_eq!(cursor.pos().to_usize(), 3);
        assert_eq!(cursor.peek(), None);
        assert_eq!(cursor.pos().to_usize(), 3);
        assert_eq!(cursor.next(), None);
    }

    #[test]
    fn different_unicode_length_test() {
        let mut cursor = Cursor::new("aécπ😀cπ🚀");
        assert_eq!(cursor.pos().to_usize(), 0);
        assert_eq!(cursor.next(), Some((0, 'a')));
        assert_eq!(cursor.pos().to_usize(), 1);
        assert_eq!(cursor.next(), Some((1, 'é')));
        assert_eq!(cursor.pos().to_usize(), 3);
        assert_eq!(cursor.next(), Some((3, 'c')));
        assert_eq!(cursor.pos().to_usize(), 4);
        assert_eq!(cursor.next(), Some((4, 'π')));
        assert_eq!(cursor.pos().to_usize(), 6);
        assert_eq!(cursor.next(), Some((6, '😀')));
        assert_eq!(cursor.pos().to_usize(), 10);
        assert_eq!(cursor.next(), Some((10, 'c')));
        assert_eq!(cursor.pos().to_usize(), 11);
        assert_eq!(cursor.next(), Some((11, 'π')));
        assert_eq!(cursor.pos().to_usize(), 13);
        assert_eq!(cursor.next(), Some((13, '🚀')));
        assert_eq!(cursor.pos().to_usize(), 17);
        assert_eq!(cursor.next(), None);
    }

    #[test]
    fn slice_test() {
        let mut cursor = Cursor::new("ab😀πcd🚀é ");
        assert_eq!(cursor.pos().to_usize(), 0);
        assert_eq!(cursor.try_next_slice(4), Some("ab😀π"));
        assert_eq!(cursor.pos().to_usize(), 8);
        assert_eq!(cursor.try_next_slice(4), Some("cd🚀é"));
        assert_eq!(cursor.pos().to_usize(), 16);
        assert_eq!(cursor.try_next_slice(2), None);
        assert_eq!(cursor.pos().to_usize(), 17);
    }

    #[test]
    fn skip_until_char_test() {
        let mut cursor = Cursor::new("Hello, World! 😀 How aré you?");
        cursor.skip_until_char(' ');    // "Hello,"
        assert_eq!(cursor.pos().to_usize(), 6);
        cursor.next();
        cursor.skip_until_char(' ');    // "World!"
        assert_eq!(cursor.pos().to_usize(), 13);
        cursor.next();
        assert_eq!(cursor.pos().to_usize(), 14);
        cursor.skip_until_char(' ');    // "😀"
        assert_eq!(cursor.pos().to_usize(), 18);
        cursor.next();
        assert_eq!(cursor.pos().to_usize(), 19);
        cursor.skip_until_char(' ');    // "How"
        assert_eq!(cursor.pos().to_usize(), 22);
        cursor.next();
        assert_eq!(cursor.pos().to_usize(), 23);
        cursor.skip_until_char(' ');    // "aré"
        assert_eq!(cursor.pos().to_usize(), 27);
        cursor.next();
        assert_eq!(cursor.pos().to_usize(), 28);
        cursor.skip_until_char(' ');    // "you?"
        assert_eq!(cursor.pos().to_usize(), 32);
        cursor.skip_until_char(' ');    // "" - empty
        assert_eq!(cursor.pos().to_usize(), 32);
    }

    #[test]
    fn nth_test() {
        let mut cursor = Cursor::new("aécπ😀cπ🚀");
        assert_eq!(cursor.pos().to_usize(), 0);
        assert_eq!(cursor.nth(2), Some((3, 'c')));
        assert_eq!(cursor.pos().to_usize(), 4);
        assert_eq!(cursor.nth(2), Some((10, 'c')));
        assert_eq!(cursor.pos().to_usize(), 11);
        assert_eq!(cursor.nth(5), None);
        assert_eq!(cursor.pos().to_usize(), 17);
    }

    #[test]
    fn lookahead_test() {
        let mut cursor = Cursor::new("aécπ😀cπ🚀");
        assert_eq!(cursor.pos().to_usize(), 0);
        assert_eq!(cursor.lookahead(2), Some((3, 'c')));
        assert_eq!(cursor.pos().to_usize(), 0);
        assert_eq!(cursor.lookahead(5), Some((10, 'c')));
        assert_eq!(cursor.pos().to_usize(), 0);
        assert_eq!(cursor.lookahead(8), None);
        assert_eq!(cursor.pos().to_usize(), 0);
    }
}
