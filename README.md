1. Installer Rust : https://rust-lang.org/fr/tools/install/
2. Cloner le dépôt
3. cargo run
4. extensions vscode : rust-analyzer et CodeLLDB

## Structure du Projet

```text
rustty/
├── Cargo.toml             # Dépendances (iced, serde, etc.)
└── src/
    ├── main.rs            # Point d'entrée
    ├── config.rs          # Gestion du JSON
    └── ui/
        ├── mod.rs         # Déclaration des modules UI
        ├── login.rs       # Vue principale
        ├── theme.rs       # Thèmes et styles
        └── components/    
            ├── mod.rs     
            ├── table.rs   
            ├── sidebar.rs 
            └── forms/     
                ├── mod.rs 
                ├── general.rs
                ├── auth.rs    
                └── themes.rs  
```

