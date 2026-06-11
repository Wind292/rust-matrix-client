use std::sync::Arc;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tokio::sync::Mutex;

use crate::content::{Cache, Event, UnknownEvent};

pub struct History {
    messages: Vec<HistoryRow>,
}

impl History {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        for (row_index, row) in self.messages.iter().enumerate() {
            let y = area.y.saturating_add(row_index as u16);
            if y >= area.y.saturating_add(area.height) {
                break;
            }

            let row_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };

            row.clone().render(row_area, buf);
        }
    }

    pub async fn from_cache(cache: Arc<Mutex<Cache>>, start: usize, length: usize) {
        let mut c = cache.lock().await;
  
        let slice: Vec<&Event> = c.events[start..length+start].iter().clone().collect();
    }
}

#[derive(Clone)]
pub struct HistoryRow {}

impl Widget for HistoryRow {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {}
}
