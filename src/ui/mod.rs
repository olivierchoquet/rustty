/***** Contient la structure MyApp et la logique update  ******/

// iced::futures::SinkExt est nécessaire pour l'envoi asynchrone de messages
use iced::futures::SinkExt;
// Importation des widgets et types nécessaires d'iced
use iced::widget::{button, column, container, row, scrollable, text, text_input};
// Importation des types de base d'iced
use iced::{Alignment, Element, Length, Task, Theme, window};
use russh::client;
// Importation des types pour la gestion de la concurrence
use std::sync::Arc;
// Importation de Mutex asynchrone de tokio
use tokio::sync::Mutex;
// Importation des mon module
use crate::ssh::{MyHandler, SshChannel};

pub mod login;
pub mod terminal;

// Identifiant unique pour le widget scrollable du terminal
pub const SCROLLABLE_ID: &str = "terminal_scroll";
// Nombre maximum de lignes à conserver dans le terminal (tampon)
// usize est le type pour les tailles et indices non signés qui s'adapte à l'architecture (32 ou 64 bits)
const MAX_TERMINAL_LINES: usize = 1000;

// --- DÉFINITION DES COULEURS PERSONNALISÉES POUR LA FENETRE TERMINAL---
pub const COLOR_BG: iced::Color = iced::Color::from_rgb(0.05, 0.07, 0.1); // Bleu-nuit profond
pub const COLOR_ACCENT: iced::Color = iced::Color::from_rgb(0.0, 0.5, 1.0); // Bleu Windows
pub const COLOR_TEXT: iced::Color = iced::Color::from_rgb(0.9, 0.9, 0.9); // Blanc cassé
pub const COLOR_PROMPT: iced::Color = iced::Color::from_rgb(0.2, 0.8, 0.4); // Vert émeraude

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
    ClearLogs,
    DoNothing,
    WindowClosed(window::Id),
    HistoryPrev,
    HistoryNext,
    FocusNext(String),
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
        Self {
            ip: "".into(),
            port: "".into(),
            username: "".into(),
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

    // --- LOGIQUE DE MISE À JOUR (UPDATE) ---

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Récupération des entrées utilisateur
            Message::InputIP(ip) => {
                self.ip = ip;
                Task::none()
            }
            Message::InputPort(p) => {
                self.port = p;
                Task::none()
            }
            Message::InputUsername(u) => {
                self.username = u;
                Task::none()
            }
            Message::InputPass(p) => {
                self.password = p;
                Task::none()
            }

            Message::ButtonConnection => {
                // Rust il faut cloner les valeurs pour les déplacer dans la tâche asynchrone
                let host = self.ip.clone();
                let user = self.username.clone();
                let pass = self.password.clone();
                let port = self.port.parse::<u16>().unwrap_or(22);

                // On lance une tâche asynchrone pour la connexion
                Task::stream(iced::stream::channel(100, move |mut output| async move {
                    let config = Arc::new(client::Config::default());
                    let handler = MyHandler {
                        sender: output.clone(),
                    };

                    if let Ok(mut handle) =
                        client::connect(config, (host.as_str(), port), handler).await
                    {
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

            Message::SshConnected(Ok(handle)) => {
                // Ouverture de la deuxième fenêtre (Terminal)
                let (id, task) = window::open(window::Settings {
                    size: iced::Size::new(950.0, 650.0),
                    ..Default::default()
                });

                // On ouvre un canal de session SSH (Shell)
                let h = handle.clone();
                let open_ch = Task::perform(
                    async move {
                        let mut h_lock = h.lock().await;
                        if let Ok(mut ch) = h_lock.channel_open_session().await {
                            // On demande un PTY (Pseudo-Terminal) pour avoir un shell interactif
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
                // On combine les tâches pour ouvrir la fenêtre et le canal SSH
                Task::batch(vec![
                    task.discard(),
                    Task::done(Message::TerminalWindowOpened(id)),
                    open_ch,
                ])
            }

            // Stockage de l'ID de la fenêtre terminal
            Message::TerminalWindowOpened(id) => {
                self.terminal_window_id = Some(id);
                Task::none()
            }

            // Stockage du canal SSH actif
            Message::SetChannel(ch) => {
                self.active_channel = Some(ch);
                self.logs.push_str("--- SESSION ÉTABLIE ---\n");
                Task::none()
            }

            // Réception de données SSH à afficher dans le terminal
            Message::SshData(data) => {
                // 1. On ajoute les nouvelles données reçues du serveur
                self.logs.push_str(&data);

                // 2. On découpe le texte en lignes pour compter
                let lines: Vec<&str> = self.logs.lines().collect();

                // 3. Si on dépasse MAX_TERMINAL_LINES lignes...
                if lines.len() > MAX_TERMINAL_LINES {
                    // On ne garde que les MAX_TERMINAL_LINES dernières lignes
                    // On les rejoint avec un saut de ligne "\n"
                    self.logs = lines[lines.len() - MAX_TERMINAL_LINES..].join("\n");
                    // On rajoute un saut de ligne à la fin pour que la suite s'écrive dessous
                    self.logs.push('\n');
                }

                // 4. On demande à Iced de scroller tout en bas
                scrollable::snap_to(
                    scrollable::Id::new(SCROLLABLE_ID),
                    scrollable::RelativeOffset::END,
                )
            }

            // Envoi de la commande tapée par l'utilisateur
            Message::SendCommand => {
                // 1. On vérifie si l'input n'est pas vide
                if !self.terminal_input.is_empty() {
                    // 2. On gère l'historique
                    // On n'ajoute à l'historique que si c'est différent de la dernière commande (évite les doublons)
                    if self.history.last() != Some(&self.terminal_input) {
                        self.history.push(self.terminal_input.clone());
                    }
                    // On réinitialise l'index de navigation car on repart sur une nouvelle saisie
                    self.history_index = None;

                    // 3. Logique d'envoi SSH existante
                    if let Some(ch_arc) = &self.active_channel {
                        let cmd = format!("{}\n", self.terminal_input);
                        self.terminal_input.clear();
                        let ch_clone = ch_arc.clone();

                        return Task::perform(
                            async move {
                                let mut ch = ch_clone.lock().await;
                                let _ = ch.data(cmd.as_bytes()).await;
                            },
                            |_| Message::DoNothing,
                        );
                    }
                }
                Task::none()
            }

            // Vider l'écran du terminal
            Message::ClearLogs => {
                self.logs.clear();
                Task::none()
            }
            // Mise à jour de l'entrée terminal
            Message::InputTerminal(input) => {
                self.terminal_input = input;
                Task::none()
            }
            // Gestion des erreurs de connexion SSH
            Message::SshConnected(Err(e)) => {
                self.logs.push_str(&format!("Erreur: {}\n", e));
                Task::none()
            }
            // Gestion de la fermeture des fenêtres
            Message::WindowClosed(id) => {
                if Some(id) == self.terminal_window_id {
                    self.logs.push_str("Fermeture de la session SSH...\n");

                    if let Some(ch_arc) = self.active_channel.take() {
                        // On récupère le canal et on le ferme
                        return Task::perform(
                            async move {
                                let mut ch = ch_arc.lock().await;
                                // Envoi du signal de fermeture au serveur
                                let _ = ch.close().await;
                            },
                            |_| Message::DoNothing,
                        );
                    }
                    self.terminal_window_id = None;
                }
                // Arrêter l'application si c'est la fenêtre de login qui est fermée
                if Some(id) == self.login_window_id {
                    //return iced::exit(); // Commande Iced pour arrêter le daemon proprement

                    std::process::exit(0); // Tue le processus père et tous les threads immédiatement
                }
                //Task::none()
                window::close(id) // Cette ligne s'exécute si ce n'est pas le login
            }
            Message::WindowOpened(id) => {
                // Si c'est la fenêtre de login, on donne le focus
                if Some(id) == self.login_window_id {
                    return text_input::focus(text_input::Id::new(ID_IP));
                }
                Task::none()
            }
            Message::HistoryPrev => {
                if !self.history.is_empty() {
                    // 2. On calcule le nouvel index (on remonte dans le temps)
                    let new_index = match self.history_index {
                        None => self.history.len().checked_sub(1),
                        Some(idx) => idx.checked_sub(1),
                    };

                    if let Some(idx) = new_index {
                        self.history_index = Some(idx);
                        // 3. On remplace le texte de l'input par la commande historique
                        self.terminal_input = self.history[idx].clone();
                    }
                }
                Task::none()
            }
            Message::HistoryNext => {
                if let Some(idx) = self.history_index {
                    let next_idx = idx + 1;

                    if next_idx < self.history.len() {
                        // On descend à la commande suivante
                        self.history_index = Some(next_idx);
                        self.terminal_input = self.history[next_idx].clone();
                    } else {
                        // On est revenu au présent (plus de commandes après)
                        self.history_index = None;
                        self.terminal_input.clear();
                    }
                }
                Task::none()
            }
            Message::FocusNext(id) => {
                // Cette commande dit à Iced de mettre le curseur dans le champ spécifié
                text_input::focus(text_input::Id::new(id))
            }
            Message::TabPressed => {
                // On définit l'ordre de tabulation
                // IP -> Username -> Password -> (retour à IP)

                // Pour savoir quel champ focaliser, on peut soit stocker le focus actuel,
                // soit utiliser un petit "hack" simple : on fait circuler le focus.
                // Créons une variable dans MyApp pour suivre le focus : focus_index: usize

                self.focus_index = (self.focus_index + 1) % 4;

                let target_id = match self.focus_index {
                    0 => ID_IP,
                    1 => ID_PORT,
                    2 => ID_USER,
                    3 => ID_PASS,
                    _ => ID_IP,
                };

                text_input::focus(text_input::Id::new(target_id))
            }
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
