use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppConfig {
    pub last_ip: String,
    pub last_username: String,
    pub last_port: String,
}

impl AppConfig {
    pub fn load() -> Self {
    if let Some(proj_dirs) = ProjectDirs::from("com", "och", "rust_putty") {
        let path = proj_dirs.config_dir().join("config.json");
        println!("Tentative de lecture du fichier : {:?}", path); // DEBUG
        
        if let Ok(content) = fs::read_to_string(&path) {
            println!("Contenu trouvé : {}", content); // DEBUG
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    println!("Config chargée avec succès : {:?}", config); // DEBUG
                    return config;
                },
                Err(e) => println!("Erreur de désérialisation : {:?}", e),
            }
        } else {
            println!("Aucun fichier de config trouvé à cette adresse.");
        }
    }
    Self::default()
}

    pub fn save(&self) {
        if let Some(proj_dirs) = ProjectDirs::from("com", "och", "rust_putty") {
            let path = proj_dirs.config_dir();

            // 1. Créer le dossier s'il n'existe pas
            if let Err(e) = fs::create_dir_all(path) {
                println!("Erreur création dossier : {:?}", e);
                return;
            }

            let file_path = path.join("config.json");

            // 2. Sérialiser en JSON
            if let Ok(content) = serde_json::to_string_pretty(self) {
                // 3. Écrire le fichier
                match fs::write(&file_path, content) {
                    Ok(_) => println!("Configuration sauvegardée dans : {:?}", file_path),
                    Err(e) => println!("Erreur écriture fichier : {:?}", e),
                }
            }
        } else {
            println!("Impossible de déterminer le dossier de config de l'OS");
        }
    }
}
