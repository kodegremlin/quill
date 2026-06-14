#![allow(dead_code)]

use crossterm::style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextElement {
    Identifier,
    Keyword,
    String,
    Comment,
    Documentation,
    Number,
    Type,
    Definition,
    Boolean,
    Special,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiElement {
    Background,
    SearchMatches,
    CurrentMatch,
    Modes(Mode),
    StatusBar,
    StatusBarInactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeElement {
    Text(TextElement),
    Ui(UiElement),
}

pub struct Theme;

impl Theme {
    pub fn color_for(&self, color: ThemeElement) -> style::Color {
        match color {
            ThemeElement::Text(text) => color_for_text(text),
            ThemeElement::Ui(ui) => color_for_ui(ui),
        }
    }
}

fn color_for_text(element: TextElement) -> style::Color {
    use style::Color::*;
    match element {
        TextElement::Identifier => White,
        TextElement::Keyword => Red,
        TextElement::String => Blue,
        TextElement::Comment => todo!(),
        TextElement::Documentation => todo!(),
        TextElement::Number => todo!(),
        TextElement::Type => todo!(),
        TextElement::Definition => todo!(),
        TextElement::Boolean => todo!(),
        TextElement::Special => todo!(),
    }
}

fn color_for_ui(element: UiElement) -> style::Color {
    match element {
        UiElement::Background => todo!(),
        UiElement::SearchMatches => todo!(),
        UiElement::CurrentMatch => todo!(),
        UiElement::Modes(mode) => match mode {
            Mode::Normal => todo!(),
            Mode::Insert => todo!(),
            Mode::Visual => todo!(),
        },
        UiElement::StatusBar => todo!(),
        UiElement::StatusBarInactive => todo!(),
    }
}
