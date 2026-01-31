use crate::ui::{Message, theme::TerminalColors};
use async_trait::async_trait;
use iced::{Color, futures::channel::mpsc};
use russh::{client, keys::key};

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

    async fn data(
        &mut self,
        _id: russh::ChannelId,
        data: &[u8],
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        let raw_text = String::from_utf8_lossy(data);

        // On parse en utilisant l'état actuel du handler
        let (segments, new_color, new_bold) = parse_ansi_with_state(
            &raw_text,
            self.current_color,
            self.is_bold,
            &self.colors_config,
        );

        // On met à jour l'état pour le prochain paquet de données
        self.current_color = new_color;
        self.is_bold = new_bold;

        // On envoie les segments stylisés au lieu d'une String brute
        let _ = self.sender.try_send(Message::SshData(segments));

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TextSegment {
    pub content: String,
    pub color: Color,
    pub is_bold: bool,
}

impl TextSegment {
    pub fn new(content: String, color: Color) -> Self {
        Self {
            content,
            color,
            is_bold: false,
        }
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

pub fn parse_ansi_with_state(
    input: &str, 
    mut current_color: Color, 
    mut current_bold: bool,
    config: &TerminalColors
) -> (Vec<TextSegment>, Color, bool) {
    let mut segments = Vec::new();
    let parts = input.split("\x1b[");

    for (i, part) in parts.enumerate() {
        if i == 0 {
            if !part.is_empty() {
                segments.push(TextSegment { content: part.to_string(), color: current_color, is_bold: current_bold });
            }
            continue;
        }

        if let Some(m_pos) = part.find('m') {
            let code_str = &part[..m_pos];
            let rest = &part[m_pos + 1..];

            for code in code_str.split(';') {
                match code.parse::<u8>().unwrap_or(0) {
                    0 => { current_color = config.text; current_bold = false; },
                    1 => current_bold = true,
                    c @ 30..=37 => current_color = get_ansi_color(c, config),
                    _ => {}, // On ignore les autres codes (mouvements, etc.)
                }
            }

            if !rest.is_empty() {
                segments.push(TextSegment { content: rest.to_string(), color: current_color, is_bold: current_bold });
            }
        }
    }
    (segments, current_color, current_bold)
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
