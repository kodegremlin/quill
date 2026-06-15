#![allow(dead_code)] // * FIXME: remove after the project uses this module.

/* TODO: document the code and for simple functions just introduce what
the function does */

use anyhow::{Result, anyhow};
use std::ops::{Index, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use unicode_width::UnicodeWidthChar;

/// Number of spaces a tab takes.
const TAB_STOP: usize = 4;

/// Row is the line representation of what a String would look like. It is a different
/// datatype as we need more than just text for proper handling of data, colors, etc.
#[derive(Debug, Default)]
pub struct Row {
    /// This is the raw text. Everything else derives from it.
    buffer: String,

    /// Stores the rendered information. "Hello\tWorld" -> "Hello    World"
    render: String,

    /// Cache of byte indices of characters in `buf`. This will be empty when `buf`
    /// only contains single byte characters so as to not allocate memory.
    indices: Vec<usize>,
}

impl Row {
    /// Returns an empty `Row`.
    pub fn empty() -> Self {
        Self {
            buffer: String::new(),
            render: String::new(),
            indices: Vec::with_capacity(0),
        }
    }

    /// Returns an initialized `Row` from the provided parameter [line] as long
    /// as [line] can be converted into a string using `into()`.
    pub fn new<S: Into<String>>(line: S) -> Result<Self> {
        let mut row = Self {
            buffer: line.into(),
            render: String::new(),
            indices: Vec::with_capacity(0),
        };
        // processes the given line and update the render and indices fields if
        // necessary.
        row.update_render()?;
        Ok(row)
    }

    pub fn len(&self) -> usize {
        if self.indices.is_empty() {
            self.buffer.len()
        } else {
            self.indices.len()
        }
    }

    /// Returns the byte index of the character
    pub fn char_to_byte_idx(&self, char_idx: usize) -> usize {
        let len = self.indices.len();

        if len == char_idx {
            self.buffer.len()
        } else if len == 0 {
            char_idx
        } else {
            self.indices[char_idx]
        }
    }

    pub fn byte_to_char_idx(&self, byte_idx: usize) -> usize {
        if self.indices.is_empty() {
            return byte_idx;
        }
        if self.buffer.len() == byte_idx {
            return self.indices.len(); // pointing after the last character in the vec.
        }
        // TODO: could be optimized to O(n) by storing the byte indices in cache
        // as well but introduces more maintenence for little gain.
        // Will do later if needed.
        self.indices
            .iter()
            .position(|&bi| bi == byte_idx)
            .expect("byte index is not at the correct boundary of UTF-8")
    }

    /// Processes the buffer and renders the ASCII such as '\t' '\n' accordingly
    /// and stores them in `Row.render`.
    /// If there are any multi-byte characters, their indices are stored as
    /// cache so as to avoid lookup time in buffer or render again.
    fn update_render(&mut self) -> Result<()> {
        self.render.clear();
        self.render.reserve(self.buffer.len());

        let mut index = 0;
        let mut num_chars = 0;

        for ch in self.buffer.chars() {
            if let Some(width) = ch.width_cjk() {
                index += width;
                self.render.push(ch);
            } else if ch == '\t' {
                loop {
                    self.render.push(' ');
                    index += 1;
                    if index % TAB_STOP == 0 {
                        break;
                    }
                }
            } else {
                // Control characters are valid UTF-8 but they should not appear
                // in text and we won't be handling them.
                return Err(anyhow!("Control character in text: {}", ch));
            }
            num_chars += 1;
        }
        if num_chars == self.buffer.len() {
            // If number of chars is the same as byte length, this line includes
            // no multi-byte character. Hence, no memory is allocated to the
            // heap.
            self.indices = Vec::with_capacity(0);
        } else {
            self.indices.clear();
            self.indices.reserve(num_chars);
            for (idx, _) in self.buffer.char_indices() {
                self.indices.push(idx);
            }
        }
        Ok(())
    }

    pub fn buffer(&self) -> &str {
        self.buffer.as_str()
    }

    pub fn render_text(&self) -> &str {
        self.render.as_str()
    }

    /// Returns the character at the provided index.
    ///
    /// # Panic
    /// If the character is not found at the provided index.
    pub fn char_at(&self, idx: usize) -> char {
        self.chat_at_checked(idx)
            .expect("character should have been present at the provided index")
    }

    /// Returns an Option with the character at the provided index.
    pub fn chat_at_checked(&self, idx: usize) -> Option<char> {
        // if it doesn't work properly check the trait implementations.
        self[idx..].chars().next()
    }

    pub fn rx_from_cx(&self, cx: usize) -> usize {
        self[..cx].chars().fold(0, |rx, ch| {
            if ch == '\t' {
                rx + TAB_STOP - (rx % TAB_STOP)
            } else {
                rx + ch.width_cjk().unwrap()
            }
        })
    }

    pub fn insert_char(&mut self, idx: usize, ch: char) {
        if self.len() <= idx {
            self.buffer.push(ch);
        } else {
            let b_idx = self.char_to_byte_idx(idx);
            self.buffer.insert(b_idx, ch);
        }
        self.update_render().unwrap();
    }

    pub fn insert_str<S: AsRef<str>>(&mut self, idx: usize, strs: S) {
        if self.len() <= idx {
            self.buffer.push_str(strs.as_ref());
        } else {
            let b_idx = self.char_to_byte_idx(idx);
            self.buffer.insert_str(b_idx, strs.as_ref());
        }
        self.update_render().unwrap();
    }

    pub fn delete_char(&mut self, idx: usize) {
        if idx < self.len() {
            let b_idx = self.char_to_byte_idx(idx);
            self.buffer.remove(b_idx);
            self.update_render().unwrap();
        }
    }

    pub fn append<S: AsRef<str>>(&mut self, strs: S) {
        let strs = strs.as_ref();
        if strs.is_empty() {
            return;
        }
        self.buffer.push_str(strs);
        self.update_render().unwrap();
    }

    pub fn truncate(&mut self, idx: usize) {
        if idx < self.len() {
            let b_idx = self.char_to_byte_idx(idx);
            self.truncate(b_idx);
            self.update_render().unwrap();
        }
    }

    /// `delete_char` can be used instead, double check why remove_char is needed.
    /// The current implementation differs only in bounds check.
    pub fn remove_char(&mut self, idx: usize) {
        let b_idx = self.char_to_byte_idx(idx);
        self.buffer.remove(b_idx);
        self.update_render().unwrap();
    }

    pub fn remove(&mut self, start: usize, end: usize) {
        if start < end {
            let b_start = self.char_to_byte_idx(start);
            let b_end = self.char_to_byte_idx(end);
            self.buffer.drain(b_start..b_end);
            self.update_render().unwrap();
        }
    }
}

// Implementation of the Index trait with every Range type.

impl Index<Range<usize>> for Row {
    type Output = str;

    fn index(&self, r: Range<usize>) -> &Self::Output {
        let start = self.char_to_byte_idx(r.start);
        let end = self.char_to_byte_idx(r.end);
        &self.buffer[start..end]
    }
}

impl Index<RangeFrom<usize>> for Row {
    type Output = str;

    fn index(&self, r: RangeFrom<usize>) -> &Self::Output {
        let start = self.char_to_byte_idx(r.start);
        &self.buffer[start..]
    }
}

impl Index<RangeTo<usize>> for Row {
    type Output = str;

    fn index(&self, r: RangeTo<usize>) -> &Self::Output {
        let end = self.char_to_byte_idx(r.end);
        &self.buffer[..end]
    }
}

impl Index<RangeInclusive<usize>> for Row {
    type Output = str;

    fn index(&self, r: RangeInclusive<usize>) -> &Self::Output {
        let start = self.char_to_byte_idx(*r.start());
        let end = self.char_to_byte_idx(*r.end());
        &self.buffer[start..=end]
    }
}

impl Index<RangeToInclusive<usize>> for Row {
    type Output = str;

    fn index(&self, r: RangeToInclusive<usize>) -> &Self::Output {
        let end = self.char_to_byte_idx(r.end);
        &self.buffer[..=end]
    }
}
