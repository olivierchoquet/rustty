// iced::futures::SinkExt est nécessaire pour l'envoi asynchrone de messages
use iced::futures::SinkExt;
use iced::widget::text_input;
use iced::{Element, Task, window};
use russh::client;
// Importation des types pour la gestion de la concurrence
use std::sync::Arc;
// Importation de Mutex asynchrone de tokio
use tokio::sync::Mutex;
// Mes modules internes
use crate::config::AppConfig;
use crate::ssh::{MyHandler, SshChannel};

pub mod login;
pub mod terminal;
pub mod theme;

// Identifiant unique pour le widget scrollable du terminal
pub const SCROLLABLE_ID: &str = "terminal_scroll";
// Nombre maximum de lignes à conserver dans le terminal (tampon)
// usize est le type pour les tailles et indices non signés qui s'adapte à l'architecture (32 ou 64 bits)
const MAX_TERMINAL_LINES: usize = 1000;

// --- DÉFINITION DES COULEURS PERSONNALISÉES POUR LA FENETRE TERMINAL---
//pub const COLOR_BG: iced::Color = iced::Color::from_rgb(0.05, 0.07, 0.1); // Bleu-nuit profond
//pub const COLOR_ACCENT: iced::Color = iced::Color::from_rgb(0.0, 0.5, 1.0); // Bleu Windows
//pub const COLOR_TEXT: iced::Color = iced::Color::from_rgb(0.9, 0.9, 0.9); // Blanc cassé
//pub const COLOR_PROMPT: iced::Color = iced::Color::from_rgb(0.2, 0.8, 0.4); // Vert émeraude

// Dans ui/mod.rs
//pub const COLOR_BG: iced::Color = iced::Color::from_rgb(0.11, 0.12, 0.15); // Gris très sombre
//pub const COLOR_SURFACE: iced::Color = iced::Color::from_rgb(0.16, 0.18, 0.22); // Gris moyen pour les inputs
//pub const COLOR_ACCENT: iced::Color = iced::Color::from_rgb(0.0, 0.47, 0.85); // Bleu Windows/VSCode
//pub const COLOR_SUCCESS: iced::Color = iced::Color::from_rgb(0.3, 0.8, 0.4); // Vert terminal
//pub const COLOR_TEXT: iced::Color = iced::Color::from_rgb(0.9, 0.9, 0.9);


// Identifiants pour le focus - touche tabulation
pub const ID_IP: &str = "ip_input";
pub const ID_PORT: &str = "port_input";
pub const ID_USER: &str = "user_input";
pub const ID_PASS: &str = "pass_input";

// --- DÉFINITION DES MESSAGES ET DE L'APPLICATION ---
#[derive(Clone)]
pub enum Message {
    InputIP(String),
    InputPort(String),
    InputUsername(String),
    InputPass(String),
    ButtonConnection,
    SshConnected(Result<Arc<Mutex<russh::client::Handle<MyHandler>>>, String>),
    TerminalWindowOpened(window::Id),
    SetChannel(Arc<Mutex<SshChannel>>),
    SshData(String),
    InputTerminal(String),
    SendCommand,
    DoNothing,
    WindowClosed(window::Id),
    HistoryPrev,
    HistoryNext,
    TabPressed,
    WindowOpened(window::Id),
}

// Implémentation de Debug pour Message pour faciliter le débogage
impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SshMsg")
    }
}

pub struct MyApp {
    pub ip: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub logs: String, // Contient tout le texte affiché dans le terminal
    pub login_window_id: Option<window::Id>,
    pub terminal_window_id: Option<window::Id>,
    pub active_channel: Option<Arc<Mutex<SshChannel>>>, // La session SSH active
    pub terminal_input: String, // Ce que l'utilisateur tape dans le terminal
    pub history: Vec<String>,   // Liste des commandes passées
    pub history_index: Option<usize>, // Position actuelle dans l'historique
    pub focus_index: usize,     // 0 = IP, 1 = PORT, 2 = USER, 3 = PASS
}

impl MyApp {
    pub fn new(login_id: window::Id) -> Self {
        //Récupération de la configuration sauvegardée
        let config = AppConfig::load();
        Self {
            ip: config.last_ip.into(),
            port: config.last_port.into(),
            username: config.last_username.into(),
            password: "".into(),
            logs: String::from("Prêt...\n"),
            login_window_id: Some(login_id),
            terminal_window_id: None,
            active_channel: None,
            terminal_input: String::new(),
            history: Vec::new(),
            history_index: None,
            focus_index: 0,
        }
    }

