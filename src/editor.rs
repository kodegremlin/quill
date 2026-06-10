use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};

use std::io;

use crate::editor::terminal::{self as term, Position, Size};

mod terminal;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
}

impl Editor {
    /// Initializes the terminal and starts the repl with a clear screen. Clears the terminal
    /// before exiting.
    ///
    /// # Panic
    /// If there is an error in the repl it panics and clears the terminal before exiting.
    pub fn run(&mut self) {
        term::initialize().unwrap();
        let result = self.repl();
        term::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event);
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            // checks if code is 'q' first and THEN if the modifier is control,
            // otherwise skips.
            match code {
                KeyCode::Char('q') if modifiers == &KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => (),
            }
        }
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        term::hide_cursor()?;
        if self.should_quit {
            term::clear_screen()?;
            term::print("Goodbye.\r\n")?;
        } else {
            Self::draw_rows()?;
            term::move_cursor_to(Position { x: 0, y: 0 })?;
        }
        term::show_cursor()?;
        term::execute()?;
        Ok(())
    }

    fn draw_rows() -> Result<(), io::Error> {
        let Size { height, .. } = term::size()?;
        for current_row in 0..height {
            term::clear_line()?;
            if current_row == height / 3 {
                Self::draw_welcome_msg()?;
            } else {
                term::print("~")?;
            }
            if current_row.saturating_add(1) < height {
                term::print("\r\n")?;
            }
        }
        Ok(())
    }

    fn draw_welcome_msg() -> Result<(), io::Error> {
        let mut welcome_msg = format!("{NAME} editor -- version {VERSION}");
        let width = term::size()?.width;
        let len = welcome_msg.len();
        let padding = (width.saturating_sub(len)) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_msg = format!("~{spaces}{welcome_msg}");
        welcome_msg.truncate(width);
        term::print(welcome_msg)?;
        Ok(())
    }
}
