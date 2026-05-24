use std::io;
use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Widget;
use ratatui::widgets::BorderType;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::auth::{self, AuthState};
use crate::errors::MatrixError;
use crate::gui::AppState;

const CURSOR_CHAR: char = '_'; // █ ▌ _

#[derive(Debug, Default, Clone)]
pub struct LoginWidget {
    focused_field: LoginFields,
    password: String,
    username: String,
    server_address: String,
    output: Arc<Mutex<String>>,
    pub auth_state: Arc<Mutex<Option<AuthState>>>,
    pub exit: bool,
}

#[derive(PartialEq, Debug, Default, Clone)]
enum LoginFields {
    #[default]
    ServerAddress,
    Username,
    Password,
}

impl LoginWidget {
    pub fn new() -> Self {
        LoginWidget {
            focused_field: LoginFields::ServerAddress,
            exit: false,
            password: String::new(),
            username: String::new(),
            server_address: "https://".to_string(),
            output: Arc::new(Mutex::new(String::new())),
            auth_state: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn handle_events(&mut self, key_event: KeyEvent) -> io::Result<()> {
        self.handle_key_event(key_event).await;
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> AppState {
        match key_event.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Tab => self.increment_selection(),
            KeyCode::Char(c) => self.type_char(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Enter => self.submit().await,
            _ => {}
        };
        AppState::Login(self.clone())
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
            LoginFields::ServerAddress => self.server_address.pop(),
        };
    }

    async fn submit(&mut self) {
        if !(self.server_address.starts_with("https://") || self.server_address.starts_with("http://")) {
            *self.output.lock().unwrap() = "Error! Missing `https://` or `http://`".to_string();
            return;
        }

        *self.output.lock().unwrap() = "Connecting to server...".to_string();

        let output = Arc::clone(&self.output);
        let server = self.server_address.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let auth_state = self.auth_state.clone();

        tokio::spawn(async move {
            let auth_res = auth::AuthState::login_password(&server, &username, &password).await;

            let message: Result<AuthState, String> = match auth_res {
                Ok(auth) => {
                    Ok(auth)
                },
                Err(e) => {
                    if let Some(e) = e.downcast_ref::<reqwest::Error>() {
                        if e.is_builder() {
                            Err("Error! Homeserver's address is invalid".to_string())
                        } else {
                            Err("Error! Cannot connect to homeserver".to_string())
                        }
                    } else if let Some(e) = e.downcast_ref::<MatrixError>() {
                        match e {
                            MatrixError::MatrixError(_, Some(msg)) => Err(msg.clone()),
                            MatrixError::MatrixError(code, None) => Err(format!("Error! Server returned: {}", code)),
                        }
                    } else {
                        Err("Error! Unknown error".to_string())
                    }
                }
            };

            match message {
                Ok(auth) => *auth_state.lock().unwrap() = Some(auth),
                Err(e) => *output.lock().unwrap() = e,
            }
        });
    }
}

impl Widget for LoginWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header, _, address, _, username, _, password, output, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(2),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(10),
        ])
        .areas(area);

        HeaderWidget::new().render(header, buf);

        TextBoxWidget::new(
            "Homeserver".to_string(),
            self.server_address,
            self.focused_field == LoginFields::ServerAddress,
            false,
        )
        .render(address, buf);

        TextBoxWidget::new(
            "Username".to_string(),
            self.username,
            self.focused_field == LoginFields::Username,
            false,
        )
        .render(username, buf);

        TextBoxWidget::new(
            "Password".to_string(),
            self.password,
            self.focused_field == LoginFields::Password,
            true,
        )
        .render(password, buf);

        let text = self.output.lock().unwrap().clone();

        OutputWidget::new(text).render(output, buf);

    }
}

struct HeaderWidget {}

impl HeaderWidget {
    fn new() -> Self {
        Self {}
    }
}
impl Widget for HeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Line::from(vec![
            Span::styled(
                " <tab> ",
                Style::default().fg(Color::Black).bg(Color::LightBlue),
            ),
            Span::raw(" switch     "),
            Span::styled(
                " <enter> ",
                Style::default().fg(Color::Black).bg(Color::LightBlue),
            ),
            Span::raw(" submit     "),
            Span::styled(
                " <esc> ",
                Style::default().fg(Color::Black).bg(Color::LightBlue),
            ),
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
        Self {
            title,
            selected,
            inside,
            hide,
        }
    }
}

impl Widget for TextBoxWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clamp and center
        let width = 40u16.min(area.width);
        let height = 12u16.min(area.height);
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let centered = Rect::new(x, y, width, height);

        // Draw border and get inner area in one step
        let color = if self.selected {
            Color::Blue
        } else {
            Color::White
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .fg(color)
            .title(format!(" {} ", self.title))
            .title_alignment(Alignment::Center);

        let inner = block.inner(centered); // ← computes the inner Rect for you
        block.render(centered, buf);

        // Render content inside
        let mut inside = self.inside;
        if self.hide {
            inside = "*".repeat(inside.len())
        }
        if self.selected {
            inside.push(CURSOR_CHAR);
        }
        Paragraph::new(inside)
            .alignment(Alignment::Left)
            .render(inner, buf);
    }
}


struct OutputWidget {
    text: String,
}

impl OutputWidget {
    fn new(text: String) -> Self {
        Self { text }
    }
}

impl Widget for OutputWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.text)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}

