use crate::{
    buffer::TextBuffer,
    renderer::Renderer,
    status_bar::StatusBar,
    terminal::{Terminal, TerminalGuard},
};

use anyhow::Result;
use std::{
    io::{self, Write},
    time::Duration,
};

mod buffer;
mod color;
mod diff;
mod highlight;
mod history;
mod lang;
mod renderer;
mod row;
mod status_bar;
mod terminal;

/* TODO 1. document the code and for simple functions just introduce what
the function does.
*/

fn main() -> Result<()> {
    let _guard = TerminalGuard::enter()?;
    let size = Terminal::size()?;
    let mut renderer = Renderer::new(size, io::stdout())?;
    renderer.set_info_msg("Test Message: The engine is working");
    let mut buf = TextBuffer::empty();
    buf.set_file("test.rs");
    let status = StatusBar::from_buffer(&buf, (1, 1));

    let mut stdout = io::stdout();

    renderer.draw_status_bar(&mut stdout, &status)?;
    renderer.draw_message_bar(&mut stdout, renderer.message.as_ref().unwrap())?;
    stdout.flush()?;
    std::thread::sleep(Duration::from_secs(10));
    Ok(())
}
