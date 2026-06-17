#![allow(dead_code)] // * FIXME: remove this after module in use.

/// Where the cursor at/was-at depending on the concerned diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditDiff {
    InsertChar { at: CursorPosition, ch: char },
    DeleteChar { at: CursorPosition, ch: char },
    InsertLine { at: CursorPosition, ch: char },
    DeleteLine { at: CursorPosition, ch: char },
}

impl EditDiff {
    pub fn inverse() {
        
    }
}
