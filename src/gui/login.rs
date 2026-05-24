use std::any::Any;
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::macros::vertical;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph}};
use ratatui::widgets::{Borders, BorderType};

const CURSOR_CHAR: char = '_'; // █ ▌ _


#[derive(PartialEq,Debug, Default, Clone)]
pub struct LoginWidget {
    focused_field: LoginFields,
    password: String,
    username: String,
    server_address: String,
    pub exit: bool
}


#[derive(PartialEq,Debug, Default, Clone)]
enum LoginFields { 
    #[default]
    ServerAddress,
    Username,
    Password,
}

impl LoginWidget {
    pub fn new() -> Self {
        LoginWidget { focused_field: LoginFields::ServerAddress, exit: false, password: String::new(), username: String::new(), server_address: String::new() }
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Tab => self.increment_selection(),
            KeyCode::Char(c) => self.type_char(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Enter => self.submit(),
            _=>{}
        }
    }

    fn increment_selection(&mut self) {
        match self.focused_field {
            LoginFields::ServerAddress => self.focused_field = LoginFields::Username,
            LoginFields::Username => self.focused_field = LoginFields::Password,
            LoginFields::Password => self.focused_field = LoginFields::ServerAddress,
        }
    }

    fn type_char(&mut self, c: char) {
        match self.focused_field {
            LoginFields::Username => self.username.push(c),
            LoginFields::Password => self.password.push(c),
            LoginFields::ServerAddress => self.server_address.push(c),
        }
    }


    fn backspace(&mut self) {
        match self.focused_field {
            LoginFields::Username => self.username.pop(),
            LoginFields::Password => self.password.pop(),
            LoginFields::ServerAddress => self.server_address.pop()
        };
    }

    fn submit(&mut self) {
        
    }

}



impl Widget for LoginWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header, n, address, n2,  username, n3, password, n4] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(2), 
            Constraint::Length(3),  
            Constraint::Fill(1),   
            Constraint::Length(3), 
            Constraint::Fill(1),   
            Constraint::Length(3), 
            Constraint::Fill(2),    
        ]).areas(area);
        
        HeaderWidget::new().render(header, buf);
        TextBoxWidget::new("Homeserver".to_string(), self.server_address, self.focused_field == LoginFields::ServerAddress, false).render(address, buf);
        TextBoxWidget::new("Username".to_string(), self.username, self.focused_field == LoginFields::Username, false).render(username, buf);
        TextBoxWidget::new("Password".to_string(), self.password, self.focused_field == LoginFields::Password, true).render(password, buf);
    }

    
}

struct HeaderWidget {

}

impl HeaderWidget {
    fn new() -> Self {
        Self {  }
    }
}
impl Widget for HeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Line::from(vec![
            Span::styled(" <tab> ", Style::default().fg(Color::Black).bg(Color::LightBlue)),
            Span::raw(" switch     "),
            Span::styled(" <enter> ", Style::default().fg(Color::Black).bg(Color::LightBlue)),
            Span::raw(" submit     "),
            Span::styled(" <esc> ", Style::default().fg(Color::Black).bg(Color::LightBlue)),
            Span::raw(" quit"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().style(Style::default().bg(Color::Blue)))
        .render(area, buf);
            
    }
}



struct TextBoxWidget {
    title: String,
    inside: String,
    selected: bool,
    hide: bool,
}

impl TextBoxWidget {
    fn new(title: String, inside: String, selected: bool, hide: bool) -> Self {
        Self { title, selected, inside, hide }
    }
}

impl Widget for TextBoxWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clamp and center
        let width  = 40u16.min(area.width);
        let height = 12u16.min(area.height);
        let x = area.x + (area.width.saturating_sub(width))  / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let centered = Rect::new(x, y, width, height);

        // Draw border and get inner area in one step
        let color = if self.selected { Color::Blue } else { Color::White };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .fg(color)
            .title(format!(" {} ", self.title))
            .title_alignment(Alignment::Center);

        let inner = block.inner(centered); // ← computes the inner Rect for you
        block.render(centered, buf);

        // Render content inside
        let mut inside = self.inside;
        if self.hide { inside = "*".repeat(inside.len()) }
        if self.selected { inside.push(CURSOR_CHAR); }
        Paragraph::new(inside)
            .alignment(Alignment::Left)
            .render(inner, buf);
    }
}


