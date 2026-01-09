/*****  Contient uniquement la vue du terminal moderne  ******/

use iced::widget::{column, container, row, scrollable, text, text_input, button};
use iced::{Alignment, Element, Length, Border, Padding};
use crate::ui::{MyApp, Message, SCROLLABLE_ID, COLOR_BG, COLOR_ACCENT, COLOR_TEXT, COLOR_PROMPT};


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