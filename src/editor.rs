use std::{io::Write, path::Path};

use anyhow::Result;

use crate::{
    buffer::{CursorDir, TextBuffer},
    highlight::Highlighting,
    renderer::Renderer,
    status_bar::{Position, StatusBar},
    terminal::{Event, Size},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditStep {
    Continue,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditAction {
    Move(CursorDir),
    MovePage(CursorDir),
    MoveToEdge(CursorDir),
    MoveByWord(CursorDir),
    MoveParagraph(CursorDir),
    InsertChar(char),
    InsertTab,
    InsertLine,
    DeleteChar,
    DeleteRightChar,
    DeleteWord,
    DeleteUntilLineEnd,
    DeleteUntilLineHead,
    Undo,
    Redo,
}

#[derive(Debug)]
pub struct Document {
    pub buffer: TextBuffer,
    pub highlight: Highlighting,
}

impl Document {
    pub fn new(buffer: TextBuffer) -> Self {
        let highlight = Highlighting::new(buffer.lang, buffer.rows());
        Self { buffer, highlight }
    }

    pub fn execute<W: Write>(&mut self, action: EditAction, renderer: &Renderer<W>) {
        use EditAction::*;
        match action {
            Move(dir) => self.buffer.step(dir),
            MovePage(dir) => {
                self.buffer
                    .jump_page_up_down(dir, renderer.rowoff, renderer.rows());
            }
            MoveToEdge(dir) => self.buffer.jump_to_edge(dir),
            MoveByWord(dir) => self.buffer.step_by_word(dir),
            MoveParagraph(dir) => self.buffer.jump_paragraphs(dir),
            InsertChar(ch) => self.buffer.insert_char(ch),
            InsertTab => self.buffer.insert_tab(),
            InsertLine => self.buffer.insert_line(),
            DeleteChar => self.buffer.delete_char(),
            DeleteRightChar => self.buffer.delete_right_char(),
            DeleteWord => self.buffer.delete_word(),
            DeleteUntilLineEnd => self.buffer.delete_until_line_end(),
            DeleteUntilLineHead => self.buffer.delete_until_line_head(),
            Undo => {
                if !self.buffer.undo() {
                    log::debug!(
                        target: "editor.rs/Document::execute",
                        "undo returned false; modified={} curr_row={:?}",
                        self.buffer.modified(), self.buffer.rows()[self.buffer.row_idx()]
                    );
                }
            }
            Redo => {
                if !self.buffer.redo() {
                    log::debug!(
                        target: "editor.rs/Document::execute",
                        "redo returned false; modified={} curr_row={:?}",
                        self.buffer.modified(), self.buffer.rows()[self.buffer.row_idx()]
                    );
                }
            }
        }
    }
}

pub struct Editor<I, W>
where
    W: Write,
    I: Iterator<Item = Result<Event>>,
{
    documents: Vec<Document>,
    doc_idx: usize,
    renderer: Renderer<W>,
    input: I,
    quitting: bool,
    status_bar: StatusBar,
}

impl<I, W> Editor<I, W>
where
    W: Write,
    I: Iterator<Item = Result<Event>>,
{
    pub fn new(input: I, output: W, size: Size) -> Result<Self> {
        let renderer = Renderer::new(size, output)?;
        let buffer = TextBuffer::empty();
        let status_bar = StatusBar::from_buffer(&buffer, Position { curr: 1, size: 1 });

        let document = Document::new(buffer);
        Ok(Self {
            documents: vec![document],
            doc_idx: 0,
            renderer,
            input,
            quitting: false,
            status_bar,
        })
    }

    pub fn open<P>(input: I, output: W, size: Size, paths: &[P]) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        if paths.is_empty() {
            return Self::new(input, output, size);
        }
        let renderer = Renderer::new(size, output)?;

        let mut documents = Vec::with_capacity(paths.len());
        for path in paths {
            let buffer = TextBuffer::open(path)?;
            documents.push(Document::new(buffer));
        }
        let status_bar = StatusBar::from_buffer(
            &documents[0].buffer,
            Position {
                curr: 1,
                size: documents.len(),
            },
        );
        Ok(Self {
            documents,
            doc_idx: 0,
            renderer,
            input,
            quitting: false,
            status_bar,
        })
    }

    pub fn doc(&self) -> &Document {
        &self.documents[self.doc_idx]
    }

    pub fn doc_mut(&mut self) -> &mut Document {
        &mut self.documents[self.doc_idx]
    }
}
