use crate::messages::{Message, SshMessage};
use async_trait::async_trait;
use iced::{Color, futures::channel::mpsc};
use russh::{
    ChannelId,
    client::{self, Session},
    keys::key,
};

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

    async fn data(
        &mut self,
        _id: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // On envoie les données brutes à l'UI
        // L'UI devra décider comment les interpréter (soit texte brut, soit codes ANSI)
        let _ = self.sender.try_send(Message::Ssh(SshMessage::DataReceived(data.to_vec())));
        Ok(())
    }
}
