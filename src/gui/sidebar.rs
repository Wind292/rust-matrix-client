use ratatui::style::{Color, Modifier, Style};
use ratatui::layout::{Alignment, Constraint, Layout};

use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Borders, Widget};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph},
};

pub struct SidebarWidget {
    rooms: Vec<(String, String)>,
}

impl SidebarWidget {
    pub fn new(rooms: Vec<(String, String)>) -> Self {
        Self { rooms }
    }
}

impl Widget for SidebarWidget {
fn render(self, area: Rect, buf: &mut Buffer) {
    // Right border for the whole sidebar
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::new().fg(Color::DarkGray));
    let inner = block.inner(area);
    block.render(area, buf);

    if self.rooms.is_empty() { return; }

    // 1 header + 1 spacer + 2 rows per room
    let mut constraints = vec![
        Constraint::Length(1), // header
        Constraint::Length(1), // spacer
    ];
    for _ in &self.rooms {
        constraints.push(Constraint::Length(1)); // room name
        constraints.push(Constraint::Length(1)); // last message
        constraints.push(Constraint::Length(1)); // gap between rooms
    }

    let rows = Layout::vertical(constraints).split(inner);

    // Header
    Paragraph::new(" Rooms")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .render(rows[0], buf);

    // Divider line manually via block
    Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .render(rows[1], buf);

    for (i, room) in self.rooms.into_iter().enumerate() {
        let name_row = rows[2 + (i )];

        // Highlight selected room
        let is_selected = false;//Some(i) == self.selected;
        let name_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let line = Line::from(vec![
            Span::styled(format!(" {}", room.0), name_style),
            Span::raw(": "),
            Span::styled(room.1, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]);

        Paragraph::new(line).render(name_row, buf);
    }
}
}