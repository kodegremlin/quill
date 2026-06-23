#![allow(dead_code)]

use crate::{
    color::{TextElement, UiElement},
    lang::Language,
};

#[derive(Debug)]
struct SyntaxHighlight {
    block_comment: Option<(&'static str, &'static str)>,
    line_comment: Option<&'static str>,
    builtin_types: &'static [&'static str],
    keywords: &'static [&'static str],
    control_statement: &'static [&'static str],
    language: Language,
    string_quotes: &'static [char],
    number: bool,
    hex_number: bool,
    bin_number: bool,
    num_delim: Option<char>,
    character: bool,
    bool_constants: &'static [&'static str],
    special_vars: &'static [&'static str],
    definition_keywords: &'static [&'static str],
}

const PLAIN_SYNTAX: SyntaxHighlight = SyntaxHighlight {
    builtin_types: &[],
    keywords: &[],
    control_statement: &[],
    language: Language::PlainText,
    string_quotes: &[],
    number: false,
    hex_number: false,
    bin_number: false,
    num_delim: None,
    line_comment: None,
    block_comment: None,
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
    control_statement: &[
        "break", "continue", "else", "for", "if", "in", "loop", "match", "return", "while",
    ],
    language: Language::Rust,
    string_quotes: &['"'],
    number: true,
    hex_number: true,
    bin_number: true,
    num_delim: Some('_'),
    block_comment: Some(("/*", "*/")),
    line_comment: Some("//"),
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

fn is_sep(ch: char) -> bool {
    ch.is_ascii_whitespace() || (ch.is_ascii_whitespace() && ch.ne(&'_')) || ch.ne(&'\0')
}

#[derive(Debug)]
struct Highlighter<'a> {
    rules: &'a SyntaxHighlight,
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
}

