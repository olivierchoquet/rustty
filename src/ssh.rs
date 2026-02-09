use std::sync::Arc;

use crate::messages::{Message, SshMessage};
use async_trait::async_trait;
use iced::{Color, Task, futures::{SinkExt, channel::mpsc}, window};
use russh::{
    ChannelId, Pty, client::{self, Session}, keys::key
};
use tokio::sync::Mutex;

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

// Dans src/ssh.rs

pub struct SshService;

impl SshService {
    /// Crée la tâche de connexion
    pub fn connect(profile_ip: String, port: u16, user: String, pass: String) -> Task<Message> {
        Task::stream(iced::stream::channel(100, move |mut output| async move {
            let config = Arc::new(client::Config::default());
            let handler = MyHandler { sender: output.clone() };

            match client::connect(config, (profile_ip.as_str(), port), handler).await {
                Ok(mut handle) => {
                    if handle.authenticate_password(user, pass).await.unwrap_or(false) {
                        let _ = output.send(Message::Ssh(SshMessage::Connected(Ok(Arc::new(Mutex::new(handle)))))).await;
                    } else {
                        let _ = output.send(Message::Ssh(SshMessage::Connected(Err("Échec d'authentification".into())))).await;
                    }
                }
                Err(_) => {
                    let _ = output.send(Message::Ssh(SshMessage::Connected(Err("Serveur introuvable".into())))).await;
                }
            }
        }))
    }

    /// Crée la tâche d'ouverture du shell PTY
    pub fn open_shell(handle: Arc<Mutex<client::Handle<MyHandler>>>) -> Task<Message> {
        let (id, win_task) = window::open(window::Settings {
            size: iced::Size::new(950.0, 650.0),
            ..Default::default()
        });

        let manual_modes: Vec<(Pty, u32)> = vec![
            (Pty::ICRNL, 1),
            (Pty::ONLCR, 1),
        ];

        let shell_task = Task::perform(
            async move {
                let mut ch = {
                    let mut h_lock = handle.lock().await;
                    h_lock.channel_open_session().await.ok()?
                }; 

                ch.request_pty(true, "xterm-256color", 80, 24, 0, 0, &manual_modes).await.ok()?;
                ch.request_shell(true).await.ok()?;
                
                
                Some(Arc::new(Mutex::new(ch)))
            },
            |ch| ch.map(|channel| Message::Ssh(SshMessage::SetChannel(channel)))
                   .unwrap_or(Message::DoNothing),
        );

        Task::batch(vec![
            win_task.discard(),
            Task::done(Message::Ssh(SshMessage::TerminalWindowOpened(id))),
            shell_task,
        ])
    }
}