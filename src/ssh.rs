use crate::ui::{Message, theme::TerminalColors};
use async_trait::async_trait;
use iced::{Color, futures::channel::mpsc};
use russh::{ChannelId, client::{self, Session}, keys::key};

pub type SshChannel = russh::Channel<russh::client::Msg>;

pub struct MyHandler {
    pub sender: mpsc::Sender<Message>,
    // On garde en mémoire l'état du style
    pub current_color: Color,
    pub is_bold: bool,
    pub colors_config: TerminalColors,
}

#[async_trait]
impl client::Handler for MyHandler {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _key: &key::PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }

    /*  async fn data(&mut self, _id: russh::ChannelId, data: &[u8], _session: &mut client::Session) -> Result<(), Self::Error> {
        let raw_text = String::from_utf8_lossy(data);
        let clean_text = strip_ansi_codes(&raw_text);
        let _ = self.sender.try_send(Message::SshData(clean_text));
        //let _ = self.sender.try_send(Message::SshData(raw_text.to_string()));
        Ok(())


    }*/

async fn data(&mut self, _id: ChannelId, data: &[u8], _session: &mut Session) -> Result<(), Self::Error> {
    // On envoie les données brutes à l'UI
    // L'UI devra décider comment les interpréter (soit texte brut, soit codes ANSI)
    let _ = self.sender.try_send(Message::SshData(data.to_vec()));
    Ok(())
}
}

fn get_ansi_color(code: u8, colors: &TerminalColors) -> Color {
    match code {
        30 => Color::BLACK,
        31 => Color::from_rgb(0.8, 0.2, 0.2), // Rouge
        32 => Color::from_rgb(0.2, 0.8, 0.2), // Vert
        33 => Color::from_rgb(0.8, 0.8, 0.2), // Jaune
        34 => Color::from_rgb(0.2, 0.2, 0.8), // Bleu
        35 => Color::from_rgb(0.8, 0.2, 0.8), // Magenta
        36 => Color::from_rgb(0.2, 0.8, 0.8), // Cyan
        37 => colors.text,                    // Blanc/Texte par défaut
        90..=97 => colors.text,               // Couleurs brillantes (simplifiées ici)
        _ => colors.text,                     // Reset ou inconnu
    }
}


pub fn strip_ansi_codes(input: &str) -> String {
    let mut output = String::new();
    let mut iter = input.chars().peekable();
    // les caractères d'échappement ANSI commencent par \x1b[
    while let Some(c) = iter.next() {
        if c == '\x1b' {
            if let Some('[') = iter.peek() {
                iter.next();
                while let Some(&next_c) = iter.peek() {
                    iter.next();
                    if next_c.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }
        output.push(c);
    }
    output
}
