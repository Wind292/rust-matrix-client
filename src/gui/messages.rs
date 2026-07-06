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
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::auth::AuthState;
use crate::content::Cache;
use crate::errors::BoxError;
use crate::gui::AppState;
use crate::gui::history::History;
use crate::gui::sidebar::SidebarWidget;
use crate::utils;

#[derive(Debug, Default, Clone)]
pub struct MessagesWidget {
    auth: AuthState,
    error: Arc<Mutex<Vec<String>>>,
    sidebar_selection: Option<usize>, // the room the sidebar is hovering over
    current_room: Option<String>,     // the room in which the messages are being displayed are from
    sidebar_width: u16,
    //                     name    subtext  roomid
    rooms: Arc<Mutex<Vec<((String, String), String)>>>,
    room_caches: Arc<Mutex<HashMap<String, Cache>>>,
    scroll: usize,
    pub exit: bool,
}

impl MessagesWidget {
    pub fn new(auth: AuthState) -> Self {
        let room_caches_mutex: Arc<Mutex<HashMap<String, Cache>>> = Arc::new(Mutex::new(HashMap::new()));
        let error_mutex: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let rooms_mutex: Arc<Mutex<Vec<((String, String), String)>>> = Arc::new(Mutex::new(vec![]));
        let cache_mutex: Arc<Mutex<HashMap<String, Cache>>> = Arc::new(Mutex::new(HashMap::new()));

        utils::async_sync(rooms_mutex.clone(), cache_mutex.clone(), error_mutex.clone(), auth.clone());

        Self {
            exit: false,
            auth,
            error: error_mutex,
            sidebar_width: 50,
            sidebar_selection: None,
            room_caches: room_caches_mutex.clone(),
            current_room: None,
            rooms: rooms_mutex,
            scroll: 0,
        }
    }
    pub async fn handle_events(&mut self, key_event: KeyEvent) -> io::Result<()> {
        self.handle_key_event(key_event).await;
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> AppState {
        match key_event.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Up => self.sidebar_increment(-1).await,
            KeyCode::Down => self.sidebar_increment(1).await,
            KeyCode::Enter => self.sidebar_select().await,
            KeyCode::PageUp => {
                self.write_auth().await;
            }
            _ => {}
        };
        AppState::Messaging(self.clone())
    }

    pub async fn write_auth(&self) -> Result<(), BoxError> {
        self.auth.save_to_disk().await;
        Ok(())
    }

    async fn sidebar_increment(&mut self, amount: i32) {
        if self.sidebar_selection.is_none() {
            self.sidebar_selection = Some(0);
            return;
        }
        let room_count = self.rooms.lock().await.len();
        
        if self.sidebar_selection == Some(0) && amount == -1 {
            self.sidebar_selection = Some(room_count-1);
            return ;
        }

        self.sidebar_selection = Some((self.sidebar_selection.unwrap() as i32 + amount) as usize);
        
        if self.sidebar_selection.unwrap_or_default() >= room_count {
            self.sidebar_selection = Some(0)
        }
    }
    
    async fn sidebar_select(&mut self) {
        let rooms = self.rooms.lock().await;
        let current_room = rooms.get(self.sidebar_selection.unwrap_or(0));

        self.current_room = Some(current_room.unwrap_or(&(("".to_string(), "".to_string()), "".to_string())).to_owned().1);
    }
}

impl Widget for MessagesWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [otherthanbar, topbar] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let [sidebar, messaging] =
            Layout::horizontal([Constraint::Length(self.sidebar_width), Constraint::Fill(1)])
                .areas(otherthanbar);

        let [title_rect, history_rect, chatbox_rect] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(3)]).areas(messaging);

        HeaderWidget::new(vec![("<esc>".to_string(), "quit".to_string())]).render(topbar, buf);

        let list_rooms: Vec<((String, String), String)> = self
            .rooms
            .try_lock()
            .as_deref()
            .cloned()
            .unwrap_or_default();

        SidebarWidget::new(list_rooms, self.sidebar_selection, self.current_room.clone()).render(sidebar, buf);
        
        let history = &History::try_from_cache(self.room_caches, self.current_room, self.scroll, area.height.into());
        match history { 
            Some(h) => {h.render(history_rect, buf);},
            None => {},
        }
    }
}

struct HeaderWidget {
    entries: Vec<(String, String)>,
}

impl HeaderWidget {
    fn new(entries: Vec<(String, String)>) -> Self {
        Self { entries }
    }
}
impl Widget for HeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut entries = vec![];

        for e in self.entries {
            entries.push(Span::styled(
                format!(" {} ", e.0),
                Style::default().fg(Color::Black).bg(Color::LightBlue),
            ));
            entries.push(Span::raw(format!(" {}     ", e.1)));
        }

        Paragraph::new(Line::from(entries))
            .alignment(Alignment::Center)
            .block(Block::default().style(Style::default().bg(Color::Blue)))
            .render(area, buf);
    }
}
