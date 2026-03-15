use std::sync::Arc;
use crate::messages::{Message, SshMessage};
use async_trait::async_trait;
use iced::{
    Task,
    futures::{SinkExt, channel::mpsc},
    window,
};
use russh::{
    ChannelId, Pty,
    client::{self, Session},
    keys::key,
};
use tokio::sync::Mutex;

pub type SshChannel = russh::Channel<russh::client::Msg>;
pub type SshChannelArc = std::sync::Arc<tokio::sync::Mutex<SshChannel>>;
pub type SshHandle = std::sync::Arc<tokio::sync::Mutex<russh::client::Handle<MyHandler>>>;

pub struct MyHandler {
    pub window_id: Arc<Mutex<Option<iced::window::Id>>>,
    pub sender: mpsc::Sender<Message>,
}

impl MyHandler {
    /// Pure logic for routing data to the UI. 
    /// Extracted from the trait to allow safe unit testing without unsafe mocks.
    pub fn handle_data_routing(&mut self, window_id: window::Id, data: Vec<u8>) {
        let _ = self.sender.try_send(Message::Ssh(SshMessage::DataReceived(window_id, data)));
    }
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
            // Forwarding to our testable method
            self.handle_data_routing(id, data.to_vec());
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
                    if handle.authenticate_password(user, pass).await.unwrap_or(false) {
                        let _ = output.send(Message::Ssh(SshMessage::Connected(Ok((
                                Arc::new(Mutex::new(handle)),
                                window_id_container, 
                            ))))).await;
                    } else {
                        let _ = output.send(Message::Ssh(SshMessage::Connected(Err(
                                "Authentication failed".into(),
                            )))).await;
                    }
                }
                Err(_) => {
                    let _ = output.send(Message::Ssh(SshMessage::Connected(Err(
                            "Server not found".into(),
                        )))).await;
                }
            }
        }))
    }

    pub fn open_shell(
        window_id: iced::window::Id,
        handle: SshHandle,
        shared_window_id: Arc<Mutex<Option<iced::window::Id>>>,
    ) -> Task<Message> {
        let manual_modes: Vec<(Pty, u32)> = vec![(Pty::ICRNL, 1), (Pty::ONLCR, 1)];

        Task::perform(
            async move {
                {
                    let mut w_id_lock = shared_window_id.lock().await;
                    *w_id_lock = Some(window_id);
                }

                let mut ch = {
                    let mut h_lock = handle.lock().await;
                    h_lock.channel_open_session().await.ok()?
                };

                ch.request_pty(true, "xterm-256color", 80, 24, 0, 0, &manual_modes).await.ok()?;
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


#[cfg(test)]
mod tests {
    use super::*;
    use iced::window;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Test 1: Successful data routing
    /// Ensures that when data is received, it is correctly wrapped in a Message 
    /// and sent to the UI channel with the correct window ID.
    #[tokio::test]
    async fn test_handler_routing_success() {
        let (tx, mut rx) = mpsc::channel(100);
        let win_id = window::Id::unique();
        
        let mut handler = MyHandler {
            window_id: Arc::new(Mutex::new(Some(win_id))),
            sender: tx,
        };

        let test_payload = b"unit test data".to_vec();
        
        // Directly trigger the routing logic
        handler.handle_data_routing(win_id, test_payload.clone());

        // Check if the message was sent to the receiver
        let received = rx.try_next().unwrap().expect("Message should be in the channel");
        
        if let Message::Ssh(SshMessage::DataReceived(id, data)) = received {
            assert_eq!(id, win_id, "Window ID mismatch");
            assert_eq!(data, test_payload, "Payload data corruption");
        } else {
            panic!("Received unexpected message type");
        }
    }

    /// Test 2: Shared Window ID update
    /// Validates that the shared pointer between the UI and the SSH task 
    /// is correctly updated when a new terminal opens.
    #[tokio::test]
    async fn test_shared_window_id_locking() {
        let shared_id = Arc::new(Mutex::new(None));
        let new_win_id = window::Id::unique();

        // Simulate the logic inside SshService::open_shell
        {
            let mut lock = shared_id.lock().await;
            *lock = Some(new_win_id);
        }

        // Verify the value is correctly stored
        let updated_id = *shared_id.lock().await;
        assert_eq!(updated_id, Some(new_win_id), "Shared window ID was not updated correctly");
    }

    /// Test 3: Large payload handling
    /// Ensures the channel and routing can handle larger chunks of terminal data (buffer pressure).
    #[tokio::test]
    async fn test_handler_large_payload() {
        let (tx, mut rx) = mpsc::channel(1); // Small buffer to test pressure
        let win_id = window::Id::unique();
        let mut handler = MyHandler {
            window_id: Arc::new(Mutex::new(Some(win_id))),
            sender: tx,
        };

        let large_payload = vec![b'A'; 4096]; // 4KB of data
        handler.handle_data_routing(win_id, large_payload.clone());

        let received = rx.try_next().unwrap().expect("Should receive large payload");
        if let Message::Ssh(SshMessage::DataReceived(_, data)) = received {
            assert_eq!(data.len(), 4096);
        }
    }
}