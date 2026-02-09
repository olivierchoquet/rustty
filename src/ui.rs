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
use crate::models::{EditSection, Profile};
// Mes modules internes
use crate::ssh::{MyHandler, SshChannel, SshService};
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
        let loaded_profiles = Profile::load_all();
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



    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Actions de haut niveau (isolées dans des méthodes)
            Message::Login(LoginMessage::Submit) => {
                SshService::connect(
                    self.current_profile.ip.clone(),
                self.current_profile.port.parse().unwrap_or(22),
                self.current_profile.username.clone(),
                self.password.clone())
            },
            Message::Ssh(SshMessage::Connected(Ok(handle))) => 
            {
                SshService::open_shell(handle)
            },
            Message::Ssh(SshMessage::Connected(Err(e))) => {
                let error_msg = format!("\r\nErreur de connexion: {}\r\n", e);

                // On "trompe" le parser en lui faisant croire que le serveur
                // a envoyé ce texte. On utilise \r\n pour être sûr d'aller à la ligne.
                self.parser.process(error_msg.as_bytes());

                Task::none()
            },
            Message::Login(msg) => self.handle_login_msg(msg),
            Message::Profile(msg)=> self.handle_profile_msg(msg),
            Message::Config(msg) => self.handle_config_msg(msg),
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

    // proxy method 
    // if save logic changes, only update this method without touching the rest of the codebase
    pub fn save_profiles(&self) {
        Profile::save_all(&self.profiles);
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


    fn handle_profile_msg(&mut self, msg: ProfileMessage) -> Task<Message> {
        match msg {
            ProfileMessage::Selected(id) => {
                if let Some(profile) = self.profiles.iter().find(|p| p.id == id) {
                    self.selected_profile_id = Some(id);
                    self.current_profile = profile.clone();
                }
            }
            ProfileMessage::InputName(name) => self.current_profile.name = name,
            ProfileMessage::InputGroup(group) => self.current_profile.group = group,
            ProfileMessage::SearchChanged(query) => self.search_query = query,
            
            ProfileMessage::Save => {
                self.perform_save_profile();
            }
            
            ProfileMessage::New => {
                self.selected_profile_id = None;
                self.current_profile = Profile::default();
            }
            
            ProfileMessage::Delete => {
                if let Some(id) = self.selected_profile_id {
                    self.profiles.retain(|p| p.id != id);
                    self.selected_profile_id = None;
                    self.current_profile = Profile::default();
                    self.save_profiles();
                }
            }
        }
        Task::none()
    }

    fn perform_save_profile(&mut self) {
        if self.current_profile.ip.is_empty() || self.current_profile.name.is_empty() {
            return;
        }

        // Normalisation du groupe
        if self.current_profile.group.is_empty() {
            self.current_profile.group = "DEFAUT".to_string();
        }
        self.current_profile.group = self.current_profile.group.to_uppercase();

        match self.selected_profile_id {
            Some(id) => {
                if let Some(index) = self.profiles.iter().position(|p| p.id == id) {
                    let mut updated = self.current_profile.clone();
                    updated.id = id;
                    self.profiles[index] = updated;
                }
            }
            None => {
                let mut new_p = self.current_profile.clone();
                new_p.id = uuid::Uuid::new_v4();
                self.selected_profile_id = Some(new_p.id);
                self.profiles.push(new_p);
            }
        }

        self.profiles.sort_by(|a, b| a.group.cmp(&b.group).then(a.name.cmp(&b.name)));
        self.save_profiles();
    }

    // Dans src/ui.rs, à l'intérieur du bloc impl MyApp

fn handle_login_msg(&mut self, msg: LoginMessage) -> Task<Message> {
    match msg {
        // Mise à jour des champs du profil "brouillon"
        LoginMessage::InputIP(ip) => {
            self.current_profile.ip = ip;
            Task::none()
        }
        LoginMessage::InputPort(port) => {
            self.current_profile.port = port;
            Task::none()
        }
        LoginMessage::InputUsername(user) => {
            self.current_profile.username = user;
            Task::none()
        }
        LoginMessage::InputPass(pass) => {
            self.password = pass;
            Task::none()
        }

        // Lancement de la connexion SSH
        LoginMessage::Submit => {
            // On peut ajouter une petite validation ici
            if self.current_profile.ip.is_empty() || self.current_profile.username.is_empty() {
                println!("LOG: Tentative de connexion avortée (champs vides)");
                return Task::none();
            }
            
            println!("LOG: Connexion vers {}...", self.current_profile.ip);
            // On appelle la méthode de connexion que nous avons nettoyée
            self.perform_ssh_connection()
        }
    }
}

fn handle_config_msg(&mut self, msg: ConfigMessage) -> Task<Message> {
    match msg {
        ConfigMessage::SectionChanged(section) => {
            println!("LOG: Changement de section vers : {:?}", section);
            self.active_section = section;
        }
        ConfigMessage::ThemeChanged(new_theme) => {
            self.current_profile.theme = new_theme;
            // On sauvegarde immédiatement pour que le choix persiste au redémarrage
            self.save_profiles();
        }
    }
    Task::none()
}
}