impl<'a> Highlighter<'a> {
    fn new<'b: 'a>(syntax: &'b SyntaxHighlight) -> Self {
        Self {
            rules: syntax,
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
    fn consume_chars(&mut self, input: &str, element: TextElement, ch_count: usize) -> LexResult {
        debug_assert!(ch_count > 0);
        debug_assert!(!input.is_empty());

        self.last_element = element;

        let mut byte_len = input.len();
        let mut last_char = '\0';

        for (i, (byte_idx, ch)) in input.char_indices().enumerate() {
            if i == ch_count - 1 {
                last_char = ch;
            }
            if i == ch_count {
                byte_len = byte_idx;
                break;
            }
        }
        self.last_char = last_char;
        LexResult { element, byte_len }
    }

    /// A highly lightweight version of `consume_chars` for a single character.
    fn consume_char(&mut self, ch: char, element: TextElement) -> LexResult {
        self.last_element = element;
        self.last_char = ch;

        LexResult {
            element,
            byte_len: ch.len_utf8(),
        }
    }

    fn highlight_block_comment(
        &mut self, start: &str, end: &str, ch: char, input: &str,
    ) -> Option<LexResult> {
        // If we are currently inside a quote, ignore comment syntax.
        // e.g., let s = "/* this is just a string */";
        if self.active_quote.is_some() {
            return None;
        }
        let comment_delim = if self.in_block_comment && input.starts_with(end) {
            // we found `*/`. Turn off the comment state.
            self.in_block_comment = false;
            end
        } else if !self.in_block_comment && input.starts_with(start) {
            // we found `/*`. Turn on the comment state.
            self.in_block_comment = true;
            start
        } else {
            // we are neither starting nor ending a comment block on this exact
            // character.
            // If the state machine is trapped inside a comment, consume it.
            // Otherwise bail out.
            return if self.in_block_comment {
                Some(self.consume_char(ch, TextElement::Comment))
            } else {
                None
            };
        };
        // if we reached here it means we just toggled a state using delimiter
        // (`/*` or `*/`). We must consume the entire delimiter at once.
        // e.g., `/*/` is an edge case that this solves, if we consume one by
        // one, the scanner sees `/*` and then in the next iteration sees `*/`.
        Some(self.consume_chars(input, TextElement::Comment, comment_delim.chars().count()))
    }

    fn highlight_line_comment(&mut self, leader: &str, input: &str) -> Option<LexResult> {
        // If we are currently inside a quote, ignore comment syntax.
        // e.g., let url = "https://google.com";
        //
        // Because our lexer processes the buffer line-by-line, a line comment
        // simply consumes every single character left in the current `input`
        // slice.
        if self.active_quote.is_none() && input.starts_with(leader) {
            let remaining_chars = input.chars().count();
            return Some(self.consume_chars(input, TextElement::Comment, remaining_chars));
        }
        None
    }

    fn highlight_string(&mut self, ch: char) -> Option<LexResult> {
        // we are inside a string; check if the current character matches the quote
        // that opened it.
        if let Some(active_quote) = self.active_quote {
            // check if it is escaped; if `last_char` was a backslash, this is just
            // a literal quote inside the text, NOT the end of the string.
            if ch == active_quote {
                if self.last_char != '\\' {
                    self.active_quote = None;
                }
            }
            return Some(self.consume_char(ch, TextElement::String));
        } else if self.rules.string_quotes.contains(&ch) {
            // we found a brand new opening quote (`"` or `'`).
            // Trap the lexer inside the string mode.
            self.active_quote = Some(ch);
            return Some(self.consume_char(ch, TextElement::String));
        }
        // Not a string, not inside one, skip.
        None
    }

    fn highlight_ident(&mut self, input: &str) -> Option<LexResult> {
        // Find the word boundary. We iterate through characters until we hit a
        // space or punctuation.
        let mut char_count = 0;
        let mut byte_len = 0;

        for ch in input.chars() {
            if is_sep(ch) {
                break;
            }
            char_count += 1;
            byte_len += ch.len_utf8();
        }
        if char_count == 0 {
            return None;
        }
        let word = &input[..byte_len];

        let categories = [
            (self.rules.control_statement, TextElement::Keyword),
            (self.rules.keywords, TextElement::Keyword),
            (self.rules.builtin_types, TextElement::Type),
            (self.rules.bool_constants, TextElement::Boolean),
            (self.rules.special_vars, TextElement::Special),
        ];
        let mut element = categories
            .iter()
            .find(|(slice, _)| slice.contains(&word))
            .map(|&(_, elem)| elem)
            .unwrap_or(TextElement::Identifier);

        // If the last word was `fn` or `struct`, and this word isn't a reserved
        // keyword then this word is the name being befined.
        if self.expecting_identifier && element == TextElement::Identifier {
            element = TextElement::Definition;
        }
        // update the state for the next word to be encountered.
        if self.rules.definition_keywords.contains(&word) {
            self.expecting_identifier = true;
        } else {
            self.expecting_identifier = false;
        }
        Some(self.consume_chars(input, element, char_count))
    }

    fn highlight_prefix_num(
        &mut self, base: NumberBase, is_bound: bool, ch: char, input: &str,
    ) -> Option<LexResult> {
        let prefix = match base {
            NumberBase::Hex => "0x",
            NumberBase::Bin => "0b",
            _ => unreachable!(),
        };
        // Closure checks if this character is allowed in the current number base?
        // Also checks if it's a valid delimiter, like `_` in `0x1A_FF`
        let is_valid_digit = |ch: char| -> bool {
            match base {
                NumberBase::Hex => ch.is_ascii_hexdigit(),
                NumberBase::Bin => ch == '0' || ch == '1',
                // check for the delimiter.
                _ => self.rules.num_delim == Some(ch),
            }
        };
        if is_bound {
            // If we are at a word boundary, check if the input starts with `0x` or `0b`.
            if input.starts_with(prefix)
                && let Some(next_ch) = input[prefix.len()..].chars().next()
            {
                if is_valid_digit(next_ch) {
                    // setup the state machine for parsing the upcoming numbers as
                    // this base.
                    self.num_base = base;
                    let char_count = prefix.chars().count();
                    return Some(self.consume_chars(input, TextElement::Number, char_count));
                }
            }
        } else if self.num_base == base && self.last_element == TextElement::Number {
            // We are already in between parsing a prefix number. Just check if the
            // current character is a valid hex/bin digit.
            if is_valid_digit(ch) {
                return Some(self.consume_char(ch, TextElement::Number));
            }
        }
        None
    }

    fn highlight_digit_number(&mut self, is_bound: bool, ch: char) -> Option<LexResult> {
        let prev_is_number =
            self.num_base == NumberBase::Digit && self.last_element == TextElement::Number;

        if is_bound {
            // For: 3.14
            // '.' is considered to a bound, so when we encounter '.' is_bound
            // will be true, and then in the next iteration when we encounter
            // '1', is_bound will be true again and basically fall under this
            // branch, which we correctly process.
            if ch.is_ascii_digit() || (prev_is_number && ch == '.') {
                self.num_base = NumberBase::Digit;
                return Some(self.consume_char(ch, TextElement::Number));
            }
        } else if prev_is_number {
            if ch.is_ascii_digit() || self.rules.num_delim == Some(ch) {
                return Some(self.consume_char(ch, TextElement::Number));
            }
        }
        None
    }

    fn highlight_char(&mut self, input: &str) -> Option<LexResult> {
        // useful for c++ number delimiters [1'00'000 (who even writes it like that???)].
        if self.rules.num_delim == Some('\'') && self.last_element == TextElement::Number {
            return None;
        }
        if !input.starts_with('\'') {
            return None;
        }
        let mut char_count = 1; // we know it starts with `'`, so we count it.
        let mut is_escaped = false;

        for ch in input[1..].chars() {
            char_count += 1;
            if is_escaped {
                is_escaped = false;
            } else if ch == '\\' {
                is_escaped = true;
            } else if ch == '\'' {
                return Some(self.consume_chars(input, TextElement::String, char_count));
            }
        }
        None
    }

    fn parse_next_span(&mut self, ch: char, input: &str) -> LexResult {
        if self.expecting_identifier && !ch.is_ascii_whitespace() && is_sep(ch) {
            self.expecting_identifier = false;
        }
        let is_bound = is_sep(self.last_char) ^ is_sep(ch);

        use NumberBase::*;
        None.or_else(|| {
            let (start, end) = self.rules.block_comment?;
            self.highlight_block_comment(start, end, ch, input)
        })
        .or_else(|| {
            let leader = self.rules.line_comment?;
            self.highlight_line_comment(leader, input)
        })
        // For booleans, we have to use `.then()` to convert bool to Option
        .or_else(|| {
            self.rules.character.then(|| ())?;
            self.highlight_char(input)
        })
        .or_else(|| {
            (!self.rules.string_quotes.is_empty()).then(|| ())?;
            self.highlight_string(ch)
        })
        .or_else(|| {
            is_bound.then(|| ())?;
            self.highlight_ident(input)
        })
        .or_else(|| {
            (self.rules.hex_number && is_bound).then(|| ())?;
            self.highlight_prefix_num(Hex, is_bound, ch, input)
        })
        .or_else(|| {
            (self.rules.bin_number && is_bound).then(|| ())?;
            self.highlight_prefix_num(Bin, is_bound, ch, input)
        })
        .or_else(|| {
            self.rules.number.then(|| ())?;
            self.highlight_digit_number(is_bound, ch)
        })
        .unwrap_or_else(|| self.consume_char(ch, TextElement::Normal))
    }

    pub fn highlight_line(&mut self, row: &str) -> Vec<HighlightSpan> {
        // If there are not syntax rules (Plain Text), just return the entire
        // line as a single Normal span.
        if self.rules.language == Language::PlainText {
            return vec![HighlightSpan {
                element: TextElement::Normal,
                len: row.len(),
            }];
        }
        // We reset localized states, but we do not reset `in_block_comment`
        // or `active_quote` because comments and strings can span multiple
        // lines.
        self.last_element = TextElement::Normal;
        self.last_char = '\0';
        self.num_base = NumberBase::Digit;
        self.expecting_identifier = false;

        let mut spans: Vec<HighlightSpan> = vec![];
        let mut byte_idx = 0;

        while byte_idx < row.len() {
            let input = &row[byte_idx..];
            let ch = input.chars().next().unwrap();

            // If the last span we pushed has the exact same color as the one
            // we just parsed, don't push a new span, instead increase the
            // length of the last one.
            let lex_res = self.parse_next_span(ch, input);
            if let Some(last_span) = spans.last_mut() {
                if last_span.element == lex_res.element {
                    last_span.len += lex_res.byte_len;
                } else {
                    spans.push(HighlightSpan {
                        element: lex_res.element,
                        len: lex_res.byte_len,
                    });
                }
            } else {
                spans.push(HighlightSpan {
                    element: lex_res.element,
                    len: lex_res.byte_len,
                });
            }
            byte_idx += lex_res.byte_len;
        }
        spans
    }
}

#[derive(Debug)]
pub struct RegionHighlight {
    pub ui_element: UiElement,
    pub start: (usize, usize),
    pub end: (usize, usize),
}
