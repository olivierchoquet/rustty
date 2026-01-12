use crate::ui::theme::{self, ThemeChoice, TerminalColors};
use crate::ui::{MAX_TERMINAL_LINES, Message, MyApp, SCROLLABLE_ID};

use iced::widget::{button, column, container, row, scrollable, text, text_input, pick_list};
use iced::{Alignment, Element, Length, Task};

pub fn view(app: &MyApp) -> Element<'_, Message> {
    let colors = app.theme_choice.get_colors();

    // --- 1. BARRE D'ONGLETS & SÉLECTEUR ---
    let tab_bar = container(
        row![
            // Onglet actif
            container(
                text(format!(" 🐚 {} ", &app.ip))
                    .size(13)
                    .font(iced::Font::MONOSPACE)
                    .color(colors.text)
            )
            .padding(iced::Padding { top: 6.0, right: 18.0, bottom: 6.0, left: 18.0 })
            .style(move |_| container::Style {
                background: Some(colors.bg.into()),
                border: iced::Border {
                    width: 1.0,
                    color: colors.accent,
                    radius: iced::border::Radius {
                        top_left: 6.0,
                        top_right: 6.0,
                        ..Default::default()
                    },
                },
                ..Default::default()
            }),

            // LE SÉLECTEUR DE THÈME (PICK LIST)
            pick_list(
                &ThemeChoice::ALL[..],
                Some(app.theme_choice),
                Message::ThemeSelected 
            )
            .text_size(12)
            .padding(5),

            button(text("+").size(16)).style(button::text).padding(10),
        ]
        .spacing(15)
        .align_y(Alignment::Center)
        .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 0.0, left: 12.0 }),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(colors.surface.into()),
        ..Default::default()
    });

    // --- 2. ZONE DE LOGS (LE TERMINAL) ---
    let terminal_logs = scrollable(
        container(
            text(&app.logs)
                .font(iced::Font::MONOSPACE)
                .size(15)
                .line_height(iced::widget::text::LineHeight::Relative(1.5))
                .color(colors.text),
        )
        .padding(25.0)
        .width(Length::Fill),
    )
    .id(scrollable::Id::new(SCROLLABLE_ID))
    .height(Length::Fill)
    .style(move |_theme, _status| scrollable::Style {
        container: container::Style {
            background: Some(colors.bg.into()),
            ..Default::default()
        },
        vertical_rail: scrollable::Rail {
            background: None,
            border: iced::Border::default(),
            scroller: scrollable::Scroller {
                color: colors.accent,
                border: iced::Border { radius: 2.0.into(), width: 0.0, color: iced::Color::TRANSPARENT },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: iced::Border::default(),
            scroller: scrollable::Scroller {
                color: colors.accent,
                border: iced::Border { radius: 2.0.into(), width: 0.0, color: iced::Color::TRANSPARENT },
            },
        },
        gap: None,
    });

    // --- 3. LIGNE DE COMMANDE (PROMPT) ---
    let prompt_line = container(
        row![
            container(
                text(format!(" {} ", app.username))
                    .size(12)
                    .color(colors.bg)
                    .font(iced::Font::MONOSPACE)
            )
            .padding(iced::Padding { top: 3.0, right: 12.0, bottom: 3.0, left: 12.0 })
            .style(move |_| container::Style {
                background: Some(colors.prompt.into()),
                border: iced::Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            text(" ❯ ").color(colors.prompt).size(16).font(iced::Font::MONOSPACE),
            text_input("Tapez une commande...", &app.terminal_input)
                .on_input(Message::InputTerminal)
                .on_submit(Message::SendCommand)
                .font(iced::Font::MONOSPACE)
                .size(15)
                .style(move |_theme, status| theme::input_style(colors, status))
        ]
        .spacing(12)
        .padding(iced::Padding { top: 12.0, right: 20.0, bottom: 12.0, left: 20.0 })
        .align_y(Alignment::Center),
    )
    .style(move |_| theme::main_container_style(colors));

    column![tab_bar, terminal_logs, prompt_line].into()
}

pub fn update(app: &mut MyApp, message: Message) -> Task<Message> {
    match message {
        Message::ThemeSelected(new_theme) => {
            app.theme_choice = new_theme;
        }

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

        Message::SetChannel(ch) => {
            app.active_channel = Some(ch);
            app.logs.push_str(">> Session SSH établie\n");
        }

        _ => {}
    }
    Task::none()
}