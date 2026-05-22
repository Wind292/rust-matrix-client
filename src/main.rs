mod auth;
mod utils;
mod errors;
mod events;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = auth::AuthState::login_password("http://localhost:8008", "username", "password").await?;

    let mut event_state = events::EventState::new();


    println!("{:?}", event_state.sync(client).await?);

    // event_state.create_room(client, "my_room", vec!("@otheruser:my.local.server"), "private_chat").await?;


    Ok(())
}  