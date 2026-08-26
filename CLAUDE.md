# CLAUDE.md

Consignes pour Claude Code sur ce dépôt.

**RustTY** — client SSH graphique en Rust (type PuTTY, périmètre volontairement réduit).

## Langue

- Réponds-moi toujours en français : explications, plans, résumés, questions, messages d'erreur reformulés.
- Restent en anglais : le code, les identifiants, les commentaires dans le code, les messages de commit, les noms de branches et la documentation du dépôt (`README`, doc-comments).
- Ce fichier fait exception : il est rédigé en français, c'est voulu.

## Commandes

```bash
cargo run                          # lancement en mode dev
cargo build --release              # build de production
cargo test                         # tous les tests
cargo test <test_name>             # un seul test, ex. cargo test test_handler_routing_success
cargo fmt --all
cargo clippy --all-targets -- -D warnings
make cov                           # rapport de couverture HTML (cargo llvm-cov) servi sur :8080 via miniserve
make clean                         # cargo clean + suppression du dossier de couverture
```

Les tests unitaires sont inline dans `src/ssh.rs` et `src/ui.rs` ; les tests asynchrones utilisent `#[tokio::test]`.

Serveur SSH de test local (jamais de test contre un serveur réel) :

```bash
docker run -d --name sshd-test -p 2222:2222 \
  -e USER_NAME=test -e USER_PASSWORD=test -e PASSWORD_ACCESS=true \
  linuxserver/openssh-server
```

**Avant d'annoncer qu'une tâche est terminée** : `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings` et `cargo test` doivent passer. Pas d'exception.

## Architecture

Application `iced` 0.13 lancée comme `iced::daemon` (multi-fenêtres) plutôt que via le trait `Application` classique — voir `src/main.rs`. Cela permet une fenêtre login/dashboard plus une fenêtre OS indépendante par session SSH ouverte.

**Flux de messages (style Elm)** — `src/messages.rs` définit l'enum `Message`, découpé en sous-domaines : `LoginMessage`, `SshMessage`, `ProfileMessage`, `ConfigMessage`. Le routage passe par `MyApp::update` dans `src/ui.rs`, qui délègue à une méthode `handle_*_msg` dédiée. `MyApp` est l'unique source de vérité pour tout l'état : connexion/auth, parseurs et canaux SSH par fenêtre, liste et édition des profils, buffer de logs.

**Fenêtres et rendu** — `MyApp::view` dispatche sur `window::Id` : si l'id est dans `terminal_window_ids`, rendu via `src/ui/terminal.rs`, sinon via `src/ui/dashboard.rs`. `src/ui/views/login/*` contient les formulaires du dashboard (auth, général, thèmes) ; `src/ui/components/*` les widgets partagés (sidebar, table de recherche, formulaires, barre d'actions, brand) ; `src/ui/constants.rs` les constantes `text_input::Id` qui pilotent le focus.

**Couche SSH (`src/ssh.rs`)** — connexions via `russh`. `MyHandler` implémente `russh::client::Handler` ; sa logique de routage est extraite dans une méthode simple (`handle_data_routing`) précisément pour être testable sans mocker le trait. `SshService::connect` ouvre la connexion en asynchrone via un `Task` `iced::stream::channel` et émet `SshMessage::Connected` ; `SshService::open_shell` demande ensuite un PTY + shell et émet `SshMessage::SetChannel`, le handle étant stocké dans `MyApp.active_channels`.

**Émulation terminal** — chaque fenêtre possède son `vt100::Parser` dans `MyApp.parsers: HashMap<window::Id, vt100::Parser>`, alimenté par les octets bruts de `SshMessage::DataReceived`. Un profil peut ouvrir plusieurs fenêtres en parallèle (`Profile.terminal_count`, borné 1–4), positionnées automatiquement en grille 2x2 selon `spawn_index` lors du traitement de `SshMessage::Connected`. Le clavier est centralisé dans `MyApp::handle_keyboard_event`, qui traduit les événements iced en octets bruts (codes de contrôle Ctrl+touche, touches nommées comme flèches/backspace) et les envoie via `send_to_terminal` à `focused_window_id` (repli sur le terminal ouvert le plus récemment).

**Profils et persistance** — `Profile` (dans `src/models.rs`) est sérialisé vers `profiles.json` dans le répertoire courant (`Profile::load_all` / `save_all`). Ce fichier et `logs/` sont gitignorés : état local par machine, pas données du projet.

