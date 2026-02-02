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

            let keyboard_events = iced::event::listen().map(|event| {
                match event {
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        text,
                        ..
                    }) => {
                        // 1. GESTION DU TEXTE (AZERTY : Chiffres, Ponctuation, Lettres)
                        if let Some(t) = text {
                            // On vérifie que ce n'est pas une commande CTRL
                            if !modifiers.control() {
                                // On ne prend que les caractères "imprimables"
                                // On filtre l'espace (géré plus bas) et les contrôles (Enter, Tab, etc.)
                                if let Some(c) = t.chars().next() {
                                    if !c.is_control() && c != ' ' {
                                        println!("DEBUG NORMAL CHAR: {:?}", t.as_bytes().to_vec());
                                        return Message::SshData(t.as_bytes().to_vec());
                                    }
                                }
                            }
                            
                        }

                        // 2. GESTION DES TOUCHES SPÉCIALES (Commandes précises)
                        match key {
                            iced::keyboard::Key::Named(named) => {
                                let bytes = match named {
                                    //iced::keyboard::key::Named::Enter => Some(b"\r".to_vec()),
                                    iced::keyboard::key::Named::Enter => {
                                        let bytes = b"\r".to_vec();
                                        // Ce message s'affichera dans ton terminal de lancement (cargo run)
                                        println!(
                                            "DEBUG: Touche ENTER détectée, envoi de: {:?}",
                                            bytes
                                        );
                                        Some(bytes)
                                    }
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
                                    println!("DEBUG SPECIAL CHAR: {:?}", b);
                                    return Message::SshData(b);
                                }
                            }
                            // Raccourcis CTRL (ex: Ctrl+C)
                            iced::keyboard::Key::Character(c) if modifiers.control() => {
                                let bytes = c.as_bytes();
                                if !bytes.is_empty() {
                                    // Masque binaire pour transformer une lettre en code CTRL
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
