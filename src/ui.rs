// iced::futures::SinkExt est nécessaire pour l'envoi asynchrone de messages
use iced::futures::SinkExt;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::text_input;
use iced::{Element, Task, window};
use russh::{Pty, client};
use uuid::Uuid;
// Importation des types pour la gestion de la concurrence
use std::sync::Arc;
// Importation de Mutex asynchrone de tokio
use tokio::sync::Mutex;
use crate::messages::{ConfigMessage, LoginMessage, Message, ProfileMessage, SshMessage};
// Mes modules internes
use crate::ssh::{MyHandler, SshChannel};
use crate::ui::constants::*;
use crate::ui::theme::ThemeChoice;
//use crate::ui::views::login;
use vt100;

pub mod terminal;
pub mod theme;
pub mod views;
pub mod constants;
pub mod components {
    pub mod forms;
    pub mod sidebar;
    pub mod table;
    pub mod actions;
    pub mod brand;
}

// Identifiant unique pour le widget scrollable du terminal
pub const SCROLLABLE_ID: &str = "terminal_scroll";
// Nombre maximum de lignes à conserver dans le terminal (tampon)
// usize est le type pour les tailles et indices non signés qui s'adapte à l'architecture (32 ou 64 bits)
const MAX_TERMINAL_LINES: usize = 1000;

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
    pub history: Vec<String>,                           // Liste des commandes passées
    pub history_index: Option<usize>,                   // Position actuelle dans l'historique
    pub focus_index: usize,                             // 0 = IP, 1 = PORT, 2 = USER, 3 = PASS
    // pub theme_choice: ThemeChoice,
    // Gestion des profils
    pub current_profile: Profile, // Le "brouillon" lié aux inputs
    pub selected_profile_id: Option<uuid::Uuid>, // L'ID du profil qu'on est en train d'éditer
    pub profiles: Vec<Profile>,
    //pub password: String,         // On le garde à part (sécurité)
    pub search_query: String, // Pour le filtrage global
    // Catégorie de parammètres en cours d'édition
    pub active_section: EditSection,
    pub focused_id: &'static str, // Pour gérer le focus des TextInput (IP, Port, User, Pass)
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
            parser: vt100::Parser::new(24, 80, MAX_TERMINAL_LINES),
            login_window_id: Some(login_id),
            terminal_window_id: None,
            active_channel: None,
            history: Vec::new(),
            history_index: None,
            focus_index: 0,
            profiles: loaded_profiles,
            selected_profile_id: None,
            // profil "brouillon" vide au départ
            current_profile: Profile::default(),
            search_query: "".into(),
            active_section: EditSection::General,
            focused_id: ID_PROFILE, // On commence par le champ profil
    }
}

    fn perform_ssh_connection(&self) -> Task<Message> {
        //println!("Tentative de connexion à {}...", self.current_profile.ip);
        let profile = self.current_profile.ip.clone();
        let user = self.current_profile.username.clone();
        let pass = self.password.clone();
        let port = self.current_profile.port.parse::<u16>().unwrap_or(22);

        Task::stream(iced::stream::channel(100, move |mut output| async move {
            let config = Arc::new(client::Config::default());
            let handler = MyHandler {
                sender: output.clone(),
            };

            if let Ok(mut handle) = client::connect(config, (profile.as_str(), port), handler).await
            {
                if handle
                    .authenticate_password(user, pass)
                    .await
                    .unwrap_or(false)
                {
                    println!("Authentification réussie !");
                    let _ = output
                        .send(Message::Ssh(SshMessage::Connected(Ok(Arc::new(Mutex::new(handle))))))
                        .await;
                } else {
                    println!("Échec de l'authentification.");
                    let _ = output
                        .send(Message::Ssh(SshMessage::Connected(Err("Échec d'authentification".into()))))
                        .await;
                }
            } else {
                let _ = output
                    .send(Message::Ssh(SshMessage::Connected(Err("Serveur introuvable".into()))))
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


        // Dans ton code de connexion (pas dans le handler)
        let manual_modes: Vec<(Pty, u32)> = vec![
            (Pty::ICRNL, 1), // Convertir Carriage Return en Line Feed (Indispensable pour Enter)
            (Pty::ONLCR, 1), // Convertir Line Feed en CR-LF (Indispensable pour l'affichage)
        ];

        let shell_task = Task::perform(
            async move {
                // 1. On ouvre la session et on libère le lock IMMÉDIATEMENT
                let mut ch = {
                    let mut h_lock = handle.lock().await;
                    h_lock.channel_open_session().await.ok()?
                }; // Le verrou h_lock est relâché ici !

                // 2. Maintenant on peut configurer le canal sans bloquer le reste du client
                // On remplace les let _ par des vérifications réelles
                if let Err(e) = ch
                    .request_pty(true, "xterm-256color", 80, 24, 0, 0, &manual_modes)
                    .await
                {
                    eprintln!("Erreur PTY: {:?}", e);
                    return None;
                }

                // 2. Ouverture Shell
                ch.request_shell(true).await.ok()?;

                // 3. LE TRUC MAGIQUE : On envoie un saut de ligne tout de suite
                // pour forcer le Shell à afficher le prompt initial.

                let initial_data: &[u8] = &[13];
                ch.data(initial_data).await.ok();

                println!("SYSTÈME: Shell interactif ouvert avec succès.");
                Some(Arc::new(Mutex::new(ch)))
            },
            |ch|ch.map(|channel| Message::Ssh(SshMessage::SetChannel(channel)))
       .unwrap_or(Message::DoNothing),
        );

        Task::batch(vec![
            win_task.discard(),
            Task::done(Message::Ssh(SshMessage::TerminalWindowOpened(id))),
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
            Message::Login(LoginMessage::Submit) => self.perform_ssh_connection(),
            Message::Ssh(SshMessage::Connected(Ok(handle))) => self.open_terminal(handle),
            Message::Ssh(SshMessage::Connected(Err(e))) => {
                let error_msg = format!("\r\nErreur de connexion: {}\r\n", e);

                // On "trompe" le parser en lui faisant croire que le serveur
                // a envoyé ce texte. On utilise \r\n pour être sûr d'aller à la ligne.
                self.parser.process(error_msg.as_bytes());

                Task::none()
            }

            // Délégations aux modules
            Message::Login(LoginMessage::InputIP(_))
            | Message::Login(LoginMessage::InputPort(_))
            | Message::Login(LoginMessage::InputUsername(_))
            | Message::Login(LoginMessage::InputPass(_))
            | Message::Profile(ProfileMessage::Save)
            | Message::Profile(ProfileMessage::Delete)
            | Message::Profile(ProfileMessage::New)
            | Message::Profile(ProfileMessage::InputName(_))
            | Message::Profile(ProfileMessage::InputGroup(_))
            | Message::Profile(ProfileMessage::SearchChanged(_))
            | Message::Profile(ProfileMessage::Selected(_))
            | Message::Config(ConfigMessage::SectionChanged(_))
          //  | Message::Config(ConfigMessage::ThemeChanged(_)) => login::update(self, message),
          | Message::Config(ConfigMessage::ThemeChanged(_)) => views::update(self, message),
           // | Message::TabPressed => login::update(self, message),
            Message::QuitRequested => {
                std::process::exit(0);
            }

            Message::Ssh(SshMessage::DataReceived(_))
            | Message::Ssh(SshMessage::SendData(_))
            //| Message::HistoryPrev
            //| Message::HistoryNext
            | Message::Ssh(SshMessage::SetChannel(_))
             => terminal::update(self, message),
            //| Message::KeyPressed(_, _) => terminal::update(self, message),
            //| Message::KeyboardEvent(event) => terminal::update(self, Message::KeyboardEvent(event)),
            //Message::Config(ConfigMessage::ThemeChanged((_))) => terminal::update(self, message),

            // Gestion globale (Fenêtres)
            Message::Ssh(SshMessage::TerminalWindowOpened(id)) => {
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
            
            Message::Event(event) => {
    // On extrait l'événement clavier une seule fois pour tout le bloc
    if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key,
        modifiers,
        text,
        ..
    }) = event
    {
        // --- CAS 1 : MODE LOGIN (Navigation manuelle) ---
        if self.active_channel.is_none() {
            if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab) {
                // Détermination du prochain ID
                let next_id = match self.focused_id {
                    ID_PROFILE => ID_GROUP, // On reste sur le champ du profil pour éviter de perdre les données déjà saisies
                    ID_GROUP   => ID_IP,   // Idem pour le groupe
                    ID_IP   => ID_PORT,
                    ID_PORT => ID_USER,
                    ID_USER => ID_PASS,
                    _       => ID_PROFILE,
                };

                println!("TAB détecté au login. Focus : {} -> {}", self.focused_id, next_id);
                
                self.focused_id = next_id;
                // On retourne immédiatement la tâche de focus
                return text_input::focus(text_input::Id::new(next_id));
            }
            // Si on est au login mais que ce n'est pas un TAB, on ne fait rien
            return Task::none();
        }

        // --- CAS 2 : MODE CONNECTÉ (SSH) ---
        // On arrive ici seulement si active_channel.is_some()
        if let Some(channel_arc) = &self.active_channel {
            let bytes = self.map_event_to_bytes(
                key.clone(),
                modifiers,
                text.map(|t| t.to_string()),
            );

            if let Some(b) = bytes {
                let arc = channel_arc.clone();
                return Task::perform(
                    async move {
                        let mut ch = arc.lock().await;
                        let _ = ch.data(&b[..]).await;
                    },
                    |_| Message::DoNothing,
                );
            }
        }
    }
    
    Task::none()
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
            //login::view(self)
            crate::ui::views::render(self)
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

    fn map_event_to_bytes(&self,key: Key, mods: Modifiers, text: Option<String>) -> Option<Vec<u8>> {
        // Priorité 1 : Les raccourcis CTRL
        if mods.control() {
            if let Key::Character(ref c) = key {
                let b = c.as_bytes();
                if !b.is_empty() { return Some(vec![b[0] & 0x1f]); }
            }
        }

        // Priorité 2 : Le texte normal (Lettres, chiffres, symboles)
        if let Some(t) = text {
            let c = t.chars().next()?;
            if !c.is_control() && c != ' ' {
                return Some(t.as_bytes().to_vec());
            }
        }

        // Priorité 3 : Touches spéciales
        match key {
            Key::Named(Named::Enter) => Some(b"\r\n".to_vec()),
            Key::Named(Named::Tab) => Some(b"\t".to_vec()),
            Key::Named(Named::Backspace) => Some(b"\x7f".to_vec()),
            Key::Named(Named::Space) => Some(b" ".to_vec()),
            Key::Named(Named::ArrowUp) => Some(b"\x1b[A".to_vec()),
            Key::Named(Named::ArrowDown) => Some(b"\x1b[B".to_vec()),
            Key::Named(Named::ArrowRight) => Some(b"\x1b[C".to_vec()),
            Key::Named(Named::ArrowLeft) => Some(b"\x1b[D".to_vec()),
            _ => None,
        }
    }
}
