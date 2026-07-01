#![allow(dead_code)]

use crate::{
    buffer::TextBuffer,
    highlight::Highlighting,
    renderer::Renderer,
    status_bar::{Position, StatusBar},
    terminal::{Terminal, TerminalGuard},
};

use anyhow::Result;
use clap::Parser;
use env_logger::Target;
use std::{
    fs::{self, File, OpenOptions},
    io::{self},
    time::Duration,
};

mod buffer;
mod buffer_tests;
mod color;
mod command;
mod diff;
mod editor;
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

fn setup_log_path() -> Option<File> {
    let log_path = dirs::home_dir()?
        .join(".quill")
        .join("logs")
        .join("quill.log");

    if let Err(err) = fs::create_dir_all(log_path.parent()?) {
        eprintln!("quill: could not create log directory: {}", err);
        return None;
    }

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(file) => {
            eprintln!("quill: logging to {}", log_path.display());
            Some(file)
        }
        Err(err) => {
            eprintln!("quill: could not open log file: {}", err);
            None
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(log_file) = setup_log_path() {
        env_logger::Builder::new()
            .target(Target::Pipe(Box::new(log_file)))
            .filter_level(log::LevelFilter::Debug)
            .init();
    }
    let _guard = TerminalGuard::enter()?;
    let size = Terminal::size()?;

    let mut renderer = Renderer::new(size, io::stdout())?;

    let buffer = if let Some(path) = args.file {
        TextBuffer::open(&path)?
    } else {
        TextBuffer::empty()
    };
    log::info!(
        target: "buffer",
        "CursorInfo:: col={}, row={} FileLen:: len={}",
        buffer.col_idx(), buffer.row_idx(), buffer.rows().len()
    );
    let status = StatusBar::from_buffer(
        &buffer,
        Position {
            curr: buffer.col_idx(),
            size: buffer.row_idx(),
        },
    );
    let mut hl = Highlighting::default();
    hl.lang_changed(lang::Language::Rust);

    renderer.set_info_msg("Text rendering complete, scrolling is left to be added.");
    renderer.render(&buffer, &mut hl, &status)?;

    std::thread::sleep(Duration::from_secs(20));
    Ok(())
}