    fn perform_ssh_connection(&self) -> Task<Message> {
        let host = self.ip.clone();
        let user = self.username.clone();
        let pass = self.password.clone();
        let port = self.port.parse::<u16>().unwrap_or(22);

        Task::stream(iced::stream::channel(100, move |mut output| async move {
            let config = Arc::new(client::Config::default());
            let handler = MyHandler {
                sender: output.clone(),
            };

            if let Ok(mut handle) = client::connect(config, (host.as_str(), port), handler).await {
                if handle
                    .authenticate_password(user, pass)
                    .await
                    .unwrap_or(false)
                {
                    let _ = output
                        .send(Message::SshConnected(Ok(Arc::new(Mutex::new(handle)))))
                        .await;
                } else {
                    let _ = output
                        .send(Message::SshConnected(
                            Err("Échec d'authentification".into()),
                        ))
                        .await;
                }
            } else {
                let _ = output
                    .send(Message::SshConnected(Err("Serveur introuvable".into())))
                    .await;
            }
        }))
    }

    fn open_terminal(&self, handle: Arc<Mutex<client::Handle<MyHandler>>>) -> Task<Message> {
        let (id, win_task) = window::open(window::Settings {
            size: iced::Size::new(950.0, 650.0),
            ..Default::default()
        });

        let shell_task = Task::perform(
            async move {
                let h_lock = handle.lock().await;
                if let Ok(ch) = h_lock.channel_open_session().await {
                    let _ = ch
                        .request_pty(true, "xterm-256color", 100, 30, 0, 0, &[])
                        .await;
                    let _ = ch.request_shell(true).await;
                    return Some(Arc::new(Mutex::new(ch)));
                }
                None
            },
            |ch| ch.map(Message::SetChannel).unwrap_or(Message::DoNothing),
        );

        Task::batch(vec![
            win_task.discard(),
            Task::done(Message::TerminalWindowOpened(id)),
            shell_task,
        ])
    }

    fn handle_window_closed(&mut self, id: window::Id) -> Task<Message> {
        // 1. Si c'est le terminal, on ferme proprement le canal SSH
        if Some(id) == self.terminal_window_id {
            self.terminal_window_id = None;
            if let Some(ch_arc) = self.active_channel.take() {
                return Task::perform(
                    async move {
                        let ch = ch_arc.lock().await;
                        let _ = ch.close().await;
                    },
                    |_| Message::DoNothing,
                );
            }
        }

        // 2. Si c'est le login, on tue tout le processus
        if Some(id) == self.login_window_id {
            std::process::exit(0);
        }

        // 3. Sinon, on ferme juste la fenêtre demandée
        window::close(id)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Actions de haut niveau (isolées dans des méthodes)
            Message::ButtonConnection => {
                // On prépare la sauvegarde
                let config = AppConfig {
                    last_ip: self.ip.clone(),
                    last_username: self.username.clone(),
                    last_port: self.port.clone(),
                };
                config.save(); // On écrit sur le disque
                self.perform_ssh_connection()
            }
            Message::SshConnected(Ok(handle)) => self.open_terminal(handle),
            Message::SshConnected(Err(e)) => {
                self.logs.push_str(&format!("Erreur: {}\n", e));
                Task::none()
            }

            // Délégations aux modules
            Message::InputIP(_)
            | Message::InputPort(_)
            | Message::InputUsername(_)
            | Message::InputPass(_)
            | Message::TabPressed => login::update(self, message),

            Message::SshData(_)
            | Message::HistoryPrev
            | Message::HistoryNext
            | Message::SendCommand
            | Message::SetChannel(_)
            | Message::InputTerminal(_) => terminal::update(self, message),

            // Gestion globale (Fenêtres)
            Message::TerminalWindowOpened(id) => {
                self.terminal_window_id = Some(id);
                Task::none()
            }
            Message::WindowOpened(id) => {
                if Some(id) == self.login_window_id {
                    return text_input::focus(text_input::Id::new(ID_IP));
                }
                Task::none()
            }
            Message::WindowClosed(id) => self.handle_window_closed(id),

            _ => Task::none(),
        }
    }

    // --- LOGIQUE VISUELLE (VIEW) ---

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.terminal_window_id {
            // On délègue le dessin au module terminal
            terminal::view(self)
        } else {
            // On délègue le dessin au module login
            login::view(self)
        }
    }
}
