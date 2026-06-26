use crate::{
    buffer::TextBuffer,
    renderer::{Renderer, StatusMessage},
    status_bar::StatusBar,
    terminal::{Terminal, TerminalGuard},
};

use anyhow::Result;
use std::{
    io::{self, Write},
    time::{Duration, SystemTime},
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
    renderer.render_welcome(&mut stdout)?;
    renderer.draw_status_bar(&mut stdout, &status)?;

    let msg_str = renderer.message_text();
    let status_msg = StatusMessage {
        text: msg_str.to_string(),
        timestamp: SystemTime::now(),
        kind: renderer::StatusMessageKind::Error,
    };
    renderer.draw_message_bar(&mut stdout, &status_msg)?;
    stdout.flush()?;
    std::thread::sleep(Duration::from_secs(10));
    Ok(())
}
