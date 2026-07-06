use serde_json::{Value, json, value};
use uuid::Uuid;

use crate::auth::AuthState;
use crate::content;
use crate::{auth, errors, utils::*};
use crate::errors::BoxError;

#[derive(Debug)]
pub struct EventState {
    pub account_data: Option<Value>,
    pub device_one_time_keys_count: Option<Value>,
    pub device_unused_fallback_key_types: Option<Vec<Value>>,
    pub next_batch: Option<String>,
    pub presence: Option<Value>,
    pub rooms: Option<Value>,
}

impl EventState {
    pub fn new() -> Self {
        EventState {
            account_data: None,
            device_one_time_keys_count: None,
            device_unused_fallback_key_types: None,
            next_batch: None,
            presence: None,
            rooms: None,
        }
    }

    pub async fn sync(
        &mut self,
        auth_state: AuthState,
        since: Option<String>,
    ) -> Result<(), BoxError> {
        let mut url = "v3/sync".to_string();

        if since.is_some() {
            url = format!("{}?timeout=30000&since={}", url, since.unwrap());
        }

        let response = auth_state.auth_get(&url).await;
        match response {
            Ok(resp) => {
                let account_data = resp.get("account_data");
                let device_one_time_keys_count = resp.get("device_one_time_keys_count");
                let device_unused_fallback_key_types = resp.get("device_unused_fallback_key_types");
                let rooms = resp.get("rooms").and_then(|f| Some(f.to_owned()));
                let next_batch = resp.get("next_batch");
                let presence = resp.get("presence");

                self.account_data = account_data.cloned();
                self.device_one_time_keys_count = device_one_time_keys_count.cloned();
                self.device_unused_fallback_key_types = device_unused_fallback_key_types
                    .and_then(|t| t.as_array())
                    .cloned();
                self.next_batch = next_batch
                    .cloned()
                    .and_then(|t| Some(t.as_str().unwrap_or("").to_string()));
                self.presence = presence.cloned();
                self.rooms = rooms;
            },
            Err(e) => {
                let e: BoxError = e.to_string().into();
                return Err(e)
            },
        }
        Ok(())
    }

    pub fn extract_events(
        events: Vec<Value>,
    ) -> Result<Vec<content::Event>, BoxError> {
        todo!()
    }
}

pub async fn get_rooms(
    auth_state: AuthState,
) -> Result<Vec<String>, BoxError> {
    let response = auth_state.auth_get("v3/joined_rooms").await?;

    let rooms: Option<&Vec<Value>> = response.get("joined_rooms").and_then(|t| t.as_array());

    if rooms.is_none() {
        // Make sure the server responded with a `joined_rooms` field
        return Err(errors::CustomError::MissingRequiredField.into());
    }

    let mut str_rooms: Vec<String> = Vec::new();

    for room in rooms.unwrap() {
        // Iterate over all of the Values and make sure they are strings
        let possible_roomid = room.as_str();
        match possible_roomid {
            None => return Err(errors::CustomError::InvalidDataType.into()),
            Some(id) => str_rooms.push(id.to_string()),
        }
    }

    Ok(str_rooms)
}

pub async fn leave_room(
    auth_state: AuthState,
    roomid: String,
    reason: &str,
) -> Result<(), BoxError> {
    let body = &json!({
        "reason": reason,
    })
    .to_string();

    let response = auth_state
        .auth_post(&format!("v3/rooms/{}/leave", roomid), body)
        .await?;

    Ok(())
}

pub async fn forget_room(
    auth_state: AuthState,
    roomid: String,
    reason: &str,
) -> Result<(), BoxError> {
    let body = &json!({
        "reason": reason,
    })
    .to_string();

    let response = auth_state
        .auth_post(&format!("v3/rooms/{}/forget", roomid), body)
        .await?;

    Ok(())
}

