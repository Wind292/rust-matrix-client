use core::time;
use std::collections::HashMap;
use std::mem::transmute;
use std::os::unix::process::parent_id;

use crate::errors::CustomError::*;
use crate::events::{EventState, Message, get_messages};
use serde_json::Value;
pub struct ClientState {
    rooms: HashMap<String, (Cache, Cache)>,
    next_token: String,
}

#[derive(Debug)]
pub struct Cache {
    events: Vec<Event>,
    before_token: String,
    roomid: String,
    total_history: Option<bool>,
}

impl Cache {
    pub async fn update_before(
        &mut self,
        auth_state: &mut crate::auth::AuthState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut response = get_messages(
            auth_state,
            self.roomid.clone(),
            "b",
            20,
            Some(self.before_token.clone()),
        )
        .await?;
        let mut events: &mut Vec<Value> = &mut response
            .get("chunk")
            .and_then(|f| f.as_array())
            .ok_or(MissingRequiredField)?
            .to_vec();

        // events;
        for event_json in events {
            let event: Event = parse_event(event_json.to_owned())?;
            self.events.push(event);
        }

        let end_token = response
            .get("end")
            .and_then(|f| f.as_str())
            .and_then(|f| Some(f.to_string()));

        if end_token.is_none() {
            self.total_history = Some(true);
            return Ok(());
        }

        self.before_token = end_token.unwrap();

        Ok(())
    }

    pub fn debug_types(self) {
        for event in self.events.into_iter() {
            match event {
                Event::Message(e) => {
                    println!("Message: {}", e.body)
                }
                Event::Name(e) => {
                    println!("Name set to: {}", e.name)
                }
                Event::Unknown(e) => {
                    println!("Unknown of type: {}", e.event_type)
                }
            }
        }
    }

    pub async fn from_rooms(
        rooms: Value,
    ) -> Result<HashMap<String, (Self, Self)>, Box<dyn std::error::Error>> {
        // Rooms should be something like:
        // {
        //   roomid: {
        //     state: { events: [] },
        //     timeline: { events: [] }
        //   }
        // }
        //
        let mut caches: HashMap<String, (Self, Self)> = HashMap::new();

        for room in rooms.as_object().ok_or(MissingRequiredField)? {
            let roomid = room.0;
            let room = room.1;

            let mut state_events: Vec<Event> = Vec::new();
            let mut timeline_events: Vec<Event> = Vec::new();

            let state: &Vec<Value> = room
                .get("state")
                .and_then(|f| f.get("events"))
                .and_then(|f| f.as_array())
                .ok_or(MissingRequiredField)?;
            let timeline: &Vec<Value> = room
                .get("timeline")
                .and_then(|f| f.get("events"))
                .and_then(|f| f.as_array())
                .ok_or(MissingRequiredField)?;

            // Populate the event lists
            for state_event in state.iter().rev() {
                state_events.push(parse_event(state_event.to_owned())?);
            }
            for timeline_event in timeline.iter().rev() {
                timeline_events.push(parse_event(timeline_event.to_owned())?);
            }

            // This room's caches
            let state_cache = Cache {
                // First is State
                events: state_events,
                before_token: "".to_string(), // can leave blank for state
                roomid: roomid.to_string(),
                total_history: None,
            };

            let before_token = room
                .get("timeline")
                .and_then(|f| f.get("prev_batch"))
                .and_then(|f| f.as_str())
                .and_then(|f| Some(f.to_string()))
                .ok_or(MissingRequiredField)?;
            let total_history = room
                .get("timeline")
                .and_then(|f| f.get("limited"))
                .and_then(|f| f.as_bool())
                .and_then(|f| Some(!f));
            let timeline_cache = Cache {
                // Second is timeline
                events: timeline_events,
                before_token: before_token,
                roomid: roomid.to_string(),
                total_history: total_history,
            };

            caches.insert(roomid.to_string(), (state_cache, timeline_cache));
        }

        Ok(caches)
    }
}

