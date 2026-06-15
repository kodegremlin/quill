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
        Identifier => Style {
            fg: Color::White,
            bg: None,
        },
        Keyword => Style {
            fg: Color::Red,
            bg: None,
        },
        String => Style {
            fg: Color::Blue,
            bg: None,
        },
        Comment => todo!(),
        Documentation => todo!(),
        Number => todo!(),
        Type => todo!(),
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
        SearchMatches => Style {
            fg: Color::White,
            bg: Some(Color::Blue),
        },
        CurrentMatch => Style {
            fg: Color::White,
            bg: Some(Color::Red),
        },
        StatusBar => todo!(),
        StatusBarInactive => todo!(),
    }
}
