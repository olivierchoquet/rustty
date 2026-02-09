1. Installer Rust : https://rust-lang.org/fr/tools/install/
2. Cloner le dépôt
3. cargo run
4. extensions vscode : rust-analyzer et CodeLLDB

## Structure du Projet

src/
├── main.rs         <-- Point d'entrée, déclare "mod ui;" et "mod ssh;"
├── messages.rs     <-- Tes Enums (Message, SshMessage, etc.)
├── models.rs       <-- Contient les structures de données : Profile, EditSection
├── ssh.rs          <-- Ta logique réseau/russh
├── ui.rs           <-- Le "cerveau" de l'UI (MyApp, update, view)
└── ui/             <-- Dossier privé de l'UI
    ├── constants.rs
    ├── terminal.rs
    ├── theme.rs
    ├── views.rs    <-- Aiguillage vers les différentes fenêtres
    ├── views/
    └── components/ <-- Dossier contenant tes briques (forms, buttons...)

## Remarques Rust

Remarques :

Rendu visuel (forms, components, ..)

Iced travaille avec des types Element<'a, Message>. Le 'a indique au compilateur : "Ce widget contient des références à des données qui doivent rester valides tant que le widget est affiché". Sans le 'a sur tes &str, Rust a peur que le texte disparaisse de la mémoire alors que l'interface essaie encore de l'afficher.