**Thèmes** — `src/ui/theme.rs` définit `ThemeChoice` (15 palettes) mappées vers `TerminalColors`, plus les helpers de style (`button_style`, `input_style`, `main_container_style`) partagés entre dashboard, terminal et composants. Le thème est stocké par `Profile`, pas globalement.

**Logs** — `src/main.rs` configure `fern` sur trois sorties : fichier et stdout (Warn par défaut, Debug pour la cible `rustty`), plus une sortie sur canal filtrée sur la cible `rustty`, dont les messages remontent dans l'UI via `Message::LogReceived` (panneau limité à 100 entrées, plus récentes en tête).

**Hors périmètre** — `testrust_simpleterm/` et `testrust_termnano/` sont des crates de bac à sable autonomes (leurs propres `Cargo.toml`/`Cargo.lock`, hors workspace) servant à prototyper `russh` et la gestion du terminal. Elles ne font pas partie du binaire `rustty` et ne sont pas construites par `cargo run`/`cargo build` depuis la racine. N'y touche pas sauf demande explicite.

## Règles de sécurité (non négociables)

- **Vérification de la host key** : ne la contourne, ne l'assouplis ni ne la désactive jamais, même temporairement pour faire passer un test ou débloquer une session. En pratique cela vise `check_server_key` dans `MyHandler` (`src/ssh.rs`) : aucune implémentation qui accepte inconditionnellement, aucun drapeau « accepter les hôtes inconnus » activé par défaut. Si un test a besoin d'un hôte de confiance, ajoute sa clé au magasin de test, ne désactive pas le contrôle.
- **Aucune primitive cryptographique écrite à la main.** Tout passe par `russh` et ses dépendances. S'il manque quelque chose, signale-le, ne l'implémente pas.
- **Comparaisons de secrets, MAC ou empreintes** : uniquement en temps constant (`subtle::ConstantTimeEq`). Ne remplace jamais un `ct_eq` par `==` au cours d'un refactor, même si clippy ou la lisibilité le suggèrent.
- **Secrets** (clés privées, mots de passe, secrets partagés) : `Zeroizing` ou `zeroize()` explicite. Jamais dans un `Debug` dérivé, jamais dans un log — y compris le panneau de logs de l'UI — jamais dans un message d'erreur affiché.
- **Protocole** : le respect des RFC 4251-4254 est délégué à `russh`. Toute logique protocolaire écrite à la main dans ce dépôt doit être commentée avec la référence RFC précise.

## Conventions Rust

- Édition `<À REMPLIR — voir Cargo.toml>`, MSRV `<À REMPLIR>`. N'utilise pas de fonctionnalité de langage plus récente que la MSRV.
- Pas de `unsafe` sans justification écrite et accord préalable.
- Pas de `unwrap()` / `expect()` / `panic!()` en dehors de `src/main.rs` et des tests. Erreurs typées avec `thiserror` ; `anyhow` toléré uniquement dans `src/main.rs`. Une erreur d'I/O réseau ou de parsing doit remonter dans l'UI, pas faire tomber l'application ni une fenêtre.
- **Les données réseau ne sont jamais fiables** : valider toute longueur avant allocation, pas d'indexation directe dans un buffer entrant. Concerne en premier lieu les octets passés à `vt100::Parser` et à `handle_data_routing`.
- Nouvelle dépendance = demander avant. Le projet reste volontairement léger.

## Tests

- Tests unitaires inline dans le module concerné (`#[cfg(test)]`), c'est la convention en place — ne migre pas l'existant vers `tests/`.
- `tests/` est réservé aux tests d'intégration qui parlent réellement au conteneur sshd. Ils doivent se désactiver proprement (`#[ignore]` ou détection du port 2222) si le conteneur n'est pas lancé, pour que `cargo test` reste vert sans Docker.
- Toute fonction qui consomme des octets venant du réseau (`handle_data_routing`, traduction clavier, alimentation du parseur) doit avoir au moins un cas d'entrée malformée ou tronquée.
- Vecteurs de test issus des RFC : les garder tels quels, ne pas les « corriger ».

## Workflow

- Une branche par tâche, commits conventionnels (`feat:`, `fix:`, `refactor:`).
- Diffs petits et ciblés. Pas de refacto opportuniste dans un commit de feature.
- Ne reformate pas des fichiers que la tâche ne touche pas.
- Ne mets pas à jour les dépendances sans demande explicite.
- Ne modifie ni `profiles.json` ni le contenu de `logs/` : état local de la machine.
- Si une consigne de ce fichier semble bloquer une solution, signale-le au lieu de la contourner.
