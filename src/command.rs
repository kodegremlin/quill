use std::io::Write;

use anyhow::Result;

use crate::{
    buffer::TextBuffer, highlight::Highlighting, renderer::Renderer, status_bar::StatusBar,
    terminal::KeyEvent,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    Canceled,
    Input(String),
}

pub trait Action: Sized {
    fn new<W: Write>(prompt: &mut Command<'_, W>) -> Self;

    fn on_seq<W: Write>(
        &mut self,
        _cmd: &mut Command<'_, W>,
        _input: &str,
        _seq: KeyEvent,
    ) -> Result<bool> {
        Ok(false)
    }

    fn on_end<W: Write>(
        self,
        _cmd: &mut Command<'_, W>,
        result: CommandResult,
    ) -> Result<CommandResult> {
        Ok(result)
    }
}

pub struct NoAction;

impl Action for NoAction {
    fn new<W: Write>(_cmd: &mut Command<'_, W>) -> Self {
        Self
    }
}

pub(crate) struct CommandTemplate<'a> {
    prefix: &'a str,
    suffix: &'a str,
    prefix_chars: usize,
}

impl<'a> CommandTemplate<'a> {
    pub fn new(prefix: &'a str, suffix: &'a str) -> Self {
        Self {
            prefix,
            suffix,
            prefix_chars: prefix.chars().count(),
        }
    }

    pub fn build(&self, input: &str) -> String {
        let capacity = self.prefix.len() + self.suffix.len() + input.len();
        let mut buf = String::with_capacity(capacity);
        buf.push_str(self.prefix);
        buf.push_str(input);
        buf.push_str(self.suffix);
        buf
    }

    pub fn cursor_col(&self, input: &str) -> usize {
        self.prefix_chars + input.chars().count()
    }
}

pub struct Command<'a, W: Write> {
    pub renderer: &'a mut Renderer<W>,
    pub buffer: &'a mut TextBuffer,
    pub highlight: &'a mut Highlighting,
    pub status_bar: &'a mut StatusBar,
    pub empty_is_cancel: bool,
}

impl<'a, W: Write> Command<'a, W> {
    pub fn new(
        renderer: &'a mut Renderer<W>,
        buffer: &'a mut TextBuffer,
        highlight: &'a mut Highlighting,
        status_bar: &'a mut StatusBar,
        empty_is_cancel: bool,
    ) -> Self {
        Self {
            renderer,
            buffer,
            highlight,
            status_bar,
            empty_is_cancel,
        }
    }

    fn render_screen(&mut self, input: &str, template: &CommandTemplate) -> Result<()> {
        self.renderer.set_info_msg(template.build(input));
        self.status_bar.update_from_buf(self.buffer);
        self.renderer
            .render(self.buffer, self.highlight, self.status_bar)?;

        let prompt_row = self.renderer.rows() + 1;
        let prompt_col = template.cursor_col(input);
        self.renderer.force_set_cursor(prompt_col, prompt_row)?;
        Ok(())
    }
}
