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

    async fn data(&mut self, _id: russh::ChannelId, data: &[u8], _session: &mut client::Session) -> Result<(), Self::Error> {
        tokio::io::stdout().write_all(data).await.unwrap();
        tokio::io::stdout().flush().await.unwrap();
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let config = Arc::new(russh::client::Config::default());
    let mut sh = ClientHandler;
    let mut session = client::connect(config, ("server:port"), sh).await.unwrap();

    // Remplace par tes vrais identifiants
    session.authenticate_password("user", "mdp").await.unwrap();

    let mut channel = session.channel_open_session().await.unwrap();
    
    // On demande un PTY et un Shell
    channel.request_pty(true, "xterm", 80, 24, 0, 0, &[]).await.unwrap();
    channel.request_shell(true).await.unwrap();

    println!("--- CONNECTÉ (Tapez vos commandes) ---");

    // Boucle de lecture du clavier (Stdin -> SSH)
    let mut stdin = tokio::io::stdin();
    let mut buf = [0u8; 1024];

    loop {
        let n = stdin.read(&mut buf).await.unwrap();
        if n == 0 { break; }
        channel.data(&buf[..n]).await.unwrap();
    }
}
