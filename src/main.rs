mod auth;
mod utils;
mod errors;
mod events;
mod content;

use std::slice::SplitInclusive;

use events::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = auth::AuthState::login_password("http://localhost:8008", "username", "password").await?;
    let rooms = get_rooms(&mut client).await?;
    let message = Message::new(Some("hello Matrix world!!".to_string()), None, None);


    // send_message(&mut client, rooms.get(0).unwrap().to_string(), message).await?;
    
    let mut event_sync = EventState::new();
    event_sync.sync(&mut client, None).await?;
    println!("{}", serde_json::to_string_pretty(&event_sync.next_batch.clone().unwrap()).unwrap());
    // // printl
    // for i in 1..10{
    //     let message = Message::new(Some(i.to_string()), None, None);
    //     send_message(&mut client, rooms.get(0).unwrap().to_string(), message).await?;
    // }
    // event_sync.sync(&mut client, Some(event_sync.rooms.clone().unwrap().get("join").unwrap().get(rooms.get(0).unwrap()).unwrap().get("timeline").unwrap().get("prev_batch").unwrap().as_str().unwrap().to_string())).await?;
    println!("{}", serde_json::to_string_pretty(&event_sync.rooms.clone().unwrap()).unwrap()); // .join .timeline .prevbatch

    let since_token = event_sync.rooms.clone().unwrap().get("join").unwrap().get(rooms.get(0).unwrap()).unwrap().get("timeline").unwrap().get("prev_batch").unwrap().as_str().unwrap().to_string();

    // println!("{}", serde_json::to_string_pretty(&get_messages(&mut client, rooms.get(0).unwrap().to_string(), "b", 3, Some(since_token.clone())).await.unwrap()).unwrap().to_string());

    println!("{}", since_token);

    println!("{:#?}", (&get_messages(&mut client, rooms.get(0).unwrap().to_string(), "b", 3, Some(since_token)).await));


    Ok(())
}  