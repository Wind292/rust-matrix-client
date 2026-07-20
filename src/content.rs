use core::{error, fmt, time};
use std::collections::HashMap;
use std::mem::transmute;
use std::os::unix::process::parent_id;
use serde::de::value;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::errors::CustomError::*;
use crate::events::{EventState, Message, get_messages};
use serde_json::Value;
use serde::Deserialize;
use crate::errors::BoxError;

pub struct ClientState {
    rooms: HashMap<String, (Cache, Cache)>,
    next_token: String,
}


#[derive(Debug, Default, Clone)]
pub struct Cache {
    pub events: Vec<Event>,
    before_token: String,
    roomid: String,
    total_history: Option<bool>,
}



#[derive(Debug, Deserialize, Default)]
pub struct Rooms {
    #[serde(default)]
    pub join: HashMap<String, Room>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Room {
    #[serde(default)]
    pub state: State,
    #[serde(default)]
    pub timeline: Timeline,
}

#[derive(Debug, Deserialize, Default)]
pub struct StateEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub content: Value,
}

#[derive(Debug, Deserialize, Default)]
pub struct Timeline {
    pub events: Vec<Value>,
    pub prev_batch: String,
    #[serde(default)]
    pub limited: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct State {
    pub events: Vec<Value>,
    #[serde(default)]
    pub limited: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct CacheRoom {
    pub state: Cache,
    pub timeline: Cache,
}


impl CacheRoom { 
    pub fn append(&mut self, mut newer_room: Self) {
        self.state.before_token = newer_room.state.before_token;
        self.timeline.before_token = newer_room.timeline.before_token;

        self.state.events.append(&mut newer_room.state.events);
        self.timeline.events.append(&mut newer_room.timeline.events);

        self.state.total_history = newer_room.state.total_history;
        self.timeline.total_history= newer_room.timeline.total_history;
    }
}


impl Cache {
    pub async fn update_before(
        &mut self,
        auth_state: crate::auth::AuthState,
    ) -> Result<(), BoxError> {
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
                Event::Creation(e) => {
                    println!("Room Created by {:?}", e.creators)
                }
            }
        }
    }

    pub async fn from_rooms(
        rooms: Rooms,
    ) -> Result<HashMap<String, CacheRoom>, BoxError> {
        let mut caches: HashMap<String, CacheRoom> = HashMap::new();

        for (roomid, room) in rooms.join {
            let state_events: Vec<Event> = room
                .state
                .events
                .into_iter()
                .rev()
                .map(parse_event)
                .collect::<Result<_, _>>()?;

            let timeline_events: Vec<Event> = room
                .timeline
                .events
                .into_iter()
                .rev()
                .map(parse_event)
                .collect::<Result<_, _>>()?;

            let state_cache = Cache {
                events: state_events,
                before_token: "".to_string(),
                roomid: roomid.clone(),
                total_history: None,
            };

            let timeline_cache = Cache {
                events: timeline_events,
                before_token: room.timeline.prev_batch,
                roomid: roomid.clone(),
                total_history: room.timeline.limited.map(|limited| !limited),
            };

            caches.insert(roomid, CacheRoom{ state: state_cache, timeline: timeline_cache });
        }

        Ok(caches)
    }

    pub fn spin_updater_thread(mutex: Arc<Mutex<Self>>) {
        let mutex_clone = mutex.clone();
        todo!();
        tokio::spawn(async move {

        });
    }

}

pub fn parse_event(event_json: Value) -> Result<Event, BoxError> {
    let event_type = event_json
        .get("type")
        .and_then(|f| f.as_str())
        .ok_or(MissingRequiredField)?;
    Ok(match event_type {
        "m.room.message" => Event::Message(MessageEvent::format(event_json.clone())?),
        "m.room.name" => Event::Name(NameEvent::format(event_json.clone())?),
        "m.room.create" => Event::Creation(CreationEvent::format(event_json.clone())?),
        _ => Event::Unknown(UnknownEvent::format(event_json.clone())?), // unsupported type
    })
}


#[derive(Debug, Clone)]
pub enum Event {
    Message(MessageEvent),
    Name(NameEvent),
    Unknown(UnknownEvent),
    Creation(CreationEvent)
}

impl Event {
    pub fn summary(&self) -> String {
        match self {
            Event::Message(message_event) => message_event.summary(),
            Event::Name(name_event) => name_event.summary(),
            Event::Unknown(unknown_event) => unknown_event.summary(),
            Event::Creation(creation_event) => creation_event.summary(),
        }
    }

}

