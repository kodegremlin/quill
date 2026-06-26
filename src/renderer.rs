#![allow(dead_code)]

use std::{io::Write, time::SystemTime};

use anyhow::{Context, Result, bail};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor},
    terminal::{Clear, ClearType},
};
use unicode_width::UnicodeWidthChar;

use crate::{status_bar::StatusBar, terminal::Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusMessageKind {
    Info,
    Error,
}

#[derive(Debug)]
pub struct StatusMessage {
    pub text: String,
    pub timestamp: SystemTime,
    pub kind: StatusMessageKind,
}

impl StatusMessage {
    pub fn new<S: Into<String>>(msg: S, kind: StatusMessageKind) -> Self {
        Self {
            text: msg.into(),
            timestamp: SystemTime::now(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMessage {
    Open,
    Close,
    Update,
    DoNothing,
}

impl DrawMessage {
    pub fn merge(self, rhs: Self) -> Self {
        use DrawMessage::*;
        match (self, rhs) {
            (lhs, DoNothing) => lhs,
            (DoNothing, rhs) => rhs,
            (Update, Update) => Update,
            (Open, Update) => Open,
            (Update, Close) => Close,
            (Close, Open) => Update,
            (Open, Close) => DoNothing,
            (lhs, rhs) => {
                unreachable!("invalid DrawMessage transition: {:?} then {:?}", lhs, rhs)
            }
        }
    }
}

fn window_too_small(width: usize, height: usize) -> bool {
    width < 1 || height < 3
}

pub struct Renderer<W: Write> {
    /// The usable grid height in physical rows (height - 2).
    num_rows: usize,

    /// The usable grid width in physical columns.
    num_cols: usize,

    /// The underlying unbuffered output sink.
    output: W,

    /// The expanded x co-ordinate - what's rendered on screen.
    row_idx: usize,

    /// The active transient status message, if any.
    pub message: Option<StatusMessage>,

    /// The current operational state of the message bar.
    draw_message: DrawMessage,

    /// The boundary from where the text is assumed to invalid.
    /// All rows from here to num_rows will be unconditionally
    /// repainted on the next frame.
    redraw_idx: Option<usize>,

    /// True if the cursor position moved during the frame tick.
    pub cursor_moved: bool,

    pub rowoff: usize,
    pub coloff: usize,
}

impl<W: Write> Renderer<W> {
    pub fn new(size: Size, mut output: W) -> Result<Self> {
        let height = size.height as usize;
        let width = size.width as usize;

        if window_too_small(width, height) {
            bail!(
                "Terminal window too small: {}x{} (minimum required is 1x3",
                width,
                height
            )
        }
        // Hide the cursor immediately so that the cursor doesn't jump around
        // the screen chaotically.
        queue!(output, Hide)?;
        output
            .flush()
            .context("Failed to flush intial cursor hide sequence")?;

        Ok(Self {
            // The text grid is strictly 2 rows shorter than the window; to reserve
            // permanent space for the Status Bar (row N-1) and Message Bar (row N)
            num_rows: height.saturating_sub(2),
            num_cols: width,
            output,
            row_idx: 0,
            message: None,
            draw_message: DrawMessage::DoNothing,
            redraw_idx: Some(0),
            cursor_moved: true,
            rowoff: 0,
            coloff: 0,
        })
    }

    pub fn resize(&mut self, size: Size) -> Result<()> {
        let height = size.height as usize;
        let width = size.width as usize;

        if window_too_small(width, height) {
            bail!(
                "Terminal resized below minimum viable bounds: {}x{}",
                width,
                height
            )
        }
        self.num_rows = height.saturating_sub(2);
        self.num_cols = width;
        Ok(())
    }

    fn write_sync(&mut self, bytes: &[u8]) -> Result<()> {
        self.output.write_all(bytes)?;
        self.output.flush()?;
        Ok(())
    }

    pub fn render_smoke_test(&mut self) -> Result<()> {
        let mut screen_buf = Vec::with_capacity((self.num_rows + 2) * self.num_cols);

        queue!(
            screen_buf,
            Hide,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print("Kiro Viewport Architecture: Active"),
            MoveTo(0, 1),
            Print(format!(
                "Usable text grid: {} cols x {} rows",
                self.num_cols, self.num_rows
            )),
            MoveTo(0, 2),
            Print("Hardware buffer safely bound. Ready for Layer 2..."),
            Show
        )?;
        self.write_sync(&screen_buf)
    }
}

impl<W: Write> Renderer<W> {
    fn set_msg(&mut self, status_msg: Option<StatusMessage>) {
        let rhs = match (&self.message, &status_msg) {
            (Some(prev), Some(next)) if prev.text == next.text => DrawMessage::DoNothing,
            (Some(_), Some(_)) => DrawMessage::Update,
            (None, Some(_)) => DrawMessage::Open,
            (Some(_), None) => DrawMessage::Close,
            (None, None) => DrawMessage::DoNothing,
        };
        self.draw_message = self.draw_message.merge(rhs);
        self.message = status_msg;
    }

    pub fn set_info_msg<S: Into<String>>(&mut self, msg: S) {
        use StatusMessageKind::*;
        self.set_msg(Some(StatusMessage::new(msg, Info)));
    }

    pub fn set_error_msg<S: Into<String>>(&mut self, msg: S) {
        use StatusMessageKind::*;
        self.set_msg(Some(StatusMessage::new(msg, Error)));
    }

    pub fn remove_msg(&mut self) {
        self.set_msg(None);
    }

    pub fn set_redraw_idx(&mut self, index: usize) {
        if let Some(ridx) = self.redraw_idx
            && ridx < index
        {
            return;
        }
        self.redraw_idx = Some(index);
    }

    pub fn rows(&self) -> usize {
        if self.message.is_some() {
            self.num_rows
        } else {
            self.num_rows + 1
        }
    }

    pub fn cols(&self) -> usize {
        self.num_cols
    }

    pub fn message_text(&self) -> &str {
        self.message.as_ref().map(|m| m.text.as_str()).unwrap_or("")
    }
}

impl<W: Write> Renderer<W> {
    fn update_message_bar(&mut self) -> Result<()> {
        if let Some(msg) = &self.message
            && let Ok(elapsed) = msg.timestamp.elapsed()
            && elapsed.as_secs() > 5
        {
            self.remove_msg();
        }
        if self.draw_message == DrawMessage::Close {
            // Closing the message bar reveals one more line of text at the bottom
            // of the grid.
            self.set_redraw_idx(self.num_rows);
        }
        Ok(())
    }

    fn trim_visual(text: &str, max_width: usize) -> (&str, usize) {
        let mut curr_width = 0;
        for (byte_idx, ch) in text.char_indices() {
            let ch_width = ch.width_cjk().unwrap_or(1);
            if curr_width + ch_width > max_width {
                return (&text[..byte_idx], curr_width);
            }
            curr_width += ch_width;
        }
        (text, curr_width)
    }

    pub fn draw_message_bar<B: Write>(&self, mut buf: B, msg: &StatusMessage) -> Result<()> {
        let (trimmed, _) = Self::trim_visual(&msg.text, self.num_cols);

        queue!(buf, MoveTo(0, self.num_rows as u16 + 1))?;
        if msg.kind == StatusMessageKind::Error {
            queue!(buf, SetBackgroundColor(Color::Red))?;
        }
        queue!(
            buf,
            Print(trimmed),
            ResetColor,
            Clear(ClearType::UntilNewLine)
        )?;
        Ok(())
    }

    pub fn draw_status_bar<B: Write>(&self, buf: &mut B, status_bar: &StatusBar) -> Result<()> {
        let right = status_bar.right();
        let left = status_bar.left();

        let (left_str, llen) = Self::trim_visual(&left, self.num_cols);

        queue!(
            buf,
            MoveTo(0, self.num_rows as u16),
            SetAttribute(Attribute::Reverse),
            Print(left_str)
        )?;
        let rem_cols = self.num_cols.saturating_sub(llen);

        let (right_str, rlen) = Self::trim_visual(&right, rem_cols);
        let padding = rem_cols.saturating_sub(rlen);

        if padding > 0 {
            queue!(buf, Print(" ".repeat(padding)))?;
        }
        queue!(
            buf,
            Print(right_str),
            SetAttribute(Attribute::Reset),
            Clear(ClearType::UntilNewLine),
        )?;
        Ok(())
    }
}
