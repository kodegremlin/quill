use crate::{
    buffer::TextBuffer,
    highlight::Highlighting,
    renderer::Renderer,
    status_bar::StatusBar,
    terminal::{Terminal, TerminalGuard},
};

use anyhow::Result;
use clap::Parser;
use std::{
    io::{self},
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let _guard = TerminalGuard::enter()?;
    let size = Terminal::size()?;

    let mut renderer = Renderer::new(size, io::stdout())?;

    print!("here?");

    let buffer = if let Some(path) = args.file {
        TextBuffer::open(&path)?
    } else {
        TextBuffer::empty()
    };
    let status = StatusBar::from_buffer(&buffer, (buffer.col_idx(), buffer.row_idx()));
    let mut hl = Highlighting::default();
    hl.lang_changed(lang::Language::Rust);

    renderer.set_info_msg("Text rendering complete, scrolling is left to be added.");
    renderer.render(&buffer, &mut hl, &status)?;

    std::thread::sleep(Duration::from_secs(20));
    Ok(())
}
