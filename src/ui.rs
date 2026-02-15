//use iced::futures::SinkExt;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{scrollable, text_input};
use iced::{Element, Task, window};
use std::collections::HashMap;
use uuid::Uuid;
// concurrency
use std::sync::Arc;
use tokio::sync::Mutex;

// Internal module imports
use crate::messages::{ConfigMessage, LoginMessage, Message, ProfileMessage, SshMessage};
use crate::models::{EditSection, Profile};
use crate::ssh::{MyHandler, SshChannel, SshService};
use crate::ui::constants::*;
//use vt100;

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

/// Constant for terminal buffer constraints
const MAX_TERMINAL_LINES: usize = 1000;

/// Main Application State
pub struct MyApp {
    // --- Authentication & Connection ---
    /// Temporary password storage for SSH authentication.
    pub password: String,
    pub ssh_handle: Option<crate::ssh::SshHandle>,

    // --- Window Management ---
    /// Tracks the ID of the main dashboard/login window.
    pub login_window_id: Option<window::Id>,
    pub terminal_window_ids: Vec<window::Id>,
    pub focused_window_id: Option<window::Id>,
    /// Tracks the number of opened terminals to calculate grid positioning
    pub spawn_index: usize,

    // --- Terminal Data ---
    /// Maps each window to its own VT100 state parser
    pub parsers: HashMap<window::Id, vt100::Parser>,
    /// Maps each window to its active SSH communication channel
    pub active_channels: HashMap<window::Id, Arc<Mutex<SshChannel>>>,

