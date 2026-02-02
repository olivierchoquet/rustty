mod ssh;
mod ui;

use iced::{keyboard, window};
use ui::{Message, MyApp};

pub fn main() -> iced::Result {
    // 1. On configure le daemon
    iced::daemon("RustTy", MyApp::update, MyApp::view)
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

            // 2. Événements clavier propres (remplace on_key_press)
            let keyboard_events = iced::event::listen().map(|event| {
                match event {
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        text, // <-- C'est ce champ qui contient le caractère AZERTY (ex: ".")
                        ..
                    }) => {
                        // 1. GESTION DU TEXTE (Chiffres, Ponctuation, Lettres)
                        // Si 'text' existe, c'est ce que l'utilisateur veut écrire.
                        if let Some(t) = text {
                            if !modifiers.control() {
                                // On envoie le texte brut (sauf l'espace qu'on gère en touche nommée)
                                if t != " " {
                                    return Message::SshData(t.as_bytes().to_vec());
                                }
                            }
                        }

                        // 2. GESTION DES TOUCHES SPÉCIALES ET CTRL
                        match key {
                            iced::keyboard::Key::Named(named) => {
                                let bytes = match named {
                                    iced::keyboard::key::Named::Enter => Some(b"\r".to_vec()),
                                    iced::keyboard::key::Named::Backspace => Some(b"\x7f".to_vec()),
                                    iced::keyboard::key::Named::Space => Some(b" ".to_vec()),
                                    iced::keyboard::key::Named::Tab => Some(b"\t".to_vec()),
                                    iced::keyboard::key::Named::Escape => Some(b"\x1b".to_vec()),
                                    iced::keyboard::key::Named::ArrowUp => Some(b"\x1b[A".to_vec()),
                                    iced::keyboard::key::Named::ArrowDown => {
                                        Some(b"\x1b[B".to_vec())
                                    }
                                    iced::keyboard::key::Named::ArrowRight => {
                                        Some(b"\x1b[C".to_vec())
                                    }
                                    iced::keyboard::key::Named::ArrowLeft => {
                                        Some(b"\x1b[D".to_vec())
                                    }
                                    _ => None,
                                };
                                if let Some(b) = bytes {
                                    return Message::SshData(b);
                                }
                            }
                            // Raccourcis CTRL
                            iced::keyboard::Key::Character(c) if modifiers.control() => {
                                let bytes = c.as_bytes();
                                if !bytes.is_empty() {
                                    return Message::SshData(vec![bytes[0] & 0x1f]);
                                }
                            }
                            _ => {}
                        }
                        Message::DoNothing
                    }
                    _ => Message::DoNothing,
                }
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
