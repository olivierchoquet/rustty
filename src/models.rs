use std::path::Path;

// src/models.rs
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::ui::theme::ThemeChoice;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub ip: String,
    pub port: String,
    pub username: String,
    pub group: String,
    pub theme: ThemeChoice,
    pub terminal_count: usize
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.group.to_uppercase(), self.name)
    }
}

impl Profile {
    const FILE_PATH: &'static str = "profiles.json";

    pub fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: "Nouveau Profil".into(),
            group: "DEFAUT".into(),
            ip: "".into(),
            port: "22".into(),
            username: "".into(),
            theme: crate::ui::theme::ThemeChoice::Slate, 
            terminal_count: 1,
        }
    }

    /// Charge tous les profils depuis le disque
    pub fn load_all() -> Vec<Self> {
        if !Path::new(Self::FILE_PATH).exists() {
            return Vec::new();
        }

        match std::fs::read_to_string(Self::FILE_PATH) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_else(|e| {
                eprintln!("Erreur JSON: {}", e);
                Vec::new()
            }),
            Err(_) => Vec::new(),
        }
    }

    /// Sauvegarde une liste complète de profils
    pub fn save_all(profiles: &[Self]) {
        if let Ok(json) = serde_json::to_string_pretty(profiles) {
            if let Err(e) = std::fs::write(Self::FILE_PATH, json) {
                eprintln!("Erreur d'écriture: {}", e);
            }
        }
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