impl Default for Event { 
    fn default() -> Self {
        Self::Unknown(UnknownEvent::default())
    }
}

#[derive(Debug, Clone)]
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
    fn format(json: Value) -> Result<Self, BoxError> {
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

    fn summary(&self) -> String{
        // let sender_prefix = self.sender.clone().and_then(|s| Some(format!("{}: ", s))).unwrap_or("".to_string());
        format!("{}", self.body.to_string())
    }
 
    fn display(&self) -> bool {
        true
    }
}

#[derive(Debug, Default, Clone)]
pub struct NameEvent {
    pub name: String,
    pub sender: Option<String>,
    pub event_id: Option<String>,
    pub time: Option<u64>,
}

impl NameEvent {
    fn format(json: Value) -> Result<Self, BoxError> {
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

    fn summary(&self) -> String{
        format!("Room renamed")
    }

    fn display(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct UnknownEvent {
    pub event_type: String,
    pub sender: Option<String>,
    pub event_id: Option<String>,
    pub time: Option<u64>,
}

impl UnknownEvent {
    fn format(json: Value) -> Result<Self, BoxError> {
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

    fn summary(&self) -> String{
        format!("{}", self.event_type.to_string())
    }

    fn display(&self) -> bool {
        true
    }
}

impl Default for UnknownEvent {
    fn default() -> Self {
        Self { event_type: "default".to_string(), sender: None, event_id: None, time: None }
    }
}

#[derive(Debug, Clone)]
pub struct CreationEvent {
    pub event_type: String,
    pub creators: Vec<String>,
    pub is_federated: Option<bool>,
    pub event_id: Option<String>,
    pub time: Option<u64>,
    pub room_version: Option<u32>,
    pub formatted: Option<String>
}

impl CreationEvent {
    fn format(json: Value) -> Result<Self, BoxError> {
        let event_type = json
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or(MissingRequiredField)?
            .to_string();

        let sender = json
            .get("sender")
            .and_then(|s| s.as_str())
            .ok_or(MissingRequiredField)?
            .to_string();

        let event_id = json
            .get("event_id")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let time = json.get("origin_server_ts").and_then(|t| t.as_u64());

        let content = json.get("content");

        let room_version: Option<u32> = content
            .and_then(|c| c.get("room_version"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .or(Some(1));

        let is_federated = content
            .and_then(|c| c.get("m.federate"))
            .and_then(|v| v.as_bool())
            .or(Some(true));

        // Figure out creator depending on room version:
        // - v1-10: use content.creator (fall back to sender if missing)
        // - v11+: sender is the (sole) creator
        // - v12+: sender plus content.additional_creators
        let mut creators: Vec<String> = Vec::new();
        let is_pre_v11 = matches!(room_version, Some(v) if v < 11);

        if is_pre_v11 {
            let creator = content
                .and_then(|c| c.get("creator"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| sender.clone());
            creators.push(creator);
        } else {
            creators.push(sender.clone());

            if matches!(room_version, Some(v) if v >= 12) {
                if let Some(additional) = content
                    .and_then(|c| c.get("additional_creators"))
                    .and_then(|a| a.as_array())
                {
                    for uid in additional {
                        if let Some(uid_str) = uid.as_str() {
                            creators.push(uid_str.to_string());
                        }
                    }
                }
            }
        }

        let creators_string = creators.join(", ");

        let formatted = Some(format!("Room created by {}", creators_string ));

        Ok(CreationEvent {
            event_type,
            creators,
            is_federated,
            event_id,
            time,
            room_version,
            formatted,
        })
    }

    fn summary(&self) -> String{
        "Empty Room".to_string()
    }

    fn display(&self) -> bool {
        true
    }
}

#[derive(Debug, Default, Clone)]
pub struct MemberEvent {
    pub name: String,
    pub sender: Option<String>,
    pub event_id: Option<String>,
    pub time: Option<u64>,
}

impl MemberEvent {
    fn format(json: Value) -> Result<Self, BoxError> {
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

        Ok(MemberEvent {
            name: name.to_string(),
            sender: sender,
            event_id: event_id,
            time: time,
        })
    }

    fn summary(&self) -> String{
        format!("{}", self.name.to_string())
    }

    fn display(&self) -> bool {
        true
    }
}

