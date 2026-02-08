pub mod messages;
pub mod ssh;
pub mod ui;

use iced::{Task, widget::text_input, window};
use ui::MyApp;

use crate::{messages::Message, ui::constants::*};

pub fn main() -> iced::Result {
    // 1. Configuration du daemon Iced
    iced::daemon("RustTy", MyApp::update, MyApp::view)
        //En écrivant |_|,
        //tu disais à Rust : "Reçois cet argument, mais je m'en fiche, je ne vais pas l'appeler à l'intérieur de mon code".
        //si on veut l'utiliser donc |app|
        .subscription(|_| {
            // Gestion des événements de fenêtre (Ouverture/Fermeture)
            let window_events = window::events().map(|(id, event)| match event {
                window::Event::Opened { .. } => Message::WindowOpened(id),
                window::Event::CloseRequested | window::Event::Closed => Message::WindowClosed(id),
                _ => Message::DoNothing,
            });

            let events = iced::event::listen_with(|event, status, _id| {
                match status {
                    // Si un widget (comme un TextInput) a déjà utilisé l'événement,
                    // on ne fait rien pour ne pas interférer.
                    iced::event::Status::Captured => None,

                    // Si l'événement est libre (Ignored), on l'envoie à l'update
                    iced::event::Status::Ignored => Some(Message::Event(event)),
                }
            });

            // Fusion des abonnements
            iced::Subscription::batch(vec![window_events, events])
        })
        .run_with(|| {
            // Initialisation de la fenêtre principale (Login)
            let (id, task) = window::open(window::Settings {
                size: iced::Size::new(950.0, 850.0),
                ..Default::default()
            });

            (
                MyApp::new(id),
                Task::batch(vec![
                    task.discard(),
                    // C'est ici que l'on force le curseur sur le champ PROFILE_NAME au lancement
                    text_input::focus(text_input::Id::new(ID_PROFILE)),
                ]),
            )
        })
}