fn parse_event(event_json: Value) -> Result<Event, Box<dyn std::error::Error>> {
    let event_type = event_json
        .get("type")
        .and_then(|f| f.as_str())
        .ok_or(MissingRequiredField)?;
    Ok(match event_type {
        "m.room.message" => Event::Message(MessageEvent::format(event_json.clone())?),
        "m.room.name" => Event::Name(NameEvent::format(event_json.clone())?),
        _ => Event::Unknown(UnknownEvent::format(event_json.clone())?), // unsupported type
    })
}

#[derive(Debug)]
pub enum Event {
    Message(MessageEvent),
    Name(NameEvent),
    Unknown(UnknownEvent),
}
#[derive(Debug)]
pub struct MessageEvent {
    pub body: String,
    pub msgtype: String,
    pub formatted: Option<String>,
    pub sender: Option<String>,
    pub event_id: Option<String>,
    pub room_id: Option<String>,
    pub time: Option<u64>,
}

impl MessageEvent {
    fn format(json: Value) -> Result<Self, Box<dyn std::error::Error>> {
        let content = json.get("content").ok_or(MissingRequiredField)?;
        let msg_type = content
            .get("msgtype")
            .and_then(|t| t.as_str())
            .ok_or(MissingRequiredField)?;
        let body = content
            .get("body")
            .and_then(|t| t.as_str())
            .ok_or(MissingRequiredField)?;

        let format_type = content.get("format").and_then(|t| t.as_str());
        let mut format_string = content
            .get("formatted_body")
            .and_then(|t| t.as_str())
            .and_then(|s| Some(s.to_string()));
        if format_type != Some("org.matrix.custom.html") {
            format_string = None
        } // unsupported formatting; fallbacks to body

        let sender = json
            .get("sender")
            .and_then(|t| t.as_str())
            .and_then(|s| Some(s.to_string()));
        let event_id = json
            .get("event_id")
            .and_then(|t| t.as_str())
            .and_then(|s| Some(s.to_string()));
        let room_id = json
            .get("room_id")
            .and_then(|t| t.as_str())
            .and_then(|s| Some(s.to_string()));
        let time = json.get("origin_server_ts").and_then(|t| t.as_u64());

        Ok(MessageEvent {
            body: body.to_string(),
            msgtype: msg_type.to_string(),
            formatted: format_string,
            sender: sender,
            event_id: event_id,
            room_id: room_id,
            time: time,
        })
    }

    fn display(&self) -> bool {
        true
    }
}
#[derive(Debug)]
pub struct NameEvent {
    pub name: String,
    pub sender: Option<String>,
    pub event_id: Option<String>,
    pub time: Option<u64>,
}

impl NameEvent {
    fn format(json: Value) -> Result<Self, Box<dyn std::error::Error>> {
        let content = json.get("content").ok_or(MissingRequiredField)?;
        let name = content
            .get("name")
            .and_then(|t| t.as_str())
            .ok_or(MissingRequiredField)?;

        let sender = json
            .get("sender")
            .and_then(|t| t.as_str())
            .and_then(|s| Some(s.to_string()));
        let event_id = json
            .get("event_id")
            .and_then(|t| t.as_str())
            .and_then(|s| Some(s.to_string()));
        let time = json.get("origin_server_ts").and_then(|t| t.as_u64());

        Ok(NameEvent {
            name: name.to_string(),
            sender: sender,
            event_id: event_id,
            time: time,
        })
    }

    fn display(&self) -> bool {
        true
    }
}
#[derive(Debug)]
pub struct UnknownEvent {
    pub event_type: String,
    pub sender: Option<String>,
    pub event_id: Option<String>,
    pub time: Option<u64>,
}

impl UnknownEvent {
    fn format(json: Value) -> Result<Self, Box<dyn std::error::Error>> {
        let event_type = json
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or(MissingRequiredField)?;

        let sender = json
            .get("sender")
            .and_then(|t| t.as_str())
            .and_then(|s| Some(s.to_string()));
        let event_id = json
            .get("event_id")
            .and_then(|t| t.as_str())
            .and_then(|s| Some(s.to_string()));
        let time = json.get("origin_server_ts").and_then(|t| t.as_u64());

        Ok(UnknownEvent {
            event_type: event_type.to_string(),
            sender: sender,
            event_id: event_id,
            time: time,
        })
    }

    fn display(&self) -> bool {
        true
    }
}
