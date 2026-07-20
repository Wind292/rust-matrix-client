use ratatui::{buffer::Buffer, layout::{Alignment, Constraint, Layout, Margin, Rect}, style::{Color, Modifier, Style}, widgets::{Block, Borders, Padding, Paragraph, Widget}};

pub struct InputBoxWidget {
    lines: Vec<String>,
}

impl InputBoxWidget {
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

impl Widget for InputBoxWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        

        // let [bar, text] =
        //     Layout::vertical([ Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let border_box = Block::default()
            .borders(Borders::all())
            .title("── Wind ") // TODO: change this to be something cool
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let text = border_box.inner(area);

        border_box.render(area, buf);
        
        let visible = self.lines.len().min(text.height as usize);
        let start_index = self.lines.len() - visible; // drop oldest rows that don't fit
        let y_offset = text.height as usize - visible;     // blank rows go on top, not bottom

        for (row_index, row) in self.lines[start_index..].iter().enumerate() {
            let y = text.y.saturating_add((y_offset + row_index) as u16);
            if y >= text.y.saturating_add(text.height) {
                break;
            }

            let row_area = Rect { x: text.x, y, width: text.width, height: 1 };
            row.clone().render(row_area, buf);
        }








    }
}
