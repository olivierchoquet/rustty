// iced::futures::SinkExt est nécessaire pour l'envoi asynchrone de messages
use iced::futures::SinkExt;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{scrollable, text_input};
use iced::{Element, Task, window};
use russh::server::Msg;
use russh::{Pty, client};
use uuid::Uuid;
// Importation des types pour la gestion de la concurrence
use std::sync::Arc;
// Importation de Mutex asynchrone de tokio
use crate::messages::{ConfigMessage, LoginMessage, Message, ProfileMessage, SshMessage};
use crate::models::{EditSection, Profile};
use tokio::sync::Mutex;
// Mes modules internes
use crate::ssh::{MyHandler, SshChannel, SshService};
use crate::ui::constants::*;
use crate::ui::theme::ThemeChoice;
//use crate::ui::views::login;
use vt100;

pub mod constants;
pub mod dashboard;
pub mod terminal;
pub mod theme;
pub mod components {
    pub mod actions_bar;
    pub mod brand;
    pub mod forms;
    pub mod search_table;
    pub mod sidebar;
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
    //pub terminal_window_id: Option<window::Id>,
    pub terminal_window_ids: Vec<window::Id>,
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
            terminal_window_ids: Vec::new(),
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

    // router message
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Login(msg) => self.handle_login_msg(msg),
            Message::Profile(msg) => self.handle_profile_msg(msg),
            Message::Config(msg) => self.handle_config_msg(msg),
            Message::Ssh(msg) => self.handle_ssh_msg(msg),
            Message::Event(event) => self.handle_keyboard_event(event), // Utilise la fonction dédiée !

            Message::QuitRequested => std::process::exit(0),

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

