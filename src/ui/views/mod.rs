pub mod login;
pub mod terminal;

use iced::{Element, window};
use crate::ui::{MyApp, Message};

pub fn main_view(app: &MyApp, window_id: window::Id) -> Element<'_, Message> {
    if Some(window_id) == app.terminal_window_id {
        terminal::render(app)
    } else {
        login::render(app)
    }
}