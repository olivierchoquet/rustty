1. Installer Rust : https://rust-lang.org/fr/tools/install/
2. Cloner le dépôt
3. cargo run
4. extensions vscode : rust-analyzer et CodeLLDB

## Structure du Projet

```text
Fichier	Son Rôle	Analogie
src/ui/mod.rs	Le Cerveau & la Logique	Le Chef : Il reçoit les ordres (Messages) et décide quoi faire (Update).
src/ui/login.rs	La Structure Globale	La Salle : Il définit où se trouve la Sidebar et où on affiche le contenu.
views/login/general.rs	Page "Général"	Le Menu du jour : Il contient tout le code du tableau et des inputs d'IP.
views/login/auth.rs	Page "Sécurité"	La Caisse : Il ne s'occupe que de l'utilisateur et du mot de passe.
views/login/themes.rs	Page "Thèmes"	La Décoration : Il ne s'occupe que de la grille de couleurs.
```

Remarques :

Rendu visuel (forms, components, ..)

Iced travaille avec des types Element<'a, Message>. Le 'a indique au compilateur : "Ce widget contient des références à des données qui doivent rester valides tant que le widget est affiché". Sans le 'a sur tes &str, Rust a peur que le texte disparaisse de la mémoire alors que l'interface essaie encore de l'afficher.

Refactorisation :

1. Découpage par "Responsabilités" (Fichiers)

Voici une structure de dossiers saine pour ton projet :

    src/main.rs : Point d'entrée, configuration de la fenêtre et lancement.

    src/app.rs : La structure MyApp et la boucle principale (update, subscription).

    src/ui/ : Tout ce qui concerne le rendu visuel.

        mod.rs : Expose les widgets.

        login.rs : Le formulaire (avec tes fonctions render_input_with_label).

        terminal.rs : L'affichage du terminal SSH.

        theme.rs : Tes couleurs et styles.

    src/ssh/ : La logique purement technique (Tokio, Russh).

    src/models/ : Tes structures de données (ex: Profile, Config).


src/
├── main.rs          # Initialisation et lancement
├── app.rs           # La boucle Update et le State central
├── message.rs       # Centralisation de l'Enum Message
├── ui/              # Tout le rendu visuel
│   ├── mod.rs
│   ├── constants.rs # Tes ID_IP, ID_PORT, etc.
│   └── login.rs     # Ton formulaire et render_input_with_label
└── ssh/             # Ta logique de connexion (Russh/Tokio)
    ├── mod.rs
    └── client.rs



src/
├── main.rs         <-- Point d'entrée, déclare "mod ui;" et "mod ssh;"
├── messages.rs     <-- Tes Enums (Message, SshMessage, etc.)
├── ssh.rs          <-- Ta logique réseau/russh
├── ui.rs           <-- Le "cerveau" de l'UI (MyApp, update, view)
└── ui/             <-- Dossier privé de l'UI
    ├── constants.rs
    ├── terminal.rs
    ├── theme.rs
    ├── views.rs    <-- Aiguillage vers les différentes fenêtres
    └── components/ <-- Dossier contenant tes briques (forms, buttons...)