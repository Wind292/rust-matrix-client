use std::any::Any;
use std::error::Error;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::macros::vertical;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Widget;
use ratatui::widgets::{BorderType, Borders};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

struct MessagesWidget {
    text: String,
}

impl MessagesWidget {
    fn new() -> Self {
        Self { text: String::from("value") }
    }
}

impl Widget for MessagesWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.text)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}

