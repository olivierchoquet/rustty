/*****  Contient uniquement la vue du terminal moderne  ******/

use crate::ui::{
    COLOR_ACCENT, COLOR_BG, COLOR_PROMPT, COLOR_TEXT, MAX_TERMINAL_LINES, Message, MyApp,
    SCROLLABLE_ID,
};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Padding, Task};

pub fn view(app: &MyApp) -> Element<'_, Message> {
    // 1. Définition du contenu principal (Colonne)
    let content = column![
        // --- BARRE D'ONGLETS ---
        container(
            row![
                // L'onglet actif avec une bordure bleue
                container(
                    text(format!(" 🐚 {} ", &app.ip))
                        .size(12)
                        .font(iced::Font::MONOSPACE)
                )
                .padding([5.0, 15.0])
                .style(|_| container::Style {
                    background: Some(COLOR_BG.into()),
                    border: iced::Border {
                        width: 1.0,
                        color: COLOR_ACCENT,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }),
                // Petit bouton + factice pour le look
                button(text("+").size(14)).style(button::text),
            ]
            .spacing(5)
            .padding(iced::Padding {
                top: 5.0,
                right: 10.0,
                bottom: 0.0,
                left: 10.0,
            })
        )
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Color::from_rgb(0.12, 0.14, 0.18).into()),
            ..Default::default()
        }),
        // --- ZONE DE LOGS (LE TERMINAL) ---
        scrollable(
            container(
                text(&app.logs)
                    .font(iced::Font::MONOSPACE)
                    .size(14)
                    .line_height(iced::widget::text::LineHeight::Relative(1.2))
            )
            .padding(20.0)
            .width(Length::Fill)
        )
        .id(scrollable::Id::new(SCROLLABLE_ID))
        .height(Length::Fill),
        // --- LIGNE DE COMMANDE (PROMPT) ---
        container(
            row![
                // Badge Utilisateur
                container(
                    text(format!(" {} ", app.username))
                        .size(11)
                        .color(iced::Color::BLACK)
                        .font(iced::Font::MONOSPACE)
                )
                .padding([2.0, 8.0])
                .style(|_| container::Style {
                    background: Some(COLOR_PROMPT.into()),
                    ..Default::default()
                }),
                // Séparateur stylisé
                text(" ❯ ").color(COLOR_PROMPT).font(iced::Font::MONOSPACE),
                // Champ de saisie invisible
                text_input("", &app.terminal_input)
                    .on_input(Message::InputTerminal)
                    .on_submit(Message::SendCommand)
                    .font(iced::Font::MONOSPACE)
                    .style(|_theme, _status| text_input::Style {
                        background: iced::Color::TRANSPARENT.into(),
                        border: iced::Border {
                            width: 0.0,
                            color: iced::Color::TRANSPARENT,
                            radius: 0.0.into(),
                        },
                        value: COLOR_TEXT,
                        placeholder: iced::Color::from_rgb(0.4, 0.4, 0.4),
                        selection: iced::Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                        icon: COLOR_TEXT,
                    })
            ]
            .padding([8.0, 15.0])
            .align_y(Alignment::Center)
        )
        .style(|_| container::Style {
            background: Some(COLOR_BG.into()),
            ..Default::default()
        })
    ];

    // 2. Encapsulation finale dans un container pour le fond global
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(COLOR_BG.into()),
            ..Default::default()
        })
        .into()
}

pub fn update(app: &mut MyApp, message: Message) -> Task<Message> {
    match message {
        Message::SshData(data) => {
            // 1. Mise à jour des données
            app.logs.push_str(&data);

            // 2. Limitation du nombre de lignes
            let lines: Vec<&str> = app.logs.lines().collect();
            if lines.len() > MAX_TERMINAL_LINES {
                app.logs = lines[lines.len() - MAX_TERMINAL_LINES..].join("\n");
                app.logs.push('\n');
            }

            // 3. Retourner la Task de scroll avec le Turbofish ::<scrollable::Id>
            // On utilise AbsoluteOffset ou RelativeOffset::END
            return scrollable::snap_to::<scrollable::Id>(
                scrollable::Id::new(SCROLLABLE_ID),
                scrollable::RelativeOffset::END,
            )
            .map(|_| Message::DoNothing);
        }

        Message::HistoryPrev => {
            if !app.history.is_empty() {
                let new_index = match app.history_index {
                    None => app.history.len().checked_sub(1),
                    Some(idx) => idx.checked_sub(1),
                };
                if let Some(idx) = new_index {
                    app.history_index = Some(idx);
                    app.terminal_input = app.history[idx].clone();
                }
            }
        }

        Message::HistoryNext => {
            if let Some(idx) = app.history_index {
                let next_idx = idx + 1;
                if next_idx < app.history.len() {
                    app.history_index = Some(next_idx);
                    app.terminal_input = app.history[next_idx].clone();
                } else {
                    app.history_index = None;
                    app.terminal_input.clear();
                }
            }
        }

        // Stockage du canal SSH actif
        Message::SetChannel(ch) => {
            app.active_channel = Some(ch);
            app.logs.push_str("--- SESSION ÉTABLIE ---\n");
        }

        // Envoi de la commande tapée par l'utilisateur
        Message::SendCommand => {
            // 1. On vérifie si l'input n'est pas vide
            if !app.terminal_input.is_empty() {
                // 2. On gère l'historique
                // On n'ajoute à l'historique que si c'est différent de la dernière commande (évite les doublons)
                if app.history.last() != Some(&app.terminal_input) {
                    app.history.push(app.terminal_input.clone());
                }
                // On réinitialise l'index de navigation car on repart sur une nouvelle saisie
                app.history_index = None;

                // 3. Logique d'envoi SSH existante
                if let Some(ch_arc) = &app.active_channel {
                    let cmd = format!("{}\n", app.terminal_input);
                    app.terminal_input.clear();
                    let ch_clone = ch_arc.clone();

                    return Task::perform(
                        async move {
                            let mut ch = ch_clone.lock().await;
                            let _ = ch.data(cmd.as_bytes()).await;
                        },
                        |_| Message::DoNothing,
                    );
                }
            }
        }
        // Mise à jour de l'entrée terminal
        Message::InputTerminal(input) => {
            app.terminal_input = input;
        }

        _ => {}
    }
    Task::none()
}
