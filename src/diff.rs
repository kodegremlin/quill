#![allow(dead_code)] // * FIXME: remove this after module in use.

use crate::row::Row;

/// Where the cursor is at/was-at depending on the concerned diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    pub col: usize,
    pub row: usize,
}

impl CursorPosition {
    pub fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditDiff {
    InsertChar { at: CursorPosition, ch: char },
    DeleteChar { at: CursorPosition, ch: char },
    Insert { at: CursorPosition, text: String },
    Remove { at: CursorPosition, text: String },
    Append { row: usize, text: String },
    Truncate { row: usize, removed: String },
    InsertLine { row: usize, text: String },
    DeleteLine { row: usize, text: String },
}

impl EditDiff {
    pub fn apply(&self, rows: &mut Vec<Row>) -> CursorPosition {
        use CursorPosition as cp;
        use EditDiff::*;

        match self {
            InsertChar { at, ch } => {
                rows[at.row].insert_char(at.col, *ch);
                cp::new(at.col + 1, at.row)
            }
            DeleteChar { at, .. } => {
                rows[at.row].delete_char(at.col);
                cp::new(at.col.saturating_sub(1), at.row)
            }
            Insert { at, text } => {
                rows[at.row].insert_str(at.col, text);
                let len = text.chars().count();
                cp::new(at.col + len, at.row)
            }
            Remove { at, text } => {
                let start_x = at.col - text.chars().count();
                rows[at.row].remove(start_x, at.col);
                cp::new(start_x, at.col)
            }
            Append { row, text } => {
                let len = rows[*row].len();
                rows[*row].append(text);
                cp::new(len, *row)
            }
            Truncate { row, removed } => {
                let count = removed.chars().count();
                let len = rows[*row].len();
                rows[*row].truncate(len - count);
                cp::new(len - count, *row)
            }
            InsertLine { row, text } => {
                rows.insert(
                    *row,
                    Row::new(text).expect("creating a new row should've succeded"),
                );
                cp::new(0, *row)
            }
            DeleteLine { row, .. } => {
                debug_assert!(*row > 0);
                if *row == rows.len() - 1 {
                    rows.pop();
                } else {
                    rows.remove(*row);
                }
                cp::new(rows[*row - 1].len(), *row - 1)
            }
        }
    }

    pub fn inverse(&self) -> Self {
        use EditDiff::*;
        match *self {
            InsertChar { at, ch } => DeleteChar { at: at, ch: ch },
            DeleteChar { at, ch } => InsertChar { at: at, ch: ch },
            Insert { at, ref text } => Remove {
                at,
                text: text.clone(),
            },
            Remove { at, ref text } => Insert {
                at,
                text: text.clone(),
            },
            Append { row, ref text } => Truncate {
                row,
                removed: text.clone(),
            },
            Truncate { row, ref removed } => Append {
                row,
                text: removed.clone(),
            },
            InsertLine { row, ref text } => DeleteLine {
                row,
                text: text.clone(),
            },
            DeleteLine { row, ref text } => InsertLine {
                row,
                text: text.clone(),
            },
        }
    }
}
