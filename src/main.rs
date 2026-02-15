pub mod messages;
pub mod ssh;
pub mod ui;
pub mod models;

use iced::{Task, widget::text_input, window};
use ui::MyApp;

use crate::{messages::Message, ui::constants::*};

pub fn main() -> iced::Result {
    // idec daemon to manage multiple windows and global events
    iced::daemon("RustTy", MyApp::update, MyApp::view)
        //By writing |_|,
        //you were telling Rust: “Receive this argument, but I don't care about it, I'm not going to call it inside my code.”
        //So if we want to use it, then |app|
        .subscription(|_| {
            let window_events = window::events().map(|(id, event)| match event {
                window::Event::Opened { .. } => Message::WindowOpened(id),
                window::Event::CloseRequested | window::Event::Closed => Message::WindowClosed(id),
                _ => Message::DoNothing,
            });

            let events = iced::event::listen_with(|event, status, _id| {
                match status {
                    // If a widget (like a TextInput) has already captured the event, we do nothing to avoid interference.
                    iced::event::Status::Captured => None,

                    // If the event is not captured, we forward it to the app for processing (like global shortcuts).
                    iced::event::Status::Ignored => Some(Message::Event(event)),
                }
            });

            iced::Subscription::batch(vec![window_events, events])
        })
        .run_with(|| {
            // Init the first window and get its ID and the task to open it
            let (id, task) = window::open(window::Settings {
                size: iced::Size::new(950.0, 900.0),
                ..Default::default()
            });

            (
                MyApp::new(id),
                Task::batch(vec![
                    task.discard(),
                    // Focus the TextInput PROFILE_NAME in the new window to allow immediate typing
                    text_input::focus(text_input::Id::new(ID_PROFILE)),
                ]),
            )
        })
}
