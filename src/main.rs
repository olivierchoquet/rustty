mod ssh;
mod ui;

use iced::{keyboard, window};
use ui::{Message, MyApp};

pub fn main() -> iced::Result {
    // 1. Configuration du daemon Iced
    iced::daemon("RustTy", MyApp::update, MyApp::view)
        .subscription(|_| {
            // Gestion des événements de fenêtre (Ouverture/Fermeture)
            let window_events = window::events().map(|(id, event)| {
                match event {
                    window::Event::Opened { .. } => Message::WindowOpened(id),
                    window::Event::CloseRequested | window::Event::Closed => {
                        Message::WindowClosed(id)
                    }
                    _ => Message::DoNothing,
                }
            });

            // Gestion des événements clavier
            let keyboard_events = iced::event::listen().map(|event| {
                match event {
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        text,
                        ..
                    }) => {
                        // --- 1. GESTION DES CARACTÈRES ALPHANUMÉRIQUES ---
                        if let Some(t) = text {
                            // On n'envoie le texte que si CTRL n'est pas enfoncé 
                            // (pour éviter d'envoyer "c" quand on veut faire Ctrl+C)
                            if !modifiers.control() {
                                if let Some(c) = t.chars().next() {
                                    // On filtre les touches de contrôle et l'espace (gérés plus bas)
                                    if !c.is_control() && c != ' ' {
                                        // On utilise SendSshRaw : l'UI ne fait plus d'écho local, 
                                        // elle attend que le serveur renvoie le caractère.
                                        return Message::SendSshRaw(t.as_bytes().to_vec());
                                    }
                                }
                            }
                        }

                        // --- 2. GESTION DES TOUCHES SPÉCIALES ---
                        match key {
                            iced::keyboard::Key::Named(named) => {
                                let bytes = match named {
                                    // CORRECTION : On envoie \r\n pour valider la commande côté Shell
                                    iced::keyboard::key::Named::Enter => Some(b"\r\n".to_vec()),
                                    
                                    // Touches de navigation et édition
                                    iced::keyboard::key::Named::Backspace => Some(b"\x7f".to_vec()),
                                    iced::keyboard::key::Named::Space => Some(b" ".to_vec()),
                                    iced::keyboard::key::Named::Tab => Some(b"\t".to_vec()),
                                    iced::keyboard::key::Named::Escape => Some(b"\x1b".to_vec()),
                                    
                                    // Codes ANSI pour les flèches
                                    iced::keyboard::key::Named::ArrowUp => Some(b"\x1b[A".to_vec()),
                                    iced::keyboard::key::Named::ArrowDown => Some(b"\x1b[B".to_vec()),
                                    iced::keyboard::key::Named::ArrowRight => Some(b"\x1b[C".to_vec()),
                                    iced::keyboard::key::Named::ArrowLeft => Some(b"\x1b[D".to_vec()),
                                    _ => None,
                                };
                                
                                if let Some(b) = bytes {
                                    return Message::SendSshRaw(b);
                                }
                            }

                            // --- 3. GESTION DES RACCOURCIS CTRL (ex: Ctrl+C) ---
                            iced::keyboard::Key::Character(c) if modifiers.control() => {
                                let bytes = c.as_bytes();
                                if !bytes.is_empty() {
                                    // Application du masque binaire pour les codes de contrôle
                                    return Message::SendSshRaw(vec![bytes[0] & 0x1f]);
                                }
                            }
                            _ => {}
                        }
                        Message::DoNothing
                    }
                    _ => Message::DoNothing,
                }
            });

            // Fusion des abonnements
            iced::Subscription::batch(vec![window_events, keyboard_events])
        })
        .run_with(|| {
            // Initialisation de la fenêtre principale (Login)
            let (id, task) = window::open(window::Settings {
                size: iced::Size::new(950.0, 800.0),
                ..Default::default()
            });

            (MyApp::new(id), task.discard())
        })
}