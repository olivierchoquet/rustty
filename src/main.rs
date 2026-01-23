mod ssh;
mod ui;
mod config;

use iced::{keyboard, window};
use ui::{Message, MyApp};

pub fn main() -> iced::Result {
    // 1. On configure le daemon
    iced::daemon("Rust-PuTTY Pro", MyApp::update, MyApp::view)
        // 2. On définit les abonnements (clavier, fenêtres, etc.)
        .subscription(|_| {
            let window_events = window::events().map(|(id, event)| {
                println!("Événement window reçu: {:?}", event);
                match event {
                    window::Event::Opened { .. } => Message::WindowOpened(id),
                    // On gère les deux cas : la demande de fermeture ET la fermeture effective
                    window::Event::CloseRequested | window::Event::Closed => {
                        println!("Fermeture détectée via {:?}", event);
                        Message::WindowClosed(id)
                    }
                    _ => Message::DoNothing,
                }
            });

            let keyboard_events = keyboard::on_key_press(|key, _| match key {
                keyboard::Key::Named(keyboard::key::Named::Tab) => Some(Message::TabPressed),
                keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::HistoryPrev),
                keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::HistoryNext),
                _ => None,
            });

            // On fusionne les deux flux d'événements
            iced::Subscription::batch(vec![window_events, keyboard_events])
        })
        // 3. On définit comment l'application démarre
        .run_with(|| {
            let (id, task) = window::open(window::Settings {
                size: iced::Size::new(950.0, 800.0),
                ..Default::default()
            });

            
            (MyApp::new(id), task.discard())
        })
}
