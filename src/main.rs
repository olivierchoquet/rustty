pub mod logger;
pub mod messages;
pub mod models;
pub mod ssh;
pub mod ui;

use flexi_logger::{FileSpec, Logger, WriteMode};
use iced::{Task, futures::SinkExt, widget::text_input, window};
use messages::Message;
use ui::{MyApp, constants::*};

pub fn main() -> iced::Result {
    // 1. Créer le canal DEHORS
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    // On enveloppe rx dans un Mutex pour pouvoir l'extraire de la closure Fn
    let rx_cell = std::sync::Arc::new(std::sync::Mutex::new(Some(rx)));

    // Create the log directory if it doesn't exist to prevent panics
    std::fs::create_dir_all("logs").expect("Failed to create logs directory");

    // 2. Initialiser le Logger UNE SEULE FOIS au début
    // Remplace "info" par ce filtre plus sélectif :
// 1. Créer le writer
    let dashboard_writer = Box::new(crate::logger::DashboardWriter::new(tx));

    // 2. Initialiser le logger de façon classique
    let _handle = flexi_logger::Logger::try_with_str(
        "info, zbus=warn, wgpu_core=warn, wgpu_hal=warn, naga=warn, iced_wgpu=warn",
    )
    .unwrap()
    .log_to_file(
        flexi_logger::FileSpec::default()
            .directory("logs")
            .basename("terminal_app"),
    )
    // On utilise add_writer (qui existe partout)
    .add_writer("dashboard", dashboard_writer)
    // Et on utilise duplicate_to_stderr ou duplicate_to_stdout 
    // AVEC l'option Duplicate::All. C'est souvent ce qui déclenche l'envoi aux writers customs.
    .duplicate_to_stderr(flexi_logger::Duplicate::All)
    .start()
    .unwrap();


    log::info!("Système de log initialisé au démarrage");
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
                    loop { tokio::time::sleep(std::time::Duration::from_secs(3600)).await; }
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
