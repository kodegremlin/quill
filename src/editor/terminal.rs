use std::{
    fmt::Display,
    io::{self, Write},
};

use crossterm::{
    Command, cursor, queue, style,
    terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

/// The size of the terminal.
#[derive(Clone, Copy)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

/// The position in the terminal.
#[derive(Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// Moves the cursor to the given `Position`.
/// # Arguments
/// * `Position` - the `Position` to move the cursor to. Will be truncated to `u16::MAX`
///    if bigger.
pub fn move_cursor_to(pos: Position) -> Result<(), io::Error> {
    queue_command(cursor::MoveTo(pos.x as u16, pos.y as u16))?;
    Ok(())
}

pub fn execute() -> Result<(), io::Error> {
    io::stdout().flush()?;
    Ok(())
}

pub fn initialize() -> Result<(), io::Error> {
    enable_raw_mode()?;
    clear_screen()?;
    move_cursor_to(Position { x: 0, y: 0 })?;
    execute()?;
    Ok(())
}

pub fn terminate() -> Result<(), io::Error> {
    disable_raw_mode()?;
    Ok(())
}

/// Returns `Size` which represents the number of rows and columns.
pub fn size() -> Result<Size, io::Error> {
    let (width, height) = terminal::size()?;
    Ok(Size {
        height: height as usize,
        width: width as usize,
    })
}

pub fn print<T: Display>(strs: T) -> Result<(), io::Error> {
    queue_command(style::Print(strs))?;
    Ok(())
}

fn queue_command<T: Command>(command: T) -> Result<(), io::Error> {
    queue!(io::stdout(), command)?;
    Ok(())
}

pub fn show_cursor() -> Result<(), io::Error> {
    queue_command(cursor::Show)?;
    Ok(())
}

pub fn hide_cursor() -> Result<(), io::Error> {
    queue_command(cursor::Hide)?;
    Ok(())
}

pub fn clear_line() -> Result<(), io::Error> {
    queue_command(Clear(ClearType::CurrentLine))?;
    Ok(())
}

pub fn clear_screen() -> Result<(), io::Error> {
    queue_command(Clear(ClearType::All))?;
    Ok(())
}
