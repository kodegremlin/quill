#![allow(dead_code)] // ! FIXME: remove this when module in use

use std::{
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
    slice,
};

use anyhow::Result;

use crate::{
    diff::{CursorPosition, EditDiff},
    history::History,
    lang::{Indent, Language},
    row::Row,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDir {
    Left,
    Right,
    Up,
    Down,
}

/// Contains both, actual sequence and display string.
#[derive(Debug)]
pub struct FilePath {
    pub path: PathBuf,
    pub display: String,
}

impl FilePath {
    fn from_string<S: Into<String>>(string: S) -> Self {
        let display = string.into();
        Self {
            path: PathBuf::from(&display),
            display,
        }
    }

    fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        Self {
            path: PathBuf::from(path),
            display: path.to_string_lossy().to_string(),
        }
    }
}

pub struct Lines<'a>(slice::Iter<'a, Row>);

impl<'a> ExactSizeIterator for Lines<'a> {}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|x| x.raw_text())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Debug, Default)]
pub struct TextBuffer {
    /// (col/x) co-ordinate in the internal text buffer of rows - the raw text.
    col_idx: usize,

    /// (row/y) co-ordinate in the internal text buffer of rows - the raw text.
    row_idx: usize,

    /// File the editor is actually opening.
    file: Option<FilePath>,

    /// Lines inside the text buffer.
    rows: Vec<Row>,

    /// Count of how many times undo points were created in the buffer. This
    /// value is set to 0 after just loading the buffer. When saving the
    /// buffer to file, count is reset to 0.
    /// When redo/undo is applied without ongoing changes, this count is just
    /// incremented or decremented.
    undo_count: i32,

    /// True when there are uncommitted edits in the `ongoing` buffer.
    /// This flag is necessary because `undo_count` only tracks modifications
    /// that have been fully sealed into history. This flag catches the
    /// "in-progress" batch before an undo-point is created.
    modified: bool,

    /// Language which current buffer belongs to.
    lang: Language,

    /// History per undo point for undo/redo.
    history: History,

    /// Flag to ensure at most one undo point per one key input.
    inserted_undo: bool,

    /// Flag to require screen re-rendering. The value represents the row from
    /// where we need to re-render the editor.
    redraw_from: Option<usize>,
}

impl TextBuffer {
    pub fn empty() -> Self {
        Self {
            rows: vec![Row::empty()],
            redraw_from: Some(0),
            ..Default::default()
        }
    }

    pub fn with_lines<S, I>(lines: I) -> Result<Self>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let rows = lines.map(|s| Row::new(s.as_ref())).collect::<Result<_>>()?;
        let mut buf = Self::empty();
        buf.rows = rows;
        Ok(buf)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = FilePath::from_path(path);

        if !path.try_exists()? {
            // When the path does not exist, consider it a new file.
            let mut buf = Self::empty();
            buf.file = Some(file);
            buf.lang = Language::detect(path);
            return Ok(buf);
        }
        let rows = io::BufReader::new(File::open(path)?)
            .lines()
            .map(|r| Row::new(r?))
            .collect::<Result<_>>()?;

        let mut buf = Self::empty();
        buf.file = Some(file);
        buf.rows = rows;
        buf.lang = Language::detect(path);
        Ok(buf)
    }

    fn set_redraw_idx(&mut self, line: usize) {
        if let Some(row_idx) = self.redraw_from {
            if row_idx <= line {
                return;
            }
            self.redraw_from = Some(line)
        }
    }

    fn set_cursor(&mut self, cursor: CursorPosition) {
        self.col_idx = cursor.col;
        self.row_idx = cursor.row;
    }
}

impl TextBuffer {
    pub fn insert_char(&mut self, ch: char) {
        // we don't add a undo point here to group multiple insert_char changes
        // into one undo point.
        if self.row_idx == self.rows.len() {
            self.new_diff(EditDiff::InsertLine {
                row: self.row_idx,
                text: "".to_string(),
            });
        }
        self.new_diff(EditDiff::InsertChar {
            at: CursorPosition {
                col: self.col_idx,
                row: self.row_idx,
            },
            ch,
        });
    }

    pub fn insert_tab(&mut self) {
        self.insert_undo_point();
        match self.lang.indent() {
            Indent::FourSpaces(indent) => {
                let cursor = CursorPosition {
                    col: self.col_idx,
                    row: self.row_idx,
                };
                self.new_diff(EditDiff::Insert {
                    at: cursor,
                    text: indent.to_owned(),
                });
            }
            Indent::Tab => self.insert_char('\t'),
        }
    }

    pub fn delete_char(&mut self) {
        if self.row_idx == self.rows.len() || self.col_idx == 0 && self.row_idx == 0 {
            return;
        }
        self.insert_undo_point();
        if self.col_idx > 0 {
            let col = self.col_idx - 1;
            let deleted = self.rows[self.row_idx].char_at(col);
            let cursor = CursorPosition {
                col: self.col_idx,
                row: self.row_idx,
            };
            self.new_diff(EditDiff::DeleteChar {
                at: cursor,
                ch: deleted,
            });
        } else {
            self.squash_to_previous_line();
        }
    }

