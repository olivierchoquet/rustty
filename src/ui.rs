// iced::futures::SinkExt est nécessaire pour l'envoi asynchrone de messages
use iced::futures::SinkExt;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{scrollable, text_input};
use iced::{Element, Task, window};
use russh::server::Msg;
use russh::{Pty, client};
use std::collections::HashMap;
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
    pub login_window_id: Option<window::Id>,
    pub ssh_handle: Option<crate::ssh::SshHandle>,
    // On stocke l'accès à l'ID du handler ici
    pub handler_id_controller: Option<Arc<Mutex<Option<window::Id>>>>,
    pub terminal_window_ids: Vec<window::Id>,
    // one vt100 parser by terminal window
    pub parsers: HashMap<window::Id, vt100::Parser>,
    // each ssh channel by terminal window
    pub active_channels: HashMap<window::Id, Arc<Mutex<SshChannel>>>,
    pub focus_index: usize, // 0 = IP, 1 = PORT, 2 = USER, 3 = PASS
    // Gestion des profils
    pub current_profile: Profile, // Le "brouillon" lié aux inputs
    pub selected_profile_id: Option<uuid::Uuid>, // L'ID du profil qu'on est en train d'éditer
    pub profiles: Vec<Profile>,
    pub search_query: String, // Pour le filtrage global
    // Catégorie de parammètres en cours d'édition
    pub active_section: EditSection,
    pub focused_id: &'static str, // Pour gérer le focus des TextInput (IP, Port, User, Pass)
    pub focused_window_id: Option<window::Id>,
    pub current_window_index: usize, // Ajoute ceci pour le calcul de position
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
            parsers: HashMap::new(),
            login_window_id: Some(login_id),
            terminal_window_ids: Vec::new(),
            active_channels: HashMap::new(),
            focus_index: 0,
            profiles: loaded_profiles,
            selected_profile_id: None,
            // profil "brouillon" vide au départ
            current_profile: Profile::default(),
            search_query: "".into(),
            active_section: EditSection::General,
            focused_id: ID_PROFILE,
            ssh_handle: None,
            handler_id_controller: None,
            focused_window_id: None,
            current_window_index: 0,
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
            Message::Event(iced::Event::Window(window::Event::Focused)) => {
                // Problème : Iced ne donne pas l'ID ici car l'événement global
                // suppose que vous savez quelle fenêtre est active.
                // Heureusement, il existe une meilleure façon de faire.
                Task::none()
            }

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
            terminal::render(self, window_id)
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
        // 1. On vérifie si c'est un terminal
        if self.terminal_window_ids.contains(&id) {
            self.terminal_window_ids.retain(|&w_id| w_id != id);

            // On récupère et on supprime le canal et le parser associés à CETTE fenêtre
            let channel_to_close = self.active_channels.remove(&id);
            self.parsers.remove(&id); // Nettoyage de la mémoire du parser

            // Tâche de fermeture du canal SSH spécifique
            let close_task = if let Some(ch_arc) = channel_to_close {
                Task::perform(
                    async move {
                        let mut ch = ch_arc.lock().await;
                        let _ = ch.close().await;
                    },
                    |_| Message::DoNothing,
                )
            } else {
                Task::none()
            };

            // Si c'était le dernier, on pourrait aussi réinitialiser d'autres états
            // mais ici on ferme juste proprement la fenêtre et son canal.
            return Task::batch(vec![close_task, window::close(id)]);
        }

        // 2. Si c'est la fenêtre de gestion (Login/Dashboard), on quitte l'app
        if Some(id) == self.login_window_id {
            std::process::exit(0);
        }

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
                // On contraint la valeur entre 1 et 4 pour éviter les configurations extrêmes
                self.current_profile.terminal_count = new_count.clamp(1, 4);
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

                self.current_window_index = 0; // On reset l'index de placement

                // 2. Appel au service SSH (on utilise ce que tu as déjà écrit)
                println!("LOG: Connexion vers {}...", self.current_profile.ip);

                let count = self.current_profile.terminal_count.max(1);
                let mut tasks = Vec::new();

                for _ in 0..count {
                    tasks.push(SshService::connect(
                        self.current_profile.ip.clone(),
                        self.current_profile.port.parse().unwrap_or(22),
                        self.current_profile.username.clone(),
                        self.password.clone(),
                    ));
                }
                Task::batch(tasks)
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
            // 1. Connexion réussie : on récupère le handle ET le contrôleur d'ID
            // 1. UNE connexion a réussi
            SshMessage::Connected(Ok((handle, id_controller))) => {
                let win_w = 850.0;
                let win_h = 550.0;

                // --- NOUVEAUX RÉGLAGES D'ESPACEMENT ---
                let gap_x = 15.0; // Espace horizontal (côte à côte)
                let gap_y = 45.0; // Espace vertical (haut/bas) - On l'augmente ici
                let margin_x = 40.0;
                let margin_y = 30.0;
                // --------------------------------------

                // Calcul de la grille 2x2
                let col = (self.current_window_index % 2) as f32;
                let row = (self.current_window_index / 2) as f32;

                // Utilisation des gaps distincts pour X et Y
                let x = margin_x + (col * (win_w + gap_x));
                let y = margin_y + (row * (win_h + gap_y));

                self.current_window_index += 1;

                let settings = window::Settings {
                    size: (win_w, win_h).into(),
                    position: window::Position::Specific(iced::Point::new(x, y)),
                    exit_on_close_request: true,
                    ..Default::default()
                };

                let (_id, win_task) = window::open(settings);

                win_task.map(move |id| {
                    Message::Ssh(SshMessage::TerminalWindowOpened(
                        id,
                        handle.clone(),
                        id_controller.clone(),
                    ))
                })
            }

            // 2. La fenêtre est prête, on lie l'ID au contrôleur de SA connexion
            SshMessage::TerminalWindowOpened(id, handle, id_controller) => {
                self.terminal_window_ids.push(id);

                // Calcul approximatif : 1 ligne occupe environ 18-20 pixels en hauteur
                // 1 caractère occupe environ 9-10 pixels en largeur
                let rows = 28; // Valeur sûre pour la moitié d'un écran 1080p
                let cols = 100;

                let parser = vt100::Parser::new(rows, cols, MAX_TERMINAL_LINES);
                self.parsers.insert(id, parser);

                // On lance le shell avec le contrôleur dédié à cette session
                crate::ssh::SshService::open_shell(id, handle, id_controller)
            }

            // 3. Réception des données SSH (Le "Prompt")
            SshMessage::DataReceived(id, raw_bytes) => {
                // On ne met à jour QUE le parser de la fenêtre concernée grâce à l'ID
                if let Some(parser) = self.parsers.get_mut(&id) {
                    parser.process(&raw_bytes);
                }
                // --- AJOUT POUR LE SCROLL AUTOMATIQUE ---
                // On génère l'identifiant du scrollable pour cette fenêtre spécifique
                let scroll_id = scrollable::Id::new(format!("scroll_{:?}", id));
    
                // On retourne une tâche qui force le scroll vers le bas (0.0 = top, 1.0 = bottom)
                scrollable::snap_to(scroll_id, scrollable::RelativeOffset::END)


                //Task::none()
            }

            // 4. Stockage du canal actif pour l'envoi de touches
            SshMessage::SetChannel(id, ch) => {
                self.active_channels.insert(id, ch);
                Task::none()
            }

            SshMessage::Connected(Err(e)) => {
                println!("Erreur de connexion : {}", e);
                Task::none()
            }
            SshMessage::WindowFocused(id) => {
                self.focused_window_id = Some(id);
                // Optionnel : on peut aussi forcer le scroll en bas au focus
                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn handle_keyboard_event(&mut self, event: iced::Event) -> Task<Message> {
        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) =
            event
        {
            // --- 1. PRIORITÉ AU FOCUS ---
            // On cherche l'ID de la fenêtre qui doit recevoir les touches
            let target_window_id = self
                .focused_window_id
                .or_else(|| self.terminal_window_ids.last().cloned());

            if let Some(window_id) = target_window_id {
                // On vérifie si on a un canal SSH pour cette fenêtre
                if let Some(channel_arc) = self.active_channels.get(&window_id) {
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
                    return Task::none();
                }
            }

            // --- CAS B : NAVIGATION TAB (LOGIN) ---
            if key == Key::Named(Named::Tab) {
                // ... reste de ton code Tab inchangé ...
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
