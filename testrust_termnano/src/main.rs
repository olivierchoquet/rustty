use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use russh::client;
use russh_keys::{key, ssh_key};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    // Ajoute ceci :
    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        // En retournant true, on dit : "Je fais confiance à cette clé"
        Ok(true)
    }

    async fn data(
        &mut self,
        _id: russh::ChannelId,
        data: &[u8],
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        //tokio::io::stdout().write_all(data).await.unwrap();
        //tokio::io::stdout().flush().await.unwrap();
        //Ok(())
        // En mode RAW, stdout doit être écrit très proprement
        let mut stdout = tokio::io::stdout();
        stdout.write_all(data).await.unwrap();
        stdout.flush().await.unwrap();
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let config = Arc::new(russh::client::Config::default());
    let mut sh = ClientHandler;
    let mut session = client::connect(config, ("server:port"), sh)
        .await
        .unwrap();

    enable_raw_mode().unwrap();

    // Remplace par tes vrais identifiants
    session
        .authenticate_password("user", "mdp.")
        .await
        .unwrap();

    let mut channel = session.channel_open_session().await.unwrap();

    // On demande un PTY et un Shell
    channel
        .request_pty(true, "xterm-256color", 80, 24, 0, 0, &[])
        .await
        .unwrap();
    channel.request_shell(true).await.unwrap();

    println!("--- CONNECTÉ (Tapez vos commandes) ---");

    // Boucle de lecture du clavier (Stdin -> SSH)
    let mut stdin = tokio::io::stdin();
    let mut buf = [0u8; 1024];

    loop {
        // En mode RAW, stdin.read() renverra l'octet dès que tu touches une touche
        match stdin.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                // Si on détecte Ctrl+C (code 3) on pourrait quitter,
                // mais ici on l'envoie au serveur pour qu'il gère.
                if let Err(_) = channel.data(&buf[..n]).await {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    // TRÈS IMPORTANT : Toujours désactiver le mode raw, sinon ton terminal sera cassé après le crash !
    disable_raw_mode().unwrap();
}