    pub fn delete_right_char(&mut self) {
        if self.row_idx == self.rows.len()
            || self.row_idx == self.rows.len() - 1 && self.col_idx == self.rows[self.row_idx].len()
        {
            // nothing can be deleted at the end of the buffer and the cursor
            // should not move.
            return;
        }
        self.move_cursor_one(CursorDir::Right);
        self.delete_char();
    }

    pub fn insert_line(&mut self) {
        self.insert_undo_point();

        if self.row_idx >= self.rows.len() {
            self.new_diff(EditDiff::InsertLine {
                row: self.row_idx,
                text: "".to_string(),
            });
        } else if self.col_idx >= self.rows[self.row_idx].len() {
            self.new_diff(EditDiff::InsertLine {
                row: self.row_idx + 1,
                text: "".to_string(),
            });
        } else if self.col_idx <= self.rows[self.row_idx].raw_text().len() {
            let truncated = self.rows[self.row_idx][self.col_idx..].to_owned();
            self.new_diff(EditDiff::Truncate {
                row: self.row_idx,
                removed: truncated.clone(),
            });
            self.new_diff(EditDiff::InsertLine {
                row: self.row_idx + 1,
                text: truncated,
            });
        }
    }

    pub fn move_cursor_one(&mut self, _dir: CursorDir) {
        todo!()
    }

    pub fn delete_word(&mut self) {
        if self.col_idx == 0 || self.row_idx == self.rows.len() {
            return;
        }
        self.insert_undo_point();

        let line = &self.rows[self.row_idx];
        let mut colx = self.col_idx - 1;

        // if we are on a whitespace we'll keep going back until we encounter an
        // alphanumeric character.
        while colx > 0 && line.char_at(colx).is_ascii_whitespace() {
            colx -= 1;
        }

        // we keep going back until we encounter a space before cur word meaning
        // we are pointing at the start of the word.
        while colx > 0 && !line.char_at(colx - 1).is_ascii_whitespace() {
            colx -= 1;
        }
        let removed = line[colx..self.col_idx].to_owned();

        let cursor = CursorPosition {
            col: colx,
            row: self.row_idx,
        };
        self.new_diff(EditDiff::Remove {
            at: cursor,
            text: removed,
        });
    }

    pub fn delete_until_line_end(&mut self) {
        if self.row_idx == self.rows.len() {
            return;
        }
        self.insert_undo_point();

        let row = &self.rows[self.row_idx];
        if self.col_idx == row.len() {
            // do nothing when cursor is at the end of line and at the end of the
            // text buffer; basically the last line and the last character in it.
            if self.row_idx == self.rows.len() - 1 {
                return;
            }
            self.concat_next_line();
        } else if self.col_idx < row.raw_text().len() {
            let truncated = row[self.col_idx..].to_owned();
            self.new_diff(EditDiff::Truncate {
                row: self.row_idx,
                removed: truncated,
            });
        }
    }

    pub fn delete_until_line_head(&mut self) {
        if self.col_idx == 0 && self.row_idx == 0 || self.row_idx == self.rows.len() {
            return;
        }
        self.insert_undo_point();

        if self.col_idx == 0 {
            self.squash_to_previous_line();
        } else {
            let removed = self.rows[self.row_idx][..self.col_idx].to_owned();
            let cursor = CursorPosition {
                col: 0,
                row: self.row_idx,
            };
            self.new_diff(EditDiff::Remove {
                at: cursor,
                text: removed,
            });
        }
    }

    /// At the beginning of a line, backspace concats current line to previous line.
    fn squash_to_previous_line(&mut self) {
        // move cursor to previous line/row.
        self.row_idx -= 1;
        // move cursor to the end of now current row.
        self.col_idx = self.rows[self.row_idx].len();
        self.concat_next_line();
    }

    fn concat_next_line(&mut self) {
        let removed = self.rows[self.row_idx + 1].take_buffer();
        self.new_diff(EditDiff::DeleteLine {
            row: self.row_idx + 1,
            text: removed.clone(),
        });
        self.new_diff(EditDiff::Append {
            row: self.row_idx,
            text: removed,
        });
    }

    fn new_diff(&mut self, diff: EditDiff) {
        let cursor = diff.apply(&mut self.rows);
        self.set_cursor(cursor);
        self.set_redraw_idx(cursor.row);
        self.modified = true;
        self.history.push_diff(diff);
    }

    pub fn commit_edit(&mut self) -> Option<usize> {
        self.inserted_undo = false;
        let redraw_idx = self.redraw_from;
        self.redraw_from = None;
        redraw_idx
    }

    fn insert_undo_point(&mut self) {
        if !self.inserted_undo {
            if self.history.queue_ongoing_edits() {
                self.undo_count = self.undo_count.saturating_add(1);
            }
            self.modified = false;
            self.inserted_undo = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emtpy_is_empty() {
        let empty = TextBuffer::empty();
        dbg!(&empty);
        let strings = vec!["thiß german text", "is", "a", "string", "iter"];
        let with_lines = TextBuffer::with_lines(strings.iter());
        dbg!(with_lines.unwrap());
    }
}
