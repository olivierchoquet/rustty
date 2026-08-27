pub mod messages;
pub mod models;
pub mod ssh;
pub mod ui;

use iced::{Task, futures::SinkExt, widget::text_input, window};
use messages::Message;
use ui::{MyApp, constants::*};

pub fn main() -> iced::Result {
    // Create logs dir if does not exist
    let _ = std::fs::create_dir_all("logs");

    // Log channel
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rx_cell = std::sync::Arc::new(std::sync::Mutex::new(Some(rx)));

    // Base log config
    let base_config = fern::Dispatch::new().format(|out, message, record| {
        out.finish(format_args!(
            "{}[{}][{}] {}",
            chrono::Local::now().format("[%H:%M:%S]"),
            record.target(),
            record.level(),
            message
        ))
    });

    // Log to file -> all messages !
    let file_and_console_config = fern::Dispatch::new()
        .level(log::LevelFilter::Warn)
        //.level_for("zbus", log::LevelFilter::Warn)
        //.level_for("wgpu", log::LevelFilter::Warn)
        .level_for("rustty", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("logs/terminal_app.log").expect("Erreur fichier log"));

    // Log to UI -> only rustty messages !
    let ui_config = fern::Dispatch::new()
        .filter(|metadata| metadata.target().starts_with("rustty"))
        .level(log::LevelFilter::Info)
        .chain(fern::Output::call(move |record| {
            let msg = format!("[{}] {}", record.level(), record.args());
            let _ = tx.send(msg);
        }));

    // Log assembly
    base_config
        .chain(file_and_console_config)
        .chain(ui_config)
        .apply()
        .expect("Erreur initialisation Fern");

    // iced daemon to manage multiple windows and global events
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
                    let rx_opt = rx_cell.lock().unwrap().take();

                    async move {
                        if let Some(mut rx) = rx_opt {
                            while let Some(log_msg) = rx.recv().await {
                                let _ = output.send(Message::LogReceived(log_msg)).await;
                            }
                        } else {
                            // TO AVOID that Iced launch closure in loop
                            // We sleep unused instances
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
                size: iced::Size::new(950.0, 1000.0),
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
