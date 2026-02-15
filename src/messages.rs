use std::sync::Arc;
use iced::{window, Event};
use crate::{models::EditSection, ssh::{MyHandler, SshChannel, SshHandle}, ui::theme::ThemeChoice};
use tokio::sync::Mutex;

#[derive(Clone, Debug)] 
pub enum Message {
    // --- System & Windows ---
    Event(Event),
    //KeyboardEvent(iced::keyboard::Event),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    QuitRequested,
    DoNothing,

    // --- Sub-domain (Breakdown by topic) ---
    Login(LoginMessage),     // Everything related to input fields
    Ssh(SshMessage),         // Everything related to the network/terminal
    Profile(ProfileMessage), // Everything related to the profiles database
    Config(ConfigMessage),   // Themes, sections, settings
}

#[derive(Clone, Debug)]
pub enum LoginMessage {
    InputIP(String),
    InputPort(String),
    InputUsername(String),
    InputPass(String),
    Submit, 
}

#[derive(Clone)]
pub enum SshMessage {
    Connected(Result<(crate::ssh::SshHandle, Arc<Mutex<Option<iced::window::Id>>>), String>),
    SendData(Vec<u8>),   
    TerminalWindowOpened(window::Id, SshHandle, Arc<Mutex<Option<window::Id>>>),
    SetChannel(iced::window::Id, crate::ssh::SshChannelArc), 
    DataReceived(iced::window::Id, Vec<u8>),
    WindowFocused(iced::window::Id),
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
    TerminalCountChanged(usize),
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
            // For other variants, we can just print their names without the full content for brevity
            _ => f.write_str("OtherSshMessage"), 
        }
    }
}