use std::{io::Write, path::Path};

use anyhow::Result;

use crate::{
    buffer::{CursorDir, TextBuffer},
    command::{Action, Command, CommandResult, NoAction, TextSearch},
    help::HELP,
    highlight::Highlighting,
    renderer::Renderer,
    status_bar::{Position, StatusBar},
    terminal::{Event, Key, KeySeq, Size},
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

    pub fn doc_mut(&mut self) -> &mut Document {
        &mut self.documents[self.doc_idx]
    }

    pub fn doc(&self) -> &Document {
        &self.documents[self.doc_idx]
    }

    fn canvas_init(&mut self) -> Result<()> {
        self.render_screen()?;
        Ok(())
    }

    pub fn edit(&mut self) -> Result<()> {
        self.canvas_init()?;
        while self.step()? == EditStep::Continue {
            // Keep spinning the event loop
        }
        Ok(())
    }

    fn step(&mut self) -> Result<EditStep> {
        let Some(event) = self.input.next().transpose()? else {
            return Ok(EditStep::Quit);
        };
        match event {
            Event::Resize { cols, rows } => {
                self.renderer.resize(Size {
                    width: cols,
                    height: rows,
                })?;
                self.renderer.set_redraw_idx(self.renderer.rowoff);
                self.status_bar.redraw = true;
                self.render_screen()?;
                Ok(EditStep::Continue)
            }
            Event::Key(seq) => {
                let step = self.process_keypress(seq)?;
                if step == EditStep::Continue {
                    self.render_screen()?;
                }
                Ok(step)
            }
        }
    }

    fn render_screen(&mut self) -> Result<()> {
        self.refresh_status_bar();
        let doc = &mut self.documents[self.doc_idx];
        self.renderer
            .render(&doc.buffer, &mut doc.highlight, &self.status_bar)?;
        self.status_bar.redraw = false;
        Ok(())
    }

    fn refresh_status_bar(&mut self) {
        self.status_bar.set_buf_pos(Position {
            curr: self.doc_idx + 1,
            size: self.documents.len(),
        });
        self.status_bar
            .update_from_buf(&self.documents[self.doc_idx].buffer);
    }

    fn process_keypress(&mut self, seq: KeySeq) -> Result<EditStep> {
        if seq.ctrl && seq.key == Key::Char('q') {
            return Ok(EditStep::Quit);
        }
        Ok(EditStep::Continue)
    }

    fn prompt<A>(&mut self, prompt_text: &str, cmd_empty: bool) -> Result<CommandResult>
    where
        A: Action,
    {
        let doc = &mut self.documents[self.doc_idx];
        let mut cmd = Command::new(
            &mut self.renderer,
            &mut doc.buffer,
            &mut doc.highlight,
            cmd_empty,
            &mut self.status_bar,
        );
        cmd.run::<A, _, _>(prompt_text, &mut self.input)
    }

    fn save(&mut self) -> Result<()> {
        if self.doc().buffer.filename() == "[No Name]" {
            let result = self.prompt::<NoAction>("Save as: ", true)?;
            match result {
                CommandResult::Canceled => {
                    self.renderer.set_info_msg("Save canceled");
                    return Ok(());
                }
                CommandResult::Input(name) => {
                    self.doc_mut().buffer.set_file(name);
                }
            }
        }
        let doc = self.doc_mut();
        match doc.buffer.save() {
            Ok(msg) => self.renderer.set_info_msg(msg),
            Err(err) => self
                .renderer
                .set_error_msg(format!("Failed to save: {}", err)),
        }
        Ok(())
    }

    fn search(&mut self) -> Result<()> {
        self.prompt::<TextSearch>("Search: ", true)?;
        Ok(())
    }

    fn show_help(&mut self) -> Result<()> {
        self.renderer.set_info_msg(HELP);
        Ok(())
    }
}
