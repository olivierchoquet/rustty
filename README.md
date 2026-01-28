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

