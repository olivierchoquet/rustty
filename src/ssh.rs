use async_trait::async_trait;
use russh::{client, keys::key};
use iced::futures::channel::mpsc;
use crate::ui::Message;

pub type SshChannel = russh::Channel<russh::client::Msg>;

pub struct MyHandler {
    pub sender: mpsc::Sender<Message>,
}

#[async_trait]
impl client::Handler for MyHandler {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _key: &key::PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn data(&mut self, _id: russh::ChannelId, data: &[u8], _session: &mut client::Session) -> Result<(), Self::Error> {
        let raw_text = String::from_utf8_lossy(data);
        let clean_text = strip_ansi_codes(&raw_text);
        let _ = self.sender.try_send(Message::SshData(clean_text));
        //let _ = self.sender.try_send(Message::SshData(raw_text.to_string()));
        Ok(())
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
                    if next_c.is_ascii_alphabetic() { break; }
                }
                continue;
            }
        }
        output.push(c);
    }
    output
}