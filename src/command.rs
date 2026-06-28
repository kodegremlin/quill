use std::io::Write;

use anyhow::Result;

use crate::{
    buffer::TextBuffer,
    highlight::Highlighting,
    renderer::Renderer,
    status_bar::StatusBar,
    terminal::{Event, KeySeq, Size},
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
        _seq: KeySeq,
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
    pub cmd_empty: bool,
    pub status_bar: &'a mut StatusBar,
}

impl<'a, W: Write> Command<'a, W> {
    pub fn new(
        renderer: &'a mut Renderer<W>,
        buffer: &'a mut TextBuffer,
        highlight: &'a mut Highlighting,
        cmd_empty: bool,
        status_bar: &'a mut StatusBar,
    ) -> Self {
        Self {
            renderer,
            buffer,
            highlight,
            cmd_empty,
            status_bar,
        }
    }

    fn render_screen(&mut self, input: &str, cmd_template: &CommandTemplate) -> Result<()> {
        self.renderer.set_info_msg(cmd_template.build(input));
        self.status_bar.update_from_buf(self.buffer);
        self.renderer
            .render(self.buffer, self.highlight, self.status_bar)?;

        let prompt_row = self.renderer.rows() + 1;
        let prompt_col = cmd_template.cursor_col(input);
        self.renderer.force_set_cursor(prompt_col, prompt_row)?;
        Ok(())
    }

    pub fn run<A, S, I>(&mut self, cmd: S, input: &mut I) -> Result<CommandResult>
    where
        A: Action,
        S: AsRef<str>,
        I: Iterator<Item = Result<Event>>,
    {
        let mut action = A::new(self);
        let mut cmd_buf = String::new();
        let mut canceled = false;

        // Parse the command template (e.g. "Save as: {} (ESC to cancel)").
        let cmd_template = {
            let mut parts = cmd.as_ref().splitn(2, "{}");
            let prefix = parts.next().unwrap_or("");
            let suffix = parts.next().unwrap_or("");
            CommandTemplate::new(prefix, suffix)
        };
        // Initial render before waiting for any input.
        self.render_screen("", &cmd_template)?;

        while let Some(event) = input.next().transpose()? {
            let prev_len = cmd_buf.len();

            match event {
                Event::Resize { cols, rows } => {
                    self.renderer.resize(Size {
                        width: cols,
                        height: rows,
                    })?;
                    self.renderer.set_redraw_idx(self.renderer.rowoff);
                    self.status_bar.redraw = true;
                    self.render_screen(&cmd_buf, &cmd_template)?;
                    continue;
                }
                Event::Key(seq) => {
                    use crate::terminal::Key::*;
                    match (seq.key, seq.ctrl) {
                        (Backspace, _) | (Delete, _) | (Char('h'), true) => {
                            if !cmd_buf.is_empty() {
                                cmd_buf.pop();
                            }
                        }
                        (Char('w'), true) => {
                            let is_sep =
                                |ch: char| ch.is_ascii_whitespace() || ch.is_ascii_punctuation();

                            while let Some(curr) = cmd_buf.pop() {
                                let next_is_sep = cmd_buf.chars().last().map_or(true, is_sep);
                                if !is_sep(curr) && next_is_sep {
                                    break;
                                }
                            }
                        }
                        (Char('j'), true) => cmd_buf.clear(),
                        (Esc, _) | (Char('q'), true) | (Char('g'), true) => {
                            canceled = true;
                            break;
                        }
                        (Unknown, _) => continue,
                        (Enter, _) | (Char('m'), true) => break,
                        (Char(ch), false) => cmd_buf.push(ch),
                        _ => {}
                    }
                    let should_render = action.on_seq(self, &cmd_buf, seq)?;

                    if should_render || prev_len != cmd_buf.len() {
                        self.render_screen(&cmd_buf, &cmd_template)?;
                    }
                }
            }
        }
        let result = if canceled || (self.cmd_empty && cmd_buf.is_empty()) {
            self.renderer.set_info_msg("Canceled");
            CommandResult::Canceled
        } else {
            self.renderer.remove_msg();
            self.status_bar.redraw = true;
            CommandResult::Input(cmd_buf)
        };
        action.on_end(self, result)
    }
}
