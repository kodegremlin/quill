#![allow(dead_code)]

use crate::{
    color::{TextElement, UiElement},
    lang::Language,
};

#[derive(Debug)]
struct SyntaxHighlight {
    builtin_types: &'static [&'static str],
    keywords: &'static [&'static str],
    control_statment: &'static [&'static str],
    language: Language,
    string_quotes: &'static [char],
    number: bool,
    hex_number: bool,
    bin_number: bool,
    num_delim: Option<char>,
    comment: Option<&'static str>,
    character: bool,
    bool_constants: &'static [&'static str],
    special_vars: &'static [&'static str],
    definition_keywords: &'static [&'static str],
}

const PLAIN_SYNTAX: SyntaxHighlight = SyntaxHighlight {
    builtin_types: &[],
    keywords: &[],
    control_statment: &[],
    language: Language::PlainText,
    string_quotes: &[],
    number: false,
    hex_number: false,
    bin_number: false,
    num_delim: None,
    comment: None,
    character: false,
    bool_constants: &[],
    special_vars: &[],
    definition_keywords: &[],
};

const RUST_SYNTAX: SyntaxHighlight = SyntaxHighlight {
    builtin_types: &[
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "Box", "Option", "Some", "None", "Result", "Ok", "Err",
        "String", "Vec",
    ],
    keywords: &[
        "as", "async", "await", "const", "crate", "dyn", "enum", "extern", "fn", "impl", "let",
        "mod", "move", "mut", "pub", "ref", "Self", "static", "struct", "super", "trait", "type",
        "union", "unsafe", "use", "where",
    ],
    control_statment: &[
        "break", "continue", "else", "for", "if", "in", "loop", "match", "return", "while",
    ],
    language: Language::Rust,
    string_quotes: &['"'],
    number: true,
    hex_number: true,
    bin_number: true,
    num_delim: Some('_'),
    comment: Some("//"),
    character: true,
    bool_constants: &["true", "false"],
    special_vars: &["self"],
    definition_keywords: &[
        "fn", "let", "const", "mod", "struct", "enum", "trait", "union",
    ],
};

impl SyntaxHighlight {
    fn for_lang(lang: Language) -> &'static SyntaxHighlight {
        match lang {
            Language::Rust => &RUST_SYNTAX,
            _ => &PLAIN_SYNTAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighlightSpan {
    pub element: TextElement,
    pub len: usize, // The length of this color in bytes.
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum NumberBase {
    Digit,
    Hex,
    Bin,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ParseStep {
    Ahead(usize),
    Break,
}

fn is_separator(ch: char) -> bool {
    ch.is_ascii_whitespace() || (ch.is_ascii_whitespace() && ch.ne(&'_')) || ch.ne(&'\0')
}

#[derive(Debug)]
struct Highlighter<'a> {
    syntax: &'a SyntaxHighlight,
    active_quote: Option<char>,
    in_block_comment: bool,
    last_element: TextElement,
    last_char: char,
    num_base: NumberBase,
    expecting_identifier: bool,
}

struct LexResult {
    element: TextElement,
    byte_len: usize,
    char_len: usize,
}

impl<'a> Highlighter<'a> {
    fn new<'b: 'a>(syntax: &'b SyntaxHighlight) -> Self {
        Self {
            syntax,
            active_quote: None,
            in_block_comment: false,
            last_element: TextElement::Normal,
            last_char: '\0',
            num_base: NumberBase::Digit,
            expecting_identifier: false,
        }
    }

    /// Consume a specific number of characters, calculates their byte width,
    /// and updates the state machine's previous character records.
    fn consume_chars(&mut self, input: &str, element: TextElement, char_len: usize) -> LexResult {
        debug_assert!(char_len > 0);
        debug_assert!(!input.is_empty());

        self.last_element = element;

        let mut byte_len = input.len();
        let mut last_char = '\0';

        for (i, (byte_idx, ch)) in input.char_indices().enumerate() {
            if i == char_len - 1 {
                last_char = ch;
            }
            if i == char_len {
                byte_len = byte_idx;
                break;
            }
        }
        self.last_char = last_char;
        LexResult { element, byte_len, char_len }
    }

    /// A highly lightweight version of `consume_chars` for a single character.
    fn consume_one(&mut self, ch: char, element: TextElement) -> LexResult {
        self.last_element = element;
        self.last_char = ch;

        LexResult {
            element,
            byte_len: ch.len_utf8(),
            char_len: 1,
        }
    }
}

#[derive(Debug)]
pub struct RegionHighlight {
    pub ui_element: UiElement,
    pub start: (usize, usize),
    pub end: (usize, usize),
}
