use core::time;
use std::error::Error;
use std::time::Duration;
use std::{collections::HashMap, hash::Hash, sync::Arc};

use serde_json::Value;
use tokio::sync::Mutex;

use crate::errors::{BoxError, CustomError};
use crate::{
    auth::{self, AuthState},
    content::{self, Cache, CacheRoom},
    errors::CustomError::MissingRequiredField,
    events::{EventState, get_rooms},
};

pub async fn unauth_get(server_address: &str, path: &str) -> Result<serde_json::Value, BoxError> {
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

pub fn async_sync(
    rooms_mutex: Arc<Mutex<Vec<((String, String), String)>>>,
    cache_mutex: Arc<Mutex<HashMap<String, CacheRoom>>>,
    error: Arc<Mutex<Vec<String>>>,
    auth_state: crate::auth::AuthState,
) {
    let cache_clone = cache_mutex.clone();
    let rooms_clone = rooms_mutex.clone();
    let error_clone = error.clone();

    tokio::spawn(async move {
        let mut caches = cache_clone.lock().await;

        let mut event_state = EventState::new();

        let resp = event_state.sync(auth_state.clone(), None).await;

        match resp {
            Ok(_) => {}
            Err(e) => {
                error_clone.lock().await.push(e.to_string());
                state_updater_helper(rooms_mutex, cache_mutex, error, auth_state, event_state);
                return;
            }
        }

        let cache_rooms = get_cache_rooms(&event_state, rooms_clone, error_clone)
            .await
            .ok_or(CustomError::InvalidJson)
            .and_then(|inner| inner.map_err(|_| CustomError::InvalidJson));

        match cache_rooms {
            Ok(cache_rooms_map) => {
                *caches = cache_rooms_map;
            }
            Err(e) => {
                error.clone().lock().await.push(e.to_string());
            }
        }
        state_updater_helper(rooms_mutex, cache_mutex, error, auth_state, event_state);
    });
}

pub fn state_updater_helper(
    rooms_mutex: Arc<Mutex<Vec<((String, String), String)>>>,
    cache_mutex: Arc<Mutex<HashMap<String, CacheRoom>>>,
    error_mutex: Arc<Mutex<Vec<String>>>,
    auth_state: crate::auth::AuthState,
    event_state: EventState,
) {
    let rooms = rooms_mutex.clone();
    let caches = cache_mutex.clone();
    let errors = error_mutex.clone();
    let auth = auth_state.clone();
    let mut event_state = event_state.clone();

    tokio::spawn(async move {
        loop {
            // This line long polls the server for new updates, returning when there is one
            let resp = event_state
                .sync(auth_state.clone(), event_state.next_batch.clone())
                .await;

            match resp {
                Ok(_) => {}
                Err(e) => {
                    errors.lock().await.push(e.to_string());
                    continue;
                }
            }

            // This method updates the room list automatically
            let update_caches: Option<Result<HashMap<String, CacheRoom>, Box<dyn Error + Send + Sync>>>
                = get_cache_rooms(&event_state, rooms.clone(), errors.clone()).await;

            if let Some(Ok(updated_cache)) = update_caches {
                let mut caches_lock = caches.lock().await;

                for (roomid, update_room) in updated_cache {
                    let possible_prexisting_room = caches_lock.get(&roomid);

                    match caches_lock.entry(roomid) {
                        // Room exists so we need to edit the prexisting one
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            entry.get_mut().append(update_room);
                        }
                        // Room does not exist so we just make a new entry
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert(update_room);
                        }
                    }
                }
            } else {
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
        }
    });
}

async fn get_cache_rooms(
    event_state: &EventState,
    rooms_mutex: Arc<Mutex<Vec<((String, String), String)>>>,
    error_clone: Arc<Mutex<Vec<String>>>,
) -> Option<Result<HashMap<String, CacheRoom>, Box<dyn Error + Send + Sync>>> {
    // Rooms
    let rooms_value: content::Rooms = match event_state.rooms.clone() {
        Some(v) => match serde_json::from_value(v) {
            Ok(r) => r,
            Err(e) => {
                error_clone
                    .lock()
                    .await
                    .push("Invalid sync data sent from server".to_string());
                return None;
            }
        },
        None => {
            error_clone
                .lock()
                .await
                .push("Invalid sync data sent from server".to_string());
            return None;
        }
    };

    for (roomid, room) in &rooms_value.join {
        // Deal with rooms list
        let event = room
            .timeline
            .events
            .last()
            .and_then(|v| content::parse_event(v.clone()).ok())
            .unwrap_or_default();
        let subtext = event.summary();

        // Format the room's names
        let mut room_name = roomid.chars().take(10).collect::<String>();

        // Check state first for a room name
        for e in &room.state.events.iter().rev().collect::<Vec<&Value>>() {
            if e.get("type").unwrap_or_default() == "m.room.name" {
                room_name = e
                    .get("content")
                    .unwrap_or_default()
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
            }
        }

        // then check timeline and overwrite state if found a new one 
        for e in room.timeline.events.iter().rev().collect::<Vec<&Value>>() {
            if e.get("type").unwrap_or_default() == "m.room.name" {
                room_name = e
                    .get("content")
                    .unwrap_or_default()
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
            }
            break;
        }
         

        let mut rooms_lock = rooms_mutex.lock().await;

        rooms_lock.retain(|(_, id)| id != roomid); // remove the old room from the list (if it exists)
        rooms_lock.push(((room_name, subtext), roomid.to_string())); // add the new one
    }

    let cache_rooms = Cache::from_rooms(rooms_value).await;
    Some(cache_rooms)
}

pub fn async_update_rooms(
    mutex: Arc<Mutex<Vec<((String, String), String)>>>,
    auth_state: crate::auth::AuthState,
) {
    let mutex_clone = mutex.clone();

    tokio::spawn(async move {
        let mut rooms = mutex_clone.lock().await;

        let rooms = get_rooms(auth_state).await;
        if rooms.is_err() {
            return;
        }

        for room in rooms.unwrap() {}
    });
}

pub fn async_fetch_room_events(
    cache_mutex: Arc<Mutex<Option<Cache>>>,
    roomid: String,
    auth_state: crate::auth::AuthState,
) {
    let mutex_clone = cache_mutex.clone();

    tokio::spawn(async move {});
}

pub fn logout() {
    tokio::spawn(async move {
        AuthState::delete_from_disk().await.unwrap();
    });
}
