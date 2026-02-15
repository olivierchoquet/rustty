use std::sync::Arc;

use crate::messages::{Message, SshMessage};
use async_trait::async_trait;
use iced::{
    Color, Task,
    futures::{SinkExt, channel::mpsc},
    window,
};
use russh::{
    ChannelId, Pty,
    client::{self, Session},
    keys::key,
};
use tokio::sync::Mutex;

// Alias for the SSH channel type, simplifying references throughout the code
pub type SshChannel = russh::Channel<russh::client::Msg>;
// Using Arc and Mutex to allow shared ownership and mutable access across async tasks and threads
pub type SshChannelArc = std::sync::Arc<tokio::sync::Mutex<SshChannel>>;
// Alias for the SSH handle, which manages the connection and channels. Wrapped in Arc and Mutex for safe concurrent access.
pub type SshHandle = std::sync::Arc<tokio::sync::Mutex<russh::client::Handle<MyHandler>>>;

pub struct MyHandler {
    pub window_id: Arc<Mutex<Option<iced::window::Id>>>,
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
        let w_id = *self.window_id.lock().await;
        if let Some(id) = w_id {
            let _ = self
                .sender
                .try_send(Message::Ssh(SshMessage::DataReceived(id, data.to_vec())));
        }
        Ok(())
    }
}

pub struct SshService;

impl SshService {

    pub fn connect(profile_ip: String, port: u16, user: String, pass: String) -> Task<Message> {
        Task::stream(iced::stream::channel(100, move |mut output| async move {
            let config = Arc::new(client::Config::default()); 
            let window_id_container = Arc::new(Mutex::new(None));
            let handler = MyHandler {
                sender: output.clone(),
                window_id: window_id_container.clone(), 
            };

            match client::connect(config, (profile_ip.as_str(), port), handler).await {
                Ok(mut handle) => {
                    if handle
                        .authenticate_password(user, pass)
                        .await
                        .unwrap_or(false)
                    {
                        let _ = output
                            .send(Message::Ssh(SshMessage::Connected(Ok((
                                Arc::new(Mutex::new(handle)),
                                window_id_container, 
                            )))))
                            .await;
                    } else {
                        let _ = output
                            .send(Message::Ssh(SshMessage::Connected(Err(
                                "Échec d'authentification".into(),
                            ))))
                            .await;
                    }
                }
                Err(_) => {
                    let _ = output
                        .send(Message::Ssh(SshMessage::Connected(Err(
                            "Serveur introuvable".into(),
                        ))))
                        .await;
                }
            }
        }))
    }


    pub fn open_shell(
        window_id: iced::window::Id,
        handle: SshHandle,
        shared_window_id: Arc<Mutex<Option<iced::window::Id>>>, // <--- Ajoute ceci
    ) -> Task<Message> {
        let manual_modes: Vec<(Pty, u32)> = vec![(Pty::ICRNL, 1), (Pty::ONLCR, 1)];

        Task::perform(
            async move {
                // 1. update the shared window ID directly via the Arc! 
                // This ensures that when the SSH handler receives data, it knows which window to send it to.
                {
                    let mut w_id_lock = shared_window_id.lock().await;
                    *w_id_lock = Some(window_id);
                    println!("LOG: ID partagé mis à jour pour {:?}", window_id);
                }

                // 2. Now we can open the SSH session and channel as before, knowing that the handler has the correct window ID to work with.
                let mut ch = {
                    let mut h_lock = handle.lock().await;
                    h_lock.channel_open_session().await.ok()?
                };

                ch.request_pty(true, "xterm-256color", 80, 24, 0, 0, &manual_modes)
                    .await
                    .ok()?;
                ch.request_shell(true).await.ok()?;

                Some(Arc::new(Mutex::new(ch)))
            },
            move |ch| {
                ch.map(|channel| Message::Ssh(SshMessage::SetChannel(window_id, channel)))
                    .unwrap_or(Message::DoNothing)
            },
        )
    }
}
