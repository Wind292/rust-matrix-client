use ratatui::style::{Color, Modifier, Style};
use ratatui::layout::Alignment;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Borders, Widget};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph},
};

const ROOMS_START_OFFSET: u16 = 2;

pub struct SidebarWidget {
    rooms: Vec<(String, String)>,
    selected: Option<usize>,
}

impl SidebarWidget {
    pub fn new(rooms: Vec<(String, String)>, selected: Option<usize>) -> Self {
        Self { rooms, selected}
    }

}

impl Widget for SidebarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Outer border
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::new().fg(Color::DarkGray));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 { return; }

        let header_rect = Rect { y: inner.y, height: 1, ..inner };
        Paragraph::new(" Rooms")
            .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .render(header_rect, buf);

        if inner.height < 2 { return; }

        let divider_rect = Rect { y: inner.y + 1, height: 1, ..inner };
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray))
            .render(divider_rect, buf);

        if self.rooms.is_empty() { return; }


        let max_y = inner.y.saturating_add(inner.height);

        for (i, (name, last_msg)) in self.rooms.into_iter().enumerate() {
            let base_y = inner.y
                .saturating_add(ROOMS_START_OFFSET)
                .saturating_add(i as u16);

            // nothing below this point is visible
            if base_y >= max_y { break; }

            let is_selected = self.selected == Some(i);
            let name_style = if is_selected {
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let name_rect = Rect { y: base_y, height: 1, ..inner };
            let line = Line::from(vec![
                Span::styled(format!(" {}", name), name_style),
                Span::raw(": "),
                Span::styled(
                    last_msg,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]);
            Paragraph::new(line).render(name_rect, buf);

        }
    }
}