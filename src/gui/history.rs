use std::{collections::{HashMap, VecDeque}, sync::Arc};

use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::{Paragraph, Widget}};
use tokio::sync::Mutex;

use crate::{content::{Cache, CacheRoom, Event, UnknownEvent}, utils::async_update_rooms};

pub struct History {
    messages: Vec<HistoryRow>,
}

impl History {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let visible = self.messages.len().min(area.height as usize);
        let start_index = self.messages.len() - visible; // drop oldest rows that don't fit
        let y_offset = area.height as usize - visible;     // blank rows go on top, not bottom

        for (row_index, row) in self.messages[start_index..].iter().enumerate() {
            let y = area.y.saturating_add((y_offset + row_index) as u16);
            if y >= area.y.saturating_add(area.height) {
                break;
            }

            let row_area = Rect { x: area.x, y, width: area.width, height: 1 };
            row.clone().render(row_area, buf);
        }
    }

    pub fn try_from_cache(cache: Arc<Mutex<HashMap<String, CacheRoom>>>, roomid: Option<String>, start: usize, length: usize) -> Option<Self> {
        let c = cache.try_lock();

        if c.is_err() {
            return None
        }

        let temp = c.unwrap();
        let c = temp.get(&roomid.unwrap_or_default());

        if c.is_none() { 
            return None
        }

        let c = &c.unwrap().timeline;

        let mut history_rows: VecDeque<HistoryRow> = VecDeque::new();
        let mut i = 0usize; // rows counted so far, walking from the newest event backwards

        for e in c.events.iter().rev() {
            let row = HistoryRow::from_event(e.clone());
            let row_len = row.len();

            // Still within the "skip" zone closer to newest than `start` — count it, don't keep it
            if i + row_len <= start {
                i += row_len;
                continue;
            }

            // Already collected `length` rows past the skip offset — stop
            if i >= start + length {
                break;
            }

            i += row_len;

            // Push each row to the front, preserving internal order, so the deque
            // ends up oldest -> newest without needing a separate full reverse.
            for r in row.into_iter().rev() {
                history_rows.push_front(r);
            }
        }

        Some(Self {
            messages: history_rows.into_iter().rev().collect(),
        })
    }
}

#[derive(Clone)]
pub struct HistoryRow {
    content: Line<'static>
}

impl Widget for HistoryRow {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.content.render(area, buf);
    }
}

impl HistoryRow {
    pub fn new(content: Line<'static>) -> Self {
        HistoryRow { content }
    }

    pub fn from_event(event: Event) -> Vec<HistoryRow> {
        match event {
            Event::Message(message_event) => {
                let sender =  message_event.sender.unwrap_or("Unknown".to_string());
                let body =  message_event.body;
                vec![HistoryRow::new(Line::from(format!("{}: {}", sender, body)))]
            },
            Event::Name(name_event) => {
                let sender =  name_event.sender.unwrap_or("Unknown".to_string());
                let name = name_event.name;
                vec![HistoryRow::new(Line::from(format!("{} renamed the room to {}", sender, name)))]
            },
            Event::Creation(creation_event) => {
                let creators = creation_event.creators;
                vec![HistoryRow::new(Line::from(format!("This room was created by {}", creators.join(", "))))]   
            }
            Event::Unknown(unknown_event) => {
                let event_type =  unknown_event.event_type;
                vec![HistoryRow::new(Line::from(format!("Unknown Event of {}", event_type)))]
            },
        }
    }
}