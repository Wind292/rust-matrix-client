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

use crate::auth::AuthState;
use crate::gui::AppState;
use crate::gui::sidebar::SidebarWidget;

#[derive(Debug, Default, Clone)]
pub struct MessagesWidget {
    auth: AuthState,
    text: String,
    sidebar_width: u16,
    pub exit: bool,
}

impl MessagesWidget {
    pub fn new(auth: AuthState) -> Self {
        Self { text: String::from("value"), exit: false, auth, sidebar_width: 20}
    }
    pub async fn handle_events(&mut self, key_event: KeyEvent) -> io::Result<()> {
        self.handle_key_event(key_event).await;
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> AppState {
        match key_event.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::PageUp => {self.write_auth().await;},
            _ => {}
        };
        AppState::Messaging(self.clone())
    }

    pub async fn write_auth(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.auth.save_to_disk().await;
        Ok(())
    }

}

impl Widget for MessagesWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [sidebar, other] = Layout::horizontal([
            Constraint::Length(self.sidebar_width),
            Constraint::Fill(1),
        ])
        .areas(area);

        SidebarWidget::new(vec![
            ("Bob".to_string(), "skbidi".to_string()),
            ("bobbythebob".to_string(), "whaatt".to_string()),
            ("someotherperon".to_string(), "Started a call".to_string()),
        ]).render(sidebar, buf);

        Paragraph::new(self.text)
            .alignment(Alignment::Center)
            .render(other, buf);
    }
}

