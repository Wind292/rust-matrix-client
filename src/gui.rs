mod login;
mod messages;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}};

use crate::gui::login::LoginWidget;

#[derive(Debug, Default)]
pub struct App {
    state: AppState,
    exit: bool,
}

#[derive(Debug, Default)]
pub enum AppState {
    #[default]
    Start,
    Login(LoginWidget),
    Messaging
}

impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match &self.state {
            AppState::Start => frame.render_widget(self, frame.area()),
            AppState::Login(w) => frame.render_widget(w.clone(), frame.area()),
            AppState::Messaging => frame.render_widget(self, frame.area()),
        }
        
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match &mut self.state {
            AppState::Start => self.state = AppState::Login(LoginWidget::new()),
            AppState::Login(w) => {
                w.handle_events()?;
                self.exit = w.exit;
            },
            AppState::Messaging => {},
        }
        Ok(())
    }


    fn exit(&mut self) {
        self.exit = true;
    }


}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) { }
}