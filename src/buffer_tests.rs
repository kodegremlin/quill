use std::vec::IntoIter;

use anyhow::Result;

use crate::{
    buffer::{CursorDir, TextBuffer},
    terminal::{Event, Key, KeySeq},
};

pub struct DummyInputs(IntoIter<Event>);

impl DummyInputs {
    pub fn new(events: Vec<Event>) -> Self {
        Self(events.into_iter())
    }
}

impl Iterator for DummyInputs {
    type Item = Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Ok)
    }
}

pub fn key(ch: char) -> Event {
    Event::Key(KeySeq::new(Key::Char(ch)))
}

pub fn ctrl(ch: char) -> Event {
    Event::Key(KeySeq::ctrl(Key::Char(ch)))
}

pub fn alt(ch: char) -> Event {
    Event::Key(KeySeq::alt(Key::Char(ch)))
}

pub fn special(k: Key) -> Event {
    Event::Key(KeySeq::new(k))
}

pub fn enter() -> Event {
    Event::Key(KeySeq::new(Key::Enter))
}

pub fn backspace() -> Event {
    Event::Key(KeySeq::new(Key::Backspace))
}

pub fn dispatch_test_event(buffer: &mut TextBuffer, event: Event) {
    if let Event::Key(ke) = &event {
        println!("buffer={buffer:#?}\nevent={event:#?}");
        match (ke.key, ke.ctrl, ke.alt) {
            // -- Typing --
            (Key::Char(ch), false, false) => buffer.insert_char(ch),
            (Key::Enter, false, false) => buffer.insert_line(),

            // -- Deletion --
            (Key::Backspace, false, false) => buffer.delete_char(),
            (Key::Delete, false, false) => buffer.delete_right_char(),
            (Key::Char('k'), true, false) => buffer.delete_until_line_end(),
            (Key::Char('j'), true, false) => buffer.delete_until_line_head(),
            (Key::Char('w'), true, false) => buffer.delete_word(),

            // -- Movement --
            (Key::Left, false, false) => buffer.step(CursorDir::Left),
            (Key::Right, false, false) => buffer.step(CursorDir::Right),
            (Key::Up, false, false) => buffer.step(CursorDir::Up),
            (Key::Down, false, false) => buffer.step(CursorDir::Down),

            // -- Undo / Redo --
            (Key::Char('u'), true, false) => {
                buffer.undo();
            }
            (Key::Char('r'), true, false) => {
                buffer.redo();
            }

            // Unmapped keys in tests are simply ignored
            _ => {}
        }
    }
    buffer.commit_edit();
}

fn buffer_text(buffer: &TextBuffer) -> String {
    buffer
        .rows()
        .iter()
        .map(|r| r.buffer())
        .collect::<Vec<_>>()
        .join("\n")
}

fn setup_buffer(text: &str) -> TextBuffer {
    let mut buffer = TextBuffer::empty();
    if text.is_empty() {
        return buffer;
    }
    for ch in text.chars() {
        if ch == '\n' {
            buffer.insert_line();
        } else {
            buffer.insert_char(ch);
        }
    }
    buffer.commit_edit();

    // Snap cursor back to the beginning to mimic opening a file.
    while buffer.col_idx() > 0 || buffer.row_idx() > 0 {
        buffer.step(CursorDir::Left);
    }
    buffer
}

macro_rules! diff_tests {
    (
        $title:ident,
        $title_undo:ident,
        $title_redo:ident {
            before: $before:expr,
            input: [$( $input:expr ),+ $(,)?],
            after: $after:expr,
            cursor: $cursor:expr $(,)?
        }
    ) => {
        #[test]
        fn $title() {
            let mut buffer = setup_buffer($before);
            let inputs = vec![$( $input ),+];

            for event in DummyInputs::new(inputs) {
                dispatch_test_event(&mut buffer, event.unwrap());
            }
            assert_eq!(buffer_text(&buffer), $after, "Forward text mismatch");
            assert_eq!((buffer.col_idx(), buffer.row_idx()), $cursor, "Forward cursor mismatch");
        }

        #[test]
        fn $title_undo() {
            let mut buffer = setup_buffer($before);
            let inputs = vec![$( $input ),+];
            let num_inputs = inputs.len();

            for event in DummyInputs::new(inputs) {
                dispatch_test_event(&mut buffer, event.unwrap());
            }
            for _ in 0..num_inputs {
                dispatch_test_event(&mut buffer, ctrl('u'));
            }
            assert_eq!(buffer_text(&buffer), $before, "Undo text mismatch");
            assert_eq!((buffer.col_idx(), buffer.row_idx()), (0, 0), "Undo cursor mismatch");
        }

        #[test]
        fn $title_redo() {
            let mut buffer = setup_buffer($before);
            let inputs = vec![$( $input ),+];
            let num_inputs = inputs.len();

            for event in DummyInputs::new(inputs) {
                dispatch_test_event(&mut buffer, event.unwrap());
            }
            for _ in 0..num_inputs {
                dispatch_test_event(&mut buffer, ctrl('u'));
            }
            for _ in 0..num_inputs {
                dispatch_test_event(&mut buffer, ctrl('r'));
            }
            assert_eq!(buffer_text(&buffer), $after, "Redo text mismatch");
            assert_eq!((buffer.col_idx(), buffer.row_idx()), $cursor, "Redo cursor mismatch");
        }
    };
}

diff_tests!(
    insert_char,
    insert_char_undo,
    insert_char_redo {
        before: "",
        input: [
            key('a'),
            key('b'),
            special(Key::Down), // should do nothing, can't go down on a single line.
            key('c'),
            enter(),
            key('d'),
            key('e'),
        ],
        after: "abc\nde",
        cursor: (2, 1),
    }
);
