use std::io;

mod auth;
mod utils;
mod errors;
mod events;
mod content;
mod gui;

#[tokio::main]
async fn main() -> io::Result<()> {
    ratatui::run(|terminal| gui::App::default().run(terminal))
}