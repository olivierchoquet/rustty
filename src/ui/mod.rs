// iced::futures::SinkExt est nécessaire pour l'envoi asynchrone de messages
use iced::futures::SinkExt;
use iced::widget::{Text, text_input};
use iced::{Color, Element, Task, window};
use russh::{Pty, client};
use uuid::Uuid;
// Importation des types pour la gestion de la concurrence
use std::sync::Arc;
// Importation de Mutex asynchrone de tokio
use tokio::sync::Mutex;
// Mes modules internes
use crate::ssh::{MyHandler, SshChannel};
use crate::ui::theme::{TerminalColors, ThemeChoice};
use vt100;

pub mod components;
pub mod terminal;
pub mod theme;
pub mod views;

// Identifiant unique pour le widget scrollable du terminal
pub const SCROLLABLE_ID: &str = "terminal_scroll";
// Nombre maximum de lignes à conserver dans le terminal (tampon)
// usize est le type pour les tailles et indices non signés qui s'adapte à l'architecture (32 ou 64 bits)
const MAX_TERMINAL_LINES: usize = 1000;

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
    //InputTerminal(String),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    SshData(Vec<u8>),
    DoNothing,
    WindowClosed(window::Id),
    HistoryPrev,
    HistoryNext,
    TabPressed,
    WindowOpened(window::Id),
    ThemeSelected(ThemeChoice),
    ProfileSelected(uuid::Uuid),
    SaveProfile,
    DeleteProfile,
    NewProfile,
    InputNewProfileName(String),
    InputNewProfileGroup(String),
    SearchChanged(String),
    SectionChanged(EditSection),
    ThemeChanged(ThemeChoice),
    QuitRequested,
    RawKey(Vec<u8>)
}

// Implémentation de Debug pour Message pour faciliter le débogage
impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SshMsg")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub ip: String,
    pub port: String,
    pub username: String,
    pub group: String,
    pub theme: ThemeChoice,
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.group.to_uppercase(), self.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditSection {
    General,
    Auth,
    Network,
    Advanced,
    Themes,
}

pub struct MyApp {
    pub password: String,
    //pub logs: Vec<TextSegment>, // Contient tout le texte affiché dans le terminal
    // Une liste de lignes. Chaque ligne contient ses segments colorés.
    // pub terminal_lines: Vec<Vec<TextSegment>>,
    pub parser: vt100::Parser,
    pub login_window_id: Option<window::Id>,
    pub terminal_window_id: Option<window::Id>,
    pub active_channel: Option<Arc<Mutex<SshChannel>>>, // La session SSH active
    pub terminal_input: String, // Ce que l'utilisateur tape dans le terminal
    pub history: Vec<String>,   // Liste des commandes passées
    pub history_index: Option<usize>, // Position actuelle dans l'historique
    pub focus_index: usize,     // 0 = IP, 1 = PORT, 2 = USER, 3 = PASS
    // pub theme_choice: ThemeChoice,
    // Gestion des profils
    pub current_profile: Profile, // Le "brouillon" lié aux inputs
    pub selected_profile_id: Option<uuid::Uuid>, // L'ID du profil qu'on est en train d'éditer
    pub profiles: Vec<Profile>,
    //pub password: String,         // On le garde à part (sécurité)
    pub search_query: String, // Pour le filtrage global
    // Catégorie de parammètres en cours d'édition
    pub active_section: EditSection,
}

impl MyApp {
    pub fn new(login_id: window::Id) -> Self {
        let loaded_profiles = Self::load_profiles(); // <-- On charge ici
        println!(
            "DEBUG: {} profils chargés au démarrage",
            loaded_profiles.len()
        );
        Self {
            password: "".into(),
            //logs: String::from("Prêt...\n"),
            // On initialise un terminal de 24 lignes et 80 colonnes
            parser: vt100::Parser::new(24, 80, 0),
            login_window_id: Some(login_id),
            terminal_window_id: None,
            active_channel: None,
            terminal_input: String::new(),
            history: Vec::new(),
            history_index: None,
            focus_index: 0,
            //theme_choice: ThemeChoice::Slate,
            // chargement des profils sauvegardées
            profiles: loaded_profiles,
            selected_profile_id: None,
            // profil "brouillon" vide au départ
            current_profile: Profile::default(),
            search_query: "".into(),
            active_section: EditSection::General,
        }
    }