    // --- UI State ---
    pub profiles: Vec<Profile>,
    pub current_profile: Profile,
    pub selected_profile_id: Option<uuid::Uuid>,
    pub search_query: String,
    pub active_section: EditSection,
    /// Focus for TextInput in the login form (IP, Port, User, Pass)
    pub focused_id: &'static str,
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
            login_window_id: Some(login_id),
            terminal_window_ids: Vec::new(),
            focused_window_id: None,
            spawn_index: 0,
            parsers: HashMap::new(),
            active_channels: HashMap::new(),
            profiles: Profile::load_all(),
            current_profile: Profile::default(),
            selected_profile_id: None,
            search_query: "".into(),
            active_section: EditSection::General,
            focused_id: ID_PROFILE,
            ssh_handle: None,
        }
    }

    // router message
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Login(msg) => self.handle_login_msg(msg),
            Message::Profile(msg) => self.handle_profile_msg(msg),
            Message::Config(msg) => self.handle_config_msg(msg),
            Message::Ssh(msg) => self.handle_ssh_msg(msg),
            Message::Event(event) => self.handle_keyboard_event(event),

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

    /// delegate view rendering to submodules
    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.terminal_window_ids.contains(&window_id) {
            terminal::render(self, window_id)
        } else {
            dashboard::render(self)
        }
    }

    /// proxy method
    /// if save logic changes, only update this method without touching the rest of the codebase
    pub fn save_profiles(&self) {
        Profile::save_all(&self.profiles);
    }

    /// Logic to close a terminal window and clean up associated SSH resources
    fn handle_window_closed(&mut self, id: window::Id) -> Task<Message> {
        // if the closed window is a terminal, we need to:
        // 1. Remove it from the list of active terminal windows
        // 2. Close the associated SSH channel if it exists
        // 3. Clean up the VT100 parser to free memory
        if self.terminal_window_ids.contains(&id) {
            self.terminal_window_ids.retain(|&w_id| w_id != id);
            let channel_to_close = self.active_channels.remove(&id);
            self.parsers.remove(&id);

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
            return Task::batch(vec![close_task, window::close(id)]);
        }

        // if the closed window is the login/dashboard, we want to exit the entire application
        if Some(id) == self.login_window_id {
            std::process::exit(0);
        }

        window::close(id)
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

                self.spawn_index = 0; // On reset l'index de placement

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
            // SSH Connection established, we receive the handle and the ID controller for this session
            SshMessage::Connected(Ok((handle, id_controller))) => {
                let win_w = 850.0;
                let win_h = 550.0;

                // --- rules for window placement ---
                let gap_x = 15.0; // horizontal gap between windows
                let gap_y = 45.0; // vertical gap between windows 
                let margin_x = 40.0;
                let margin_y = 30.0;
                // --------------------------------------

                // Grid 2x2
                let col = (self.spawn_index % 2) as f32;
                let row = (self.spawn_index / 2) as f32;

                let x = margin_x + (col * (win_w + gap_x));
                let y = margin_y + (row * (win_h + gap_y));

                self.spawn_index += 1;

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

            // window opened, we need to initialize the VT100 parser for this window and start the SSH shell
            SshMessage::TerminalWindowOpened(id, handle, id_controller) => {
                self.terminal_window_ids.push(id);

                // default size for the VT100 parser, it will adapt to the actual window size later when we receive the first data chunk
                let rows = 28; 
                let cols = 100;

                let parser = vt100::Parser::new(rows, cols, MAX_TERMINAL_LINES);
                self.parsers.insert(id, parser);

                crate::ssh::SshService::open_shell(id, handle, id_controller)
            }

            // Data received from SSH, we need to feed it to the correct VT100 parser based on the window ID
            SshMessage::DataReceived(id, raw_bytes) => {
                // update the correct parser/window with the new data
                if let Some(parser) = self.parsers.get_mut(&id) {
                    parser.process(&raw_bytes);
                }
                // auto scroll to bottom on new data
                let scroll_id = scrollable::Id::new(format!("scroll_{:?}", id));
                scrollable::snap_to(scroll_id, scrollable::RelativeOffset::END)

            }

            // store the active channel for this window to be able to send data back later
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
                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn handle_keyboard_event(&mut self, event: iced::Event) -> Task<Message> {
        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) =
            event
        {
            // --- CASE A : SENDING DATA TO SSH TERMINAL ---
            let target_window_id = self
                .focused_window_id
                .or_else(|| self.terminal_window_ids.last().cloned());

            if let Some(window_id) = target_window_id {
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

            // --- CASE B : NAVIGATION TAB (LOGIN) ---
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
                // max terminal windows allowed is 4, min is 1
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
}

// pure function no self needed
fn map_key_to_ssh(key: &Key, mods: Modifiers) -> Option<Vec<u8>> {
    // shortcut keyboard combinations with Control (e.g., Ctrl+C, Ctrl+D, etc.)
    // control key is pressed ? 
    if mods.control() {
        if let Key::Character(c) = key {
            let b = c.as_bytes();
            if !b.is_empty() {
                // In ASCII, Ctrl + key corresponds to the key MASKED by 0x1f
                // Example : 'c' (99) & 0x1f = 3 (Code ETX/Ctrl+C)
                return Some(vec![b[0] & 0x1f]);
            }
        }
    }

    // special keys and regular character keys
    match key {
        // regular character keys (a, b, c, 1, 2, etc.)
        Key::Character(c) => Some(c.as_bytes().to_vec()),

        // special named keys that need to be translated to their corresponding SSH byte sequences
        Key::Named(named) => match named {
            Named::Enter => Some(vec![13]),      // Carriage Return
            Named::Backspace => Some(vec![127]), // DEL (standard Linux)
            Named::Tab => Some(vec![9]),         // Horizontal Tab
            Named::Escape => Some(vec![27]),     // ESC

            // escape sequences for arrow keys (common in VT100 terminals)
            Named::ArrowUp => Some(vec![27, 91, 65]),
            Named::ArrowDown => Some(vec![27, 91, 66]),
            Named::ArrowRight => Some(vec![27, 91, 67]),
            Named::ArrowLeft => Some(vec![27, 91, 68]),

            _ => None,
        },
        _ => None,
    }
}
