use crate::{buffer::TextBuffer, lang::Language};

macro_rules! setter {
    ($method:ident, $field:ident, $t:ty) => {
        pub fn $method(&mut self, $field: $t) {
            if self.$field != $field {
                self.redraw = true;
                self.$field = $field;
            }
        }
    };
    ($method:ident, $field:ident, $t:ty, $conv:expr) => {
        pub fn $method(&mut self, $field: $t) {
            if self.$field != $field {
                self.redraw = true;
                self.$field = $conv;
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub curr: usize,
    pub size: usize,
}

#[derive(Debug)]
pub struct StatusBar {
    pub filename: String,
    pub modified: bool, // Can show a ascii [+] icon to indicate modified like helix.
    pub lang: Language,
    pub doc_idx: Position,
    pub line_pos: Position,
    pub redraw: bool,
}

impl StatusBar {
    setter!(set_buf_pos, doc_idx, Position);
    setter!(set_modified, modified, bool);
    setter!(set_filename, filename, &str, filename.to_string());
    setter!(set_lang, lang, Language);
    setter!(set_line_pos, line_pos, Position);

    pub fn from_buffer(buffer: &TextBuffer, doc_idx: Position) -> Self {
        let line_pos = Position {
            curr: buffer.row_idx() + 1,
            size: buffer.rows().len(),
        };
        Self {
            modified: buffer.modified(),
            filename: buffer.filename().to_string(),
            lang: buffer.lang,
            doc_idx,
            line_pos,
            redraw: false,
        }
    }

    pub fn left(&self) -> String {
        format!(
            "{:<20?} - {}/{} {}",
            self.filename,
            self.doc_idx.curr,
            self.doc_idx.size,
            if self.modified { " (modified) " } else { "" }
        )
    }

    pub fn right(&self) -> String {
        format!(
            "{} {}/{}",
            self.lang.name(),
            self.line_pos.curr,
            self.line_pos.size,
        )
    }

    pub fn update_from_buf(&mut self, buf: &TextBuffer) {
        self.set_modified(buf.modified());
        self.set_lang(self.lang);
        self.set_filename(buf.filename());
        let line_pos = Position {
            curr: buf.row_idx() + 1,
            size: buf.rows().len(),
        };
        self.set_line_pos(line_pos);
    }
}
