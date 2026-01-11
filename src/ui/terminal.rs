use crate::ui::theme::{self, COLOR_ACCENT, COLOR_BG, COLOR_PROMPT, COLOR_TEXT};
use crate::ui::{MAX_TERMINAL_LINES, Message, MyApp, SCROLLABLE_ID};

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Task};

pub fn view(app: &MyApp) -> Element<'_, Message> {
    // --- 1. BARRE D'ONGLETS (Style Flat & Épuré) ---
    let tab_bar = container(
        row![
            container(
                text(format!(" 🐚 {} ", &app.ip))
                    .size(13)
                    .font(iced::Font::MONOSPACE)
                    .color(COLOR_TEXT)
            )
            .padding([6.0, 18.0])
            .style(|_| container::Style {
                background: Some(COLOR_BG.into()),
                border: iced::Border {
                    width: 1.0,
                    color: COLOR_ACCENT,
                    radius: iced::border::Radius {
                        top_left: 6.0,
                        top_right: 6.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    },
                },
                ..Default::default()
            }),
            button(text("+").size(16)).style(button::text).padding(10),
        ]
        .spacing(8)
        .padding(iced::Padding {
            top: 8.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        }),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Color::from_rgb(0.14, 0.16, 0.20).into()), // Un gris légèrement bleuté
        ..Default::default()
    });

    // --- 2. ZONE DE LOGS (Plus d'espace pour la lisibilité) ---
    let terminal_logs = scrollable(
        container(
            text(&app.logs)
                .font(iced::Font::MONOSPACE)
                .size(15)
                .line_height(iced::widget::text::LineHeight::Relative(1.5)) // Plus d'espace entre les lignes
                .color(COLOR_TEXT),
        )
        .padding(25.0) // Marges plus larges pour le look moderne
        .width(Length::Fill),
    )
    .id(scrollable::Id::new(SCROLLABLE_ID))
    .height(Length::Fill)
    .style(|_theme, _status| {
        scrollable::Style {
            container: container::Style {
                background: Some(COLOR_BG.into()),
                ..Default::default()
            },
            gap: None,
            horizontal_rail: scrollable::Rail {
                background: None,
                border: iced::Border::default(),
                scroller: scrollable::Scroller {
                    color: COLOR_ACCENT,
                    border: iced::Border::default(),
                },
            },
            vertical_rail: scrollable::Rail {
                background: None,
                border: iced::Border::default(),
                scroller: scrollable::Scroller {
                    color: COLOR_ACCENT,
                    border: iced::Border::default(),
                },
            },
        }
    });

    // --- 3. LIGNE DE COMMANDE (Le "Prompt" moderne) ---
    let prompt_line = container(
        row![
            // Badge Utilisateur style "Pilule"
            container(
                text(format!(" {} ", app.username))
                    .size(12)
                    .color(COLOR_BG) // Texte sombre sur fond clair pour le contraste
                    .font(iced::Font::MONOSPACE)
            )
            .padding([3.0, 12.0])
            .style(|_| container::Style {
                background: Some(COLOR_PROMPT.into()), // Vert menthe / émeraude
                border: iced::Border {
                    radius: 12.0.into(), // Forme pilule
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                ..Default::default()
            }),
            text(" ❯ ")
                .color(COLOR_PROMPT)
                .size(16)
                .font(iced::Font::MONOSPACE),
            // Champ de saisie (sans bordures pour le look épuré)
            text_input("Tapez une commande...", &app.terminal_input)
                .on_input(Message::InputTerminal)
                .on_submit(Message::SendCommand)
                .font(iced::Font::MONOSPACE)
                .size(15)
                .style(theme::input_style)
        ]
        .spacing(12)
        .padding([12.0, 20.0])
        .align_y(Alignment::Center),
    )
    .style(|_| container::Style {
        background: Some(iced::Color::from_rgb(0.08, 0.10, 0.13).into()), // Fond de barre de commande plus sombre
        border: iced::Border {
            width: 1.0,
            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.03), // Ligne de séparation très subtile
            ..Default::default()
        },
        ..Default::default()
    });

    // Assemblage final
    column![tab_bar, terminal_logs, prompt_line].into()
}

pub fn update(app: &mut MyApp, message: Message) -> Task<Message> {
    match message {
        Message::SshData(data) => {
            app.logs.push_str(&data);

            let lines: Vec<&str> = app.logs.lines().collect();
            if lines.len() > MAX_TERMINAL_LINES {
                app.logs = lines[lines.len() - MAX_TERMINAL_LINES..].join("\n");
                app.logs.push('\n');
            }

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

        Message::SetChannel(ch) => {
            app.active_channel = Some(ch);
            app.logs.push_str(">> Session SSH établie avec succès\n");
        }

        Message::SendCommand => {
            if !app.terminal_input.is_empty() {
                if app.history.last() != Some(&app.terminal_input) {
                    app.history.push(app.terminal_input.clone());
                }
                app.history_index = None;

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

        Message::InputTerminal(input) => {
            app.terminal_input = input;
        }

        _ => {}
    }
    Task::none()
}
