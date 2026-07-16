use core::time;
use std::{collections::HashMap, hash::Hash, sync::Arc};

use tokio::sync::Mutex;

use crate::{auth::{self, AuthState}, content::{self, Cache, CacheRoom}, errors::CustomError::MissingRequiredField, events::{EventState, get_rooms}};
use crate::errors::BoxError;


pub async fn unauth_get(
    server_address: &str,
    path: &str,
) -> Result<serde_json::Value, BoxError> {
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
) -> Result<serde_json::Value, BoxError> {
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

pub fn async_sync(rooms_mutex: Arc<Mutex<Vec<((String, String), String)>>>, cache_mutex: Arc<Mutex<HashMap<String, CacheRoom>>>, error: Arc<Mutex<Vec<String>>>, auth_state: crate::auth::AuthState) {
    let cache_clone = cache_mutex.clone();
    let rooms_clone = rooms_mutex.clone();
    let error_clone = error.clone();

    tokio::spawn(async move {
        let mut caches = cache_clone.lock().await;
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


        // Rooms 
        let rooms_value: content::Rooms = match event_state.rooms.clone() {
            Some(v) => match serde_json::from_value(v) {
                Ok(r) => r,
                Err(_) => {
                    error_clone.lock().await.push("Invalid sync data sent from server".to_string());
                    return;
                }
            },
            None => {
                error_clone.lock().await.push("Invalid sync data sent from server".to_string());
                return;
            }
        };

        for (roomid, room) in &rooms_value.join {

            // Deal with rooms list
            let event = room.timeline.events.last()
                .and_then(|v| content::parse_event(v.clone()).ok())
                .unwrap_or_default();
            let subtext = event.summary();


            // Format the room's names
            let mut room_name = roomid.chars().take(10).collect::<String>();
            for e in &room.state.events {
                if e.get("type").unwrap_or_default() == "m.room.name" {
                    room_name = e.get("content").unwrap_or_default().get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                }
            }

            rooms_mutex.push(((room_name, subtext), roomid.to_string()));
        }

        
        let cache_rooms  = Cache::from_rooms(rooms_value).await;
        
        match cache_rooms {
            Ok(cache_rooms_map) => {
                *caches = cache_rooms_map;
            },
            Err(_) => {
                // error.clone().lock().await.push("".to_string());
            },
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

pub fn async_fetch_room_events(cache_mutex: Arc<Mutex<Option<Cache>>>, roomid: String, auth_state: crate::auth::AuthState) {
    let mutex_clone = cache_mutex.clone();

    tokio::spawn(async move {


    });
}

pub fn logout() {
    tokio::spawn(async move {
        AuthState::delete_from_disk().await.unwrap();
    });
}
