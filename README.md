1. Installer Rust : https://rust-lang.org/fr/tools/install/
2. Cloner le dépôt
3. cargo run
4. extensions vscode : rust-analyzer et CodeLLDB

rustty/
├── Cargo.toml             # Dépendances (iced, serde, etc.)
└── src/
    ├── main.rs            # Point d'entrée, configuration de la fenêtre
    ├── config.rs          # Gestion du JSON (chargement/sauvegarde)
    └── ui/
        ├── mod.rs         # Déclare MyApp, Message et les modules enfants
        ├── login.rs       # LE CHEF D'ORCHESTRE (La fonction view principale)
        ├── theme.rs       # Définition des couleurs et styles (TokyoNight, etc.)
        └── components/    # ÉLÉMENTS VISUELS RÉUTILISABLES
            ├── mod.rs     # Index des composants
            ├── table.rs   # Logique du tableau Zebra (header + lignes)
            ├── sidebar.rs # Barre de navigation latérale
            └── forms/     # DOSSIER DES FORMULAIRES
                ├── mod.rs # Index des formulaires
                ├── general.rs # Champs : Nom, IP, Port
                ├── auth.rs    # Champs : User, Password
                └── themes.rs  # Grille de sélection des thèmes

