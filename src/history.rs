#![allow(dead_code)] // FIXME: remove after module in use.

use std::{collections::VecDeque, mem, usize};

use crate::{
    diff::{CursorPosition, EditDiff},
    row::Row,
};

// * TODO: document the important methods like undo redo and queue...

const MAX_ENTRIES: usize = 1000;

pub type Edits = Vec<EditDiff>;

#[derive(Debug, Default, Clone)]
pub struct History {
    index: usize,
    ongoing: Edits,
    entries: VecDeque<Edits>,
}

impl History {
    pub fn queue_ongoing_edits(&mut self) -> bool {
        if self.ongoing.is_empty() {
            return false;
        }
        let entry_len = self.entries.len();
        debug_assert!(entry_len <= MAX_ENTRIES);

        let diffs = mem::take(&mut self.ongoing);

        if self.entries.len() == MAX_ENTRIES {
            self.entries.pop_front();
            self.index -= 1;
        }
        if self.index < self.entries.len() {
            self.entries.truncate(self.index);
        }
        self.index += 1;
        self.entries.push_back(diffs);
        true
    }

    pub fn push_diff(&mut self, diff: EditDiff) {
        self.ongoing.push(diff);
    }

    pub fn undo(&mut self, rows: &mut Vec<Row>) -> Option<(CursorPosition, usize, bool)> {
        let edited = self.queue_ongoing_edits();
        if self.index == 0 {
            return None;
        }
        self.index -= 1;

        let (cursor, redraw_idx) = self.entries[self.index].iter().rev().fold(
            (CursorPosition::new(0, 0), usize::MAX),
            |(_, redraw_idx), diff| {
                let cursor = diff.inverse().apply(rows);
                (cursor, redraw_idx.min(cursor.row))
            },
        );
        Some((cursor, redraw_idx, edited))
    }

    pub fn redo(&mut self, rows: &mut Vec<Row>) -> Option<(CursorPosition, usize, bool)> {
        let edited = self.queue_ongoing_edits();
        if self.index == self.entries.len() {
            return None;
        }
        self.index += 1;

        let (cursor, redraw_idx) = self.entries[self.index - 1].iter().fold(
            (CursorPosition::new(0, 0), usize::MAX),
            |(_, redraw_idx), diff| {
                let cursor = diff.apply(rows);
                (cursor, redraw_idx.min(cursor.row))
            },
        );
        Some((cursor, redraw_idx, edited))
    }
}
