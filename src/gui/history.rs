use std::{collections::HashMap, sync::Arc};

use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::{Paragraph, Widget}};
use tokio::sync::Mutex;

use crate::{content::{Cache, CacheRoom, Event, UnknownEvent}, utils::async_update_rooms};

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

        let slice: Vec<&Event> = c.events[..].iter().clone().collect();
        let mut history_rows: Vec<HistoryRow> = vec![];

        let mut i = 0;
        for e in slice.iter().rev() {
            // Generated the formatted HistoryRows in order of oldest to newest
            let row = HistoryRow::from_event((*e).clone());
            // Increment the line counter
            i += row.len(); 
            // If you are outside the context window, do not save the lines
            if i < slice.len() - start {continue;}
            if i >= (slice.len() - start) + length { break; } // after the window, so we can just break
            
            println!("start: {}, end: {}", slice.len() - start, (slice.len() - start) + length);
            
            // Save the HistoryRows
            history_rows.extend(row);
        }

        Some( Self {
            messages: history_rows
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