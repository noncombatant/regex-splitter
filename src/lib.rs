// Copyright 2022 by [Chris Palmer](https://noncombatant.org)
// SPDX-License-Identifier: Apache-2.0

//! An `Iterator` that yields `Vec<u8>`s from `Read`s, delimited by regular
//! expressions.

use std::io::{Read, Result};

use regex::bytes::Regex;

/// An `Iterator` that scans a `Read`, searches for the `delimiter`, and yields
/// the non-delimiter bytes.
///
/// This implementation uses a private buffer that will grow proportional to the
/// largest span of bytes between instances of `delimiter`.
pub struct RegexSplitter<'a, 'b> {
    reader: &'a mut dyn Read,
    delimiter: &'b Regex,
    buffer: Vec<u8>,
    // `buffer[start..end]` is the current slice in which we search for
    // `delimiter`.
    start: usize,
    end: usize,
    eof: bool,
}

/// This value is arbitrary, but seems good enough.
pub const DEFAULT_CAPACITY: usize = 64 * 1024;

impl<'a, 'b> RegexSplitter<'a, 'b> {
    /// Returns a new `StreamSplitter` that will split the bytes of `reader`
    /// into `Vec<u8>`s.
    pub fn new(reader: &'a mut dyn Read, delimiter: &'b Regex) -> Self {
        Self::with_capacity(reader, delimiter, DEFAULT_CAPACITY)
    }

    /// Returns a new `StreamSplitter` that will split the bytes of `reader`
    /// into `Vec<u8>`s. The internal buffer will be pre-allocated with at least
    /// `capacity` `u8`s of storage.
    pub fn with_capacity(reader: &'a mut dyn Read, delimiter: &'b Regex, capacity: usize) -> Self {
        Self {
            reader,
            delimiter,
            buffer: vec![0; capacity],
            start: 0,
            end: 0,
            eof: false,
        }
    }

    /// Fills the `StreamSplitter`’s buffer, growing it if it is already full.
    fn fill(&mut self) -> Result<()> {
        if self.end == self.buffer.capacity() {
            if self.start == self.end {
                // We have consumed the buffer. Reset it:
                self.start = 0;
                self.end = 0;
            } else {
                // The buffer is full. To read more, we must grow it:
                self.buffer.resize(2 * self.buffer.capacity(), 0);
            }
        }
        let cap = self.buffer.capacity();
        let n = self.reader.read(&mut self.buffer[self.end..cap])?;
        self.end += n;
        if n == 0 {
            self.eof = true;
        }
        Ok(())
    }
}

pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

impl<'a, 'b> LendingIterator for RegexSplitter<'a, 'b> {
    type Item<'c> = Result<&'c [u8]> where Self: 'c;

    fn next<'c>(&'c mut self) -> Option<Self::Item<'c>> {
        loop {
            if let Err(error) = self.fill() {
                return Some(Err(error));
            }

            if self.start == self.end && self.eof {
                return None;
            }

            let section = &self.buffer[self.start..self.end];
            if let Some(m) = self.delimiter.find(section) {
                if self.start + m.end() == self.end && !self.eof {
                    // `self.buffer` ends in delimiter-matching bytes, yet we
                    // are not at EOF. So we might not have matched the
                    // entirety of the delimiter. Therefore, start back at the
                    // top, which incurs a `fill`, which will grow
                    // `self.buffer`. The `unwrap` is OK because we must at
                    // least match the same match again.
                    continue;
                }
                self.start += m.end();
                let r = if m.start() == 0 {
                    // We matched the delimiter at the beginning of the section.
                    Ok(&section[0..0])
                } else {
                    // We matched a record.
                    Ok(&section[0..m.start()])
                };
                return Some(r);
            } else {
                // Last record, with no trailing delimiter.
                self.start = self.end;
                return Some(Ok(section));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::bytes::Regex;
    use std::io::{Seek, SeekFrom, Write};
    use tempfile::tempfile;

    use crate::{LendingIterator, RegexSplitter};

    // Makes debugging easier than `DEFAULT_CAPACITY`, which fills the terminal
    // with junk.
    const SMALL_CAPACITY: usize = 16;

    #[test]
    fn test_simple() {
        let mut file = tempfile().unwrap();
        file.write_all(b"hello\n\nworld\n").unwrap();

        file.seek(SeekFrom::Start(0)).unwrap();
        let delimiter = Regex::new(r"\s+").unwrap();
        let mut splitter = RegexSplitter::with_capacity(&mut file, &delimiter, SMALL_CAPACITY);

        let r = splitter.next().unwrap().unwrap();
        assert_eq!(b"hello", r);

        let r = splitter.next().unwrap().unwrap();
        assert_eq!(b"world", r);

        assert!(splitter.next().is_none());
    }

    #[test]
    fn test_delimiter_straddles_buffer() {
        let spaces = vec![b' '; SMALL_CAPACITY];

        let mut file = tempfile().unwrap();
        file.write_all(b"greetings").unwrap();
        file.write_all(&spaces).unwrap();
        file.write_all(b"world").unwrap();

        file.seek(SeekFrom::Start(0)).unwrap();
        let delimiter = Regex::new(r"\s+").unwrap();
        let mut splitter = RegexSplitter::with_capacity(&mut file, &delimiter, SMALL_CAPACITY);

        let r = splitter.next().unwrap().unwrap();
        assert_eq!(b"greetings", r);

        let r = splitter.next().unwrap().unwrap();
        assert_eq!(b"world", r);

        assert!(splitter.next().is_none());
    }
}
