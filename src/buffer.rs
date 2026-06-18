#![allow(dead_code)] // ! FIXME: remove this when module in use

use std::{
    path::{Path, PathBuf},
    slice,
};

use crate::row::Row;

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
        self.0.next().map(|x| x.buffer())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
