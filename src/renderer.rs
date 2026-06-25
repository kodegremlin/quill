
use std::{io::Write, time::SystemTime};

use anyhow::{Context, Result, bail};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::Print,
    terminal::{Clear, ClearType},
};

use crate::terminal::Size;

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
        self.
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
    pub fn fold(self, rhs: Self) -> Self {
        todo!()
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
    msg: Option<StatusMessage>,

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
            msg: todo!(),
            draw_message: todo!(),
            redraw_idx: todo!(),
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
