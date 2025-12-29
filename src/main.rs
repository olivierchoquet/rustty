use async_trait::async_trait;
use iced::futures::SinkExt;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{window, Element, Length, Task, Theme};
use russh::{client, keys::key};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn main() -> iced::Result {
    iced::daemon("Rust-PuTTY", MyApp::update, MyApp::view)
        .run_with(|| {
            // ÉTAPE 1 : Ouvrir la fenêtre de login au démarrage
            let (_, task) = window::open(window::Settings {
                size: iced::Size::new(800.0, 400.0),
                ..Default::default()
            });
            (MyApp::default(), task.discard())
        })
}

// --- LE HANDLER SSH ---
struct MyHandler {
    sender: iced::futures::channel::mpsc::Sender<Message>,
}

#[async_trait]
impl client::Handler for MyHandler {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _key: &key::PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn data(
        &mut self,
        _id: russh::ChannelId,
        data: &[u8],
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        let text = String::from_utf8_lossy(data).to_string();
        let _ = self.sender.try_send(Message::SshData(text));
        Ok(())
    }
}

// --- MESSAGES ---
#[derive(Clone)]
enum Message {
    InputIP(String),
    InputUsername(String),
    InputPass(String),
    InputPort(String),
    ButtonConnection,
    SshConnected(Result<Arc<Mutex<russh::client::Handle<MyHandler>>>, String>),
    SshData(String),
    InputTerminal(String),
    SendCommand,
    DoNothing,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::SshConnected(Ok(_)) => f.write_str("SshConnected(Ok)"),
            Message::SshConnected(Err(e)) => f.debug_tuple("SshConnected").field(e).finish(),
            Message::SshData(d) => f.debug_tuple("SshData").field(d).finish(),
            Message::InputTerminal(i) => f.debug_tuple("InputTerminal").field(i).finish(),
            Message::SendCommand => f.write_str("SendCommand"),
            _ => f.write_str("OtherMessage"),
        }
    }
}

// --- ÉTAT ---
struct MyApp {
    ip: String,
    username: String,
    password: String,
    port: u16,
    logs: String,
    is_connected: bool,
    ssh_handle: Option<Arc<Mutex<russh::client::Handle<MyHandler>>>>,
    terminal_input: String,
    terminal_window_id: Option<window::Id>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            ip: "courslinux.vinci.be".into(),
            username: "".into(),
            password: "".into(),
            port: 4980,
            logs: String::from("Attente de connexion...\n"),
            is_connected: false,
            ssh_handle: None,
            terminal_input: String::new(),
            terminal_window_id: None,
        }
    }
}

// --- LOGIQUE (UPDATE) ---
impl MyApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputIP(ip) => { self.ip = ip; Task::none() }
            Message::InputUsername(u) => { self.username = u; Task::none() }
            Message::InputPass(p) => { self.password = p; Task::none() }
            Message::InputPort(p) => { self.port = p.parse().unwrap_or(22); Task::none() }

            Message::ButtonConnection => {
                let host = self.ip.clone();
                let user = self.username.clone();
                let pass = self.password.clone();
                let port = self.port;
                self.logs.push_str("Connexion en cours...\n");

                Task::stream(iced::stream::channel(100, move |mut output| async move {
                    let config = Arc::new(client::Config::default());
                    let handler = MyHandler { sender: output.clone() };

                    match client::connect(config, (host.as_str(), port), handler).await {
                        Ok(mut handle) => {
                            if handle.authenticate_password(user, pass).await.unwrap_or(false) {
                                let shared_handle = Arc::new(Mutex::new(handle));
                                let _ = output.send(Message::SshConnected(Ok(shared_handle))).await;
                            } else {
                                let _ = output.send(Message::SshConnected(Err("Échec Auth".into()))).await;
                            }
                        }
                        Err(e) => { let _ = output.send(Message::SshConnected(Err(e.to_string()))).await; }
                    }
                }))
            }

            Message::SshConnected(Ok(shared_handle)) => {
                self.is_connected = true;
                self.ssh_handle = Some(shared_handle.clone());
                self.logs.push_str("Connecté ! Ouverture du terminal...\n");

                let h = shared_handle.clone();
                let shell_task = Task::perform(async move {
                    let mut handle = h.lock().await;
                    if let Ok(mut ch) = handle.channel_open_session().await {
                        let _ = ch.request_pty(true, "xterm-256color", 80, 24, 0, 0, &[]).await;
                        let _ = ch.request_shell(true).await;
                    }
                }, |_| Message::DoNothing);

                let (id, window_task) = window::open(window::Settings {
                    size: iced::Size::new(1024.0, 768.0),
                    ..Default::default()
                });
                self.terminal_window_id = Some(id);

                Task::batch(vec![shell_task, window_task.map(|_| Message::DoNothing)])
            }

            Message::SshConnected(Err(e)) => {
                self.logs.push_str(&format!("Erreur : {}\n", e));
                Task::none()
            }

            Message::SshData(data) => {
                self.logs.push_str(&data);
                Task::none()
            }

            Message::InputTerminal(input) => {
                self.terminal_input = input;
                Task::none()
            }

            Message::SendCommand => {
                if let Some(handle_arc) = &self.ssh_handle {
                    let payload = format!("{}\n", self.terminal_input);
                    self.terminal_input.clear();
                    let h = handle_arc.clone();

                    Task::perform(async move {
                        let mut handle = h.lock().await;
                        // Canal 0 par défaut pour le shell
                        if let Ok(mut ch) = handle.channel_open_session().await {
                            let _ = ch.data(payload.as_bytes()).await;
                        }
                    }, |_| Message::DoNothing)
                } else {
                    Task::none()
                }
            }
            Message::DoNothing => Task::none(),
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message, Theme> {
        if Some(window_id) == self.terminal_window_id {
            self.view_terminal()
        } else {
            self.view_login()
        }
    }

    fn view_login(&self) -> Element<'_, Message, Theme> {
        container(column![
            text("Rust-PuTTY Login").size(30),
            text_input("IP", &self.ip).on_input(Message::InputIP),
            text_input("User", &self.username).on_input(Message::InputUsername),
            text_input("Pass", &self.password).secure(true).on_input(Message::InputPass),
            button("Se connecter").on_press(Message::ButtonConnection),
            scrollable(text(&self.logs).size(12)).height(Length::Fill)
        ].spacing(15).padding(20)).into()
    }

    fn view_terminal(&self) -> Element<'_, Message, Theme> {
        container(column![
            scrollable(text(&self.logs).font(iced::Font::MONOSPACE).size(14)).height(Length::Fill),
            row![
                text_input("bash# ", &self.terminal_input)
                    .on_input(Message::InputTerminal)
                    .on_submit(Message::SendCommand),
                button("Envoyer").on_press(Message::SendCommand),
            ].spacing(10)
        ])
        .style(|_| container::Style {
            background: Some(iced::Color::BLACK.into()),
            text_color: Some(iced::Color::from_rgb(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        .padding(10).into()
    }
}