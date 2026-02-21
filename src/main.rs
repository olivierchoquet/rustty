pub mod messages;
pub mod models;
pub mod ssh;
pub mod ui;

use iced::{Task, futures::SinkExt, widget::text_input, window};
use messages::Message;
use ui::{MyApp, constants::*};

pub fn main() -> iced::Result {
    // 1. Ton canal MPSC habituel
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rx_cell = std::sync::Arc::new(std::sync::Mutex::new(Some(rx)));

    // 1. On définit le Dispatcher de base (le tronc commun)
    let base_config = fern::Dispatch::new().format(|out, message, record| {
        out.finish(format_args!(
            "{}[{}][{}] {}",
            chrono::Local::now().format("[%H:%M:%S]"),
            record.target(),
            record.level(),
            message
        ))
    });

    // 2. Configuration pour le FICHIER et la CONSOLE (On veut tout)
    let file_and_console_config = fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .level_for("zbus", log::LevelFilter::Warn)
        .level_for("wgpu", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .chain(fern::log_file("logs/terminal_app.log").expect("Erreur fichier log"));

    // 3. Configuration pour l'UI (UNIQUEMENT ton appli)
    let ui_config = fern::Dispatch::new()
        // On ne laisse passer QUE ce qui commence par le nom de ton crate (rustty)
        .filter(|metadata| metadata.target().starts_with("rustty"))
        .level(log::LevelFilter::Info)
        .chain(fern::Output::call(move |record| {
            let msg = format!("[{}] {}", record.level(), record.args());
            let _ = tx.send(msg);
        }));

    // 4. On lie tout ensemble
    base_config
        .chain(file_and_console_config)
        .chain(ui_config)
        .apply()
        .expect("Erreur initialisation Fern");
    // idec daemon to manage multiple windows and global events
    iced::daemon("RustTy", MyApp::update, MyApp::view)
        //By writing |_|,
        //you were telling Rust: “Receive this argument, but I don't care about it, I'm not going to call it inside my code.”
        //So if we want to use it, then |app|
        .subscription(move |_| {
            let window_events = window::events().map(|(id, event)| match event {
                window::Event::Opened { .. } => Message::WindowOpened(id),
                window::Event::CloseRequested | window::Event::Closed => Message::WindowClosed(id),
                _ => Message::DoNothing,
            });

            let events = iced::event::listen_with(|event, status, _id| {
                match status {
                    // If a widget (like a TextInput) has already captured the event, we do nothing to avoid interference.
                    iced::event::Status::Captured => None,

                    // If the event is not captured, we forward it to the app for processing (like global shortcuts).
                    iced::event::Status::Ignored => Some(Message::Event(event)),
                }
            });

            let rx_cell = rx_cell.clone();
            let log_events = iced::Subscription::run_with_id(
                "global-log-stream",
                iced::stream::channel(100, move |mut output| {
                    // On tente d'extraire le RX.
                    // Si c'est déjà fait (None), cette branche ne fera rien.
                    let rx_opt = rx_cell.lock().unwrap().take();

                    async move {
                        if let Some(mut rx) = rx_opt {
                            println!(">>> SUBSCRIPTION : Connexion au canal établie !");

                            while let Some(log_msg) = rx.recv().await {
                                // CE PRINT EST LE JUGE DE PAIX
                                println!("CANAL REÇOIT : {}", log_msg);
                                let _ = output.send(Message::LogReceived(log_msg)).await;
                            }
                            println!(">>> SUBSCRIPTION : Canal fermé (RX détruit)");
                        } else {
                            // Pour éviter qu'Iced ne relance cette closure en boucle,
                            // on fait dormir les instances "inutiles"
                            loop {
                                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                            }
                        }
                    }
                }),
            );

            iced::Subscription::batch(vec![window_events, events, log_events])
        })
        .run_with(|| {
            // Init the first window and get its ID and the task to open it
            let (id, task) = window::open(window::Settings {
                size: iced::Size::new(950.0, 900.0),
                ..Default::default()
            });

            (
                MyApp::new(id),
                Task::batch(vec![
                    task.discard(),
                    // Focus the TextInput PROFILE_NAME in the new window to allow immediate typing
                    text_input::focus(text_input::Id::new(ID_PROFILE)),
                ]),
            )
        })
}
