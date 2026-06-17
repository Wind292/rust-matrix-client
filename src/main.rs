use std::io;

use crate::events::{create_room, get_rooms};

mod auth;
mod content;
mod errors;
mod events;
mod gui;
mod utils;

#[tokio::main]
async fn main() -> io::Result<()> {

    let mut terminal = ratatui::init();
    let result = gui::App::default().run(&mut terminal).await;
    ratatui::restore();
    result
}