pub async fn room_summary(
    auth_state: AuthState,
    roomid: String,
) -> Result<Room, BoxError> {
    let response = auth_state
        .auth_get(&format!("v1/room_summary/{}", roomid))
        .await?;

    let response_room_id: String = response
        .get("room_id")
        .ok_or(errors::CustomError::MissingRequiredField)?
        .as_str()
        .unwrap_or("")
        .to_string();
    let response_num_joined_members: u64 = response
        .get("num_joined_members")
        .ok_or(errors::CustomError::MissingRequiredField)?
        .as_u64()
        .unwrap_or(0);
    let world_readable: bool = response
        .get("world_readable")
        .ok_or(errors::CustomError::MissingRequiredField)?
        .as_bool()
        .unwrap_or(false);
    let guest_can_join: bool = response
        .get("guest_can_join")
        .ok_or(errors::CustomError::MissingRequiredField)?
        .as_bool()
        .unwrap_or(false);

    Ok(Room {
        room_id: response_room_id,
        world_readable: world_readable,
        guest_can_join: guest_can_join,
        num_joined_members: response_num_joined_members,
        name: response
            .get("name")
            .and_then(|n| n.as_str())
            .and_then(|s| Some(s.to_string())),
        topic: response
            .get("topic")
            .and_then(|n| n.as_str())
            .and_then(|s| Some(s.to_string())),
        avatar_url: response
            .get("avatar_url")
            .and_then(|n| n.as_str())
            .and_then(|s| Some(s.to_string())),
        encryption: response
            .get("encryption")
            .and_then(|n| n.as_str())
            .and_then(|s| Some(s.to_string())),
    })
}

fn convert_to_formatted(markdown: String) -> String {
    markdown
}

pub async fn send_message(
    auth_state: &mut AuthState,
    roomid: String,
    message: Message,
) -> Result<(), BoxError> {
    let mut content = json!({
        "body": message.body,
        "msgtype": "m.text",
    });

    if message.replying_to.is_some() {
        content["m.relates.to"] = json!({
            "m.in_reply_to": json!({
                "event_id": message.replying_to.unwrap().to_string()
            })
        });
    }

    if message.formatted.is_some() {
        content["format"] = Value::String(message.formatted.unwrap().to_string());
        content["format_body"] = Value::String("org.matrix.custom.html".to_string());
    }

    send_event(
        auth_state,
        roomid,
        "m.room.message".to_string(),
        &content.to_string(),
    )
    .await
}

pub async fn send_event(
    auth_state: &mut AuthState,
    roomid: String,
    event_type: String,
    body: &str,
) -> Result<(), BoxError> {
    let transaction_id = Uuid::new_v4().simple().to_string();

    auth_state
        .auth_put(
            &format!("v3/rooms/{}/send/{}/{}", roomid, event_type, transaction_id).to_string(),
            body,
        )
        .await?;

    Ok(())
}

// dir is direction can be 'f' or 'b'
pub async fn get_messages(
    auth_state: AuthState,
    roomid: String,
    dir: &str,
    limit: u64,
    from: Option<String>,
) -> Result<Value, BoxError> {
    let mut url = format!(
        "v3/rooms/{}/messages?dir={}&limit={}",
        roomid,
        dir,
        limit.to_string()
    );

    if from.is_some() {
        // add the from param if exists
        url = format!("{}&from={}", url.to_owned(), &from.unwrap());
    }

    let response = auth_state.auth_get(&url).await?;

    Ok(response)
}

// pub async fn get_events(auth_state: &mut AuthState, roomid: String, ) -> Result<(), BoxError> {
//     auth_state.auth_get(format!("v3/rooms/{}/event/{}", roomid, ));

//     Ok(())
// }

// this is bare bones see
// 8.2 Creation for more additions
pub async fn create_room(
    auth_state: AuthState,
    name: &str,
    invites: Vec<&str>,
    preset: &str,
) -> Result<(), BoxError> {
    let body = json!({
        // "creation_content": {
        //     "m.federate": false
        // },
        "preset": preset,
        "name": name,
        "invite": invites,
    });

    let response = auth_state
        .auth_post("v3/createRoom", &body.to_string())
        .await?;

    Ok(())
}

#[derive(Debug)]
pub struct Message {
    body: Option<String>,
    replying_to: Option<String>,
    formatted: Option<String>,
}

impl Message {
    pub fn new(
        body: Option<String>,
        replying_to: Option<String>,
        formatted: Option<String>,
    ) -> Self {
        Message {
            body,
            replying_to,
            formatted,
        }
    }
}

#[derive(Debug)]
pub struct Room {
    name: Option<String>,
    room_id: String,
    topic: Option<String>,
    num_joined_members: u64,
    avatar_url: Option<String>,
    world_readable: bool,
    encryption: Option<String>,
    guest_can_join: bool,
}
