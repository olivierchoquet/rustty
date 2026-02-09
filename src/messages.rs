use std::sync::Arc;
use iced::{window, Event};
use crate::{models::EditSection, ssh::{MyHandler, SshChannel}, ui::theme::ThemeChoice};
// Importation de Mutex asynchrone de tokio
use tokio::sync::Mutex;

#[derive(Clone, Debug)] // On simplifie le Debug pour l'instant
pub enum Message {
    // --- Système & Fenêtres ---
    Event(Event),
    //KeyboardEvent(iced::keyboard::Event),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    QuitRequested,
    DoNothing,

    // --- Sous-domaines (Découpage par thématique) ---
    Login(LoginMessage),     // Tout ce qui touche aux champs de saisie
    Ssh(SshMessage),         // Tout ce qui touche au réseau/terminal
    Profile(ProfileMessage), // Tout ce qui touche à la base de données de profils
    Config(ConfigMessage),   // Thèmes, sections, réglages
}

#[derive(Clone, Debug)]
pub enum LoginMessage {
    InputIP(String),
    InputPort(String),
    InputUsername(String),
    InputPass(String),
    Submit, // Ancien ButtonConnection
}

#[derive(Clone)]
pub enum SshMessage {
    Connected(Result<Arc<Mutex<russh::client::Handle<MyHandler>>>, String>),
    SetChannel(Arc<Mutex<SshChannel>>),
    DataReceived(Vec<u8>),  // Ancien SshData
    SendData(Vec<u8>),      // Ancien SendSshRaw ou RawKey
    TerminalWindowOpened(window::Id),
}

#[derive(Clone, Debug)]
pub enum ProfileMessage {
    Selected(uuid::Uuid),
    Save,
    Delete,
    New,
    InputName(String),
    InputGroup(String),
    SearchChanged(String),
}

#[derive(Clone, Debug)]
pub enum ConfigMessage {
    SectionChanged(EditSection),
    ThemeChanged(ThemeChoice),
}

// necessary for debugging SshMessage::Connected without printing the entire SSH handle
impl std::fmt::Debug for SshMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshMessage::Connected(Ok(_)) => f.debug_tuple("Connected").field(&"Ok(SSH_HANDLE)").finish(),
            SshMessage::Connected(Err(e)) => f.debug_tuple("Connected").field(&format!("Err({})", e)).finish(),
            // Pour les autres variantes simples, on peut utiliser le nom
            _ => f.write_str("OtherSshMessage"), 
        }
    }
}