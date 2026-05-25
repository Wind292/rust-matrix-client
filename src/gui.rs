mod login;
mod messages;
mod sidebar;

use std::io;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

use crate::{auth::{self, AuthState}, gui::{login::LoginWidget, messages::MessagesWidget}};

#[derive(Debug, Default)]
pub struct App {
    state: AppState,
    exit: bool,
}

#[derive(Debug, Default, Clone)]
pub enum AppState {
    #[default]
    Start,
    Login(LoginWidget),
    Messaging(MessagesWidget),
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match &mut self.state.clone() {
            AppState::Start => frame.render_widget(&*self, frame.area()),
            AppState::Login(w) => {
                if w.auth_state.lock().unwrap().is_some() {
                    self.state = AppState::Messaging(MessagesWidget::new(w.auth_state.lock().unwrap().clone().unwrap()))
                }
                frame.render_widget(w.clone(), frame.area())
            },
            AppState::Messaging(w) => {
                frame.render_widget(w.clone(), frame.area())
            },
        }
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        match self.state { 
            AppState::Start => {
                let auth = AuthState::revive_from_disk().await;
                if auth.is_some() {
                    self.state = AppState::Messaging(MessagesWidget::new(auth.unwrap()));
                } else {
                    self.state = AppState::Login(LoginWidget::new());
                }         
            } 
            _=>{}
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match &mut self.state {
                        AppState::Start => self.state = AppState::Login(LoginWidget::new()),
                        AppState::Login(w) => {
                            w.handle_events(key_event).await?;
                            if w.exit { self.exit = true; }
                        }
                        AppState::Messaging(w) => {
                            w.handle_events(key_event).await?;
                            if w.exit { self.exit = true; }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}
