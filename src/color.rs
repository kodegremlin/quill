#![allow(dead_code)]

use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextElement {
    Identifier,
    Keyword,
    String,
    Comment,
    Documentation,
    Number,
    Type,
    Normal,
    Definition,
    Boolean,
    Special,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiElement {
    Background,
    SearchMatches,
    CurrentMatch,
    ModeNormal,
    ModeInsert,
    ModeVisual,
    StatusBar,
    StatusBarInactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeElement {
    Text(TextElement),
    Ui(UiElement),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Option<Color>,
}

impl Style {
    pub const fn new(fg: Color) -> Self {
        Self { fg, bg: None }
    }

    pub const fn with_bg(fg: Color, bg: Color) -> Self {
        Self { fg, bg: Some(bg) }
    }
}

pub struct Theme;

impl Theme {
    pub fn color_for(element: ThemeElement) -> Style {
        use ThemeElement::*;
        match element {
            Text(text) => color_for_text(text),
            Ui(ui) => color_for_ui(ui),
        }
    }
}

fn color_for_text(element: TextElement) -> Style {
    use TextElement::*;
    match element {
        Identifier => Style::new(Color::White),
        String => Style::new(Color::Blue),
        Keyword => Style::new(Color::Red),
        Comment => todo!(),
        Documentation => todo!(),
        Number => todo!(),
        Type => todo!(),
        Normal => todo!(),
        Definition => todo!(),
        Boolean => todo!(),
        Special => todo!(),
    }
}

fn color_for_ui(element: UiElement) -> Style {
    use UiElement::*;
    match element {
        Background => todo!(),
        ModeNormal => todo!(),
        ModeInsert => todo!(),
        ModeVisual => todo!(),
        SearchMatches => Style::with_bg(Color::White, Color::Red),
        CurrentMatch => Style::with_bg(Color::White, Color::Blue),
        StatusBar => todo!(),
        StatusBarInactive => todo!(),
    }
}