    fn perform_ssh_connection(&self) -> Task<Message> {
        //println!("Tentative de connexion à {}...", self.current_profile.ip);
        let Profile = self.current_profile.ip.clone();
        let user = self.current_profile.username.clone();
        let pass = self.password.clone();
        let port = self.current_profile.port.parse::<u16>().unwrap_or(22);

        Task::stream(iced::stream::channel(100, move |mut output| async move {
            let config = Arc::new(client::Config::default());
            let handler = MyHandler {
                sender: output.clone(),
                current_color: iced::Color::WHITE,
                is_bold: false,
                colors_config: TerminalColors::default(),
            };

            if let Ok(mut handle) = client::connect(config, (Profile.as_str(), port), handler).await
            {
                if handle
                    .authenticate_password(user, pass)
                    .await
                    .unwrap_or(false)
                {
                    println!("Authentification réussie !");
                    let _ = output
                        .send(Message::SshConnected(Ok(Arc::new(Mutex::new(handle)))))
                        .await;
                } else {
                    println!("Échec de l'authentification.");
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
        println!("Ouverture de la fenêtre terminal...");
        let (id, win_task) = window::open(window::Settings {
            size: iced::Size::new(950.0, 650.0),
            ..Default::default()
        });

        // On crée le vecteur de configuration attendu par request_pty
        // ONLCR (code 71) permet de transformer les \n en \r\n
        let terminal_modes = vec![(Pty::ONLCR, 1)];

        let shell_task = Task::perform(
            async move {
                let h_lock = handle.lock().await;
                if let Ok(ch) = h_lock.channel_open_session().await {
                    let _ = ch
                        .request_pty(true, "xterm-256color", 80, 24, 0, 0, &terminal_modes)
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
            Message::ButtonConnection => self.perform_ssh_connection(),
            Message::SshConnected(Ok(handle)) => self.open_terminal(handle),
            Message::SshConnected(Err(e)) => {
                let error_msg = format!("\r\nErreur de connexion: {}\r\n", e);

                // On "trompe" le parser en lui faisant croire que le serveur
                // a envoyé ce texte. On utilise \r\n pour être sûr d'aller à la ligne.
                self.parser.process(error_msg.as_bytes());

                Task::none()
            }

            // Délégations aux modules
            Message::InputIP(_)
            | Message::InputPort(_)
            | Message::InputUsername(_)
            | Message::InputPass(_)
            | Message::SaveProfile
            | Message::DeleteProfile
            | Message::NewProfile
            | Message::InputNewProfileName(_)
            | Message::InputNewProfileGroup(_)
            | Message::SearchChanged(_)
            | Message::ProfileSelected(_)
            | Message::SectionChanged(_)
            | Message::ThemeChanged(_)
            | Message::TabPressed => crate::ui::views::login::update(self, message),
            Message::QuitRequested => {
                std::process::exit(0);
            }

            Message::SshData(_)
            | Message::HistoryPrev
            | Message::HistoryNext
            | Message::SetChannel(_)
            | Message::KeyPressed(_, _) => terminal::update(self, message),
            | Message::ThemeSelected(_) => terminal::update(self, message),

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
            //login::view(self)
            crate::ui::views::login::render(self)
        }
    }

    // Sauvegarder la liste sur le disque
    pub fn save_profiles(&self) {
        // 1. Tentative de sérialisation
        match serde_json::to_string_pretty(&self.profiles) {
            Ok(json) => {
                // 2. Tentative d'écriture atomique (on écrit tout d'un coup)
                match std::fs::write("profiles.json", json) {
                    Ok(_) => {
                        println!("LOG: Sauvegarde réussie ({} profils).", self.profiles.len());
                    }
                    Err(e) => {
                        eprintln!(
                            "ERREUR CRITIQUE: Impossible d'écrire dans profiles.json: {}",
                            e
                        );
                        // Ici, on pourrait ajouter une notification UI pour l'utilisateur
                    }
                }
            }
            Err(e) => {
                eprintln!("ERREUR: Échec de la conversion en JSON: {}", e);
            }
        }
    }

    // Charger la liste au démarrage
    pub fn load_profiles() -> Vec<Profile> {
        let path = "profiles.json";

        // 1. On vérifie d'abord si le fichier existe pour éviter de traiter une erreur inutile
        if !std::path::Path::new(path).exists() {
            println!("INFO : Aucun fichier profiles.json trouvé. Démarrage à vide.");
            return Vec::new();
        }

        // 2. Tentative de lecture du fichier
        match std::fs::read_to_string(path) {
            Ok(data) => {
                // 3. Tentative de parsing JSON
                match serde_json::from_str::<Vec<Profile>>(&data) {
                    Ok(profiles) => {
                        println!("LOG : {} profil(s) chargé(s) avec succès.", profiles.len());
                        profiles
                    }
                    Err(e) => {
                        // C'est ici que l'erreur s'affichera si tes anciens profils n'ont pas d'ID
                        println!(
                            "ERREUR : Le fichier profiles.json est corrompu ou mal formé : {}",
                            e
                        );
                        println!(
                            "CONSEIL : Supprimez profiles.json pour qu'il soit recréé proprement."
                        );
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                println!(
                    "ERREUR : Impossible de lire le fichier profiles.json : {}",
                    e
                );
                Vec::new()
            }
        }
    }
}