    // delegate view rendering to submodules
    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.terminal_window_ids.contains(&window_id) {
            terminal::render(self)
        } else {
            dashboard::render(self)
        }
    }

    // proxy method
    // if save logic changes, only update this method without touching the rest of the codebase
    pub fn save_profiles(&self) {
        Profile::save_all(&self.profiles);
    }

    fn handle_window_closed(&mut self, id: window::Id) -> Task<Message> {
        // 1. On vérifie si l'ID fermé appartient à notre liste de terminaux
        if self.terminal_window_ids.contains(&id) {
            // On retire cet ID spécifique de la liste
            self.terminal_window_ids.retain(|&w_id| w_id != id);

            // Si c'était le dernier terminal ouvert, on coupe le SSH
            if self.terminal_window_ids.is_empty() {
                if let Some(ch_arc) = self.active_channel.take() {
                    // On ferme le canal asynchronement, puis on ferme la fenêtre
                    return Task::batch(vec![
                        Task::perform(
                            async move {
                                let ch = ch_arc.lock().await;
                                let _ = ch.close().await;
                            },
                            |_| Message::DoNothing,
                        ),
                        window::close(id),
                    ]);
                }
            }
            // S'il reste d'autres terminaux, on ferme juste cette fenêtre
            return window::close(id);
        }

        // 2. Si c'est la fenêtre de gestion (Login/Dashboard), on quitte l'app
        if Some(id) == self.login_window_id {
            std::process::exit(0);
        }

        // 3. Par sécurité pour toute autre fenêtre
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
            ProfileMessage::TerminalCountChanged(new_count) => {
                // On contraint la valeur entre 1 et 5
                self.current_profile.terminal_count = new_count.clamp(1, 5);
            }

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

        self.profiles
            .sort_by(|a, b| a.group.cmp(&b.group).then(a.name.cmp(&b.name)));
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
                // 1. Validation de sécurité
                if self.current_profile.ip.is_empty() || self.current_profile.username.is_empty() {
                    println!("LOG: Champs manquants pour la connexion.");
                    return Task::none();
                }

                // 2. Appel au service SSH (on utilise ce que tu as déjà écrit)
                println!("LOG: Connexion vers {}...", self.current_profile.ip);

                SshService::connect(
                    self.current_profile.ip.clone(),
                    self.current_profile.port.parse().unwrap_or(22),
                    self.current_profile.username.clone(),
                    self.password.clone(),
                )
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
   
   fn handle_ssh_msg(&mut self, msg: SshMessage) -> Task<Message> {
    match msg {
        // 1. Ouverture des fenêtres en grille
        SshMessage::Connected(Ok(handle)) => {
            let count = self.current_profile.terminal_count.max(1);
            let mut tasks = Vec::new();
            let win_w = 900.0;
            let win_h = 600.0;
            let gap = 20.0;
            let margin = 80.0;
            let max_cols = 2;

            for i in 0..count {
                let col = (i as u32) % max_cols;
                let row = (i as u32) / max_cols;
                let x = margin + (col as f32 * (win_w + gap));
                let y = margin + (row as f32 * (win_h + gap));

                let settings = window::Settings {
                    size: (win_w, win_h).into(),
                    position: window::Position::Specific(iced::Point::new(x, y)),
                    exit_on_close_request: true,
                    ..Default::default()
                };

                let (_id, win_task) = window::open(settings);
                let handle_for_closure = handle.clone(); 

                tasks.push(
                    win_task.map(move |id| {
                        Message::Ssh(SshMessage::TerminalWindowOpened(id, handle_for_closure.clone()))
                    })
                );
            }
            Task::batch(tasks)
        }

        // 2. Initialisation du shell quand la fenêtre est prête
        SshMessage::TerminalWindowOpened(id, handle) => {
            self.terminal_window_ids.push(id);
            crate::ssh::SshService::open_shell(handle)
        }

        // 3. Réception des données SSH (Le "Prompt")
        SshMessage::DataReceived(raw_bytes) => {
            // On envoie les octets bruts au parser VT100 pour interpréter les couleurs et le texte
            self.parser.process(&raw_bytes);
            
            // On force le scroll vers le bas pour voir les dernières lignes
            scrollable::snap_to::<Message>(
                scrollable::Id::new(SCROLLABLE_ID),
                scrollable::RelativeOffset::END,
            )
        }

        // 4. Stockage du canal actif pour l'envoi de touches
        SshMessage::SetChannel(ch) => {
            self.active_channel = Some(ch);
            Task::none()
        }

        _ => Task::none(),
    }
}

    fn handle_keyboard_event(&mut self, event: iced::Event) -> Task<Message> {
        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) =
            event
        {
            // CAS A : ON EST CONNECTÉ (Priorité absolue au SSH)
            if let Some(channel_arc) = &self.active_channel {
                if let Some(bytes) = map_key_to_ssh(&key, modifiers) {
                    let arc = channel_arc.clone();
                    return Task::perform(
                        async move {
                            let mut ch = arc.lock().await;
                            let _ = ch.data(&bytes[..]).await;
                        },
                        |_| Message::DoNothing,
                    );
                }
                return Task::none(); // On stoppe ici si on est connecté
            }

            // CAS B : MODE LOGIN (Navigation)
            if key == Key::Named(Named::Tab) {
                let next_id = match self.focused_id {
                    ID_PROFILE => ID_GROUP,
                    ID_GROUP => ID_IP,
                    ID_IP => ID_PORT,
                    ID_PORT => ID_USER,
                    ID_USER => ID_PASS,
                    _ => ID_PROFILE,
                };
                self.focused_id = next_id;
                return text_input::focus(text_input::Id::new(next_id));
            }
        }
        Task::none()
    }
}

// pure function no self needed
fn map_key_to_ssh(key: &Key, mods: Modifiers) -> Option<Vec<u8>> {
    // 1. Gestion des raccourcis CTRL (ex: Ctrl+C)
    // On vérifie si la touche Control est pressée
    if mods.control() {
        if let Key::Character(c) = key {
            let b = c.as_bytes();
            if !b.is_empty() {
                // En ASCII, Ctrl + touche correspond à la touche MASQUÉE par 0x1f
                // Exemple: 'a' (97) & 0x1f = 1 (Code SOH/Ctrl+A)
                return Some(vec![b[0] & 0x1f]);
            }
        }
    }

    // 2. Mapping des touches normales et spéciales
    // Note l'utilisation de &key pour ne pas "déplacer" la valeur
    match key {
        // Touches de texte classiques
        Key::Character(c) => Some(c.as_bytes().to_vec()),

        // Touches nommées (spéciales)
        Key::Named(named) => match named {
            Named::Enter => Some(vec![13]),      // Carriage Return
            Named::Backspace => Some(vec![127]), // DEL (standard Linux)
            Named::Tab => Some(vec![9]),         // Horizontal Tab
            Named::Escape => Some(vec![27]),     // ESC

            // Séquences d'échappement ANSI pour les flèches
            Named::ArrowUp => Some(vec![27, 91, 65]),
            Named::ArrowDown => Some(vec![27, 91, 66]),
            Named::ArrowRight => Some(vec![27, 91, 67]),
            Named::ArrowLeft => Some(vec![27, 91, 68]),

            _ => None,
        },
        _ => None,
    }
}
