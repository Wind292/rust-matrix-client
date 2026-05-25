use core::time;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{content::Cache, events::{EventState, get_rooms}};

pub async fn unauth_get(
    server_address: &str,
    path: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let resp = reqwest::get(server_address.to_string() + "/_matrix/client/" + path)
        .await?
        .json::<serde_json::Value>()
        .await?;
    Ok(resp)
}

pub async fn unauth_post(
    server_address: &str,
    path: &str,
    data: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let data_clone = data.to_owned();
    let resp = client
        .post(server_address.to_string() + "/_matrix/client/" + path)
        .body(data_clone)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    Ok(resp)
}

pub fn async_load_room(mutex: Arc<Mutex<Cache>>, auth_state: crate::auth::AuthState) {
    let mutex_clone = mutex.clone();

    tokio::spawn(async move {
        let mut cache = mutex_clone.lock().await;
        cache.update_before(auth_state).await;
    });
}

pub fn async_sync(mutex: Arc<Mutex<Vec<((String, String), String)>>>, error: Arc<Mutex<Vec<String>>>, auth_state: crate::auth::AuthState) {
    let rooms_clone = mutex.clone();
    let error_clone = error.clone();

    tokio::spawn(async move {
        let mut rooms_mutex = rooms_clone.lock().await;

        let mut event_state = EventState::new();
        let resp = event_state.sync(auth_state, None).await;

        match resp {
            Ok(_) => {},
            Err(e) => {
                error_clone.lock().await.push(e.to_string());
                return;
            },
        }

        let rooms_value = event_state.rooms.clone();
        let rooms_value = rooms_value.and_then(|f| f.as_object().and_then(|f| f.get("join").and_then(|f| f.as_object())).cloned());

        if rooms_value.is_none() { error_clone.lock().await.push("Invalid room data sent from server".to_string()) }

        for (roomid, value) in rooms_value.unwrap() {
            let state = value.get("state");
            let timeline = value.get("timeline");

            if state.is_none() || timeline.is_none() { error_clone.lock().await.push("Invalid room data sent from server".to_string()) }

            // Name, subtext, id


        }

        
    });
}

pub fn async_update_rooms(mutex: Arc<Mutex<Vec<((String, String), String)>>>, auth_state: crate::auth::AuthState) {
    let mutex_clone = mutex.clone();

    tokio::spawn(async move {
        let mut rooms = mutex_clone.lock().await;

        let rooms = get_rooms(auth_state).await;
        if rooms.is_err() { return }

        for room in rooms.unwrap() {

        }

    });
}




