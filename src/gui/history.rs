use ratatui::{layout::Rect, widgets::Widget};

use crate::content::Event;

pub struct History {
    messages: Vec<HistoryRow>,
}

impl History {
    pub fn render(&self, area: Rect) {
        
    }
}

pub struct HistoryRow {}

impl Widget for HistoryRow {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {}
